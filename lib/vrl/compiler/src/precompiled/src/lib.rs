use std::collections::BTreeMap;
use vrl_core::{
    Context, ExpressionError, LookupBuf, Resolved, Span, Target, Value, VrlValueArithmetic,
};

// We only want to precompile the stub for this function, and therefore don't
// reference the function arguments.
// The function body will be implemented in our code generation framework.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn vrl_execute(context: &mut Context, result: &mut Resolved) {}

#[no_mangle]
pub extern "C" fn vrl_resolved_initialize(result: *mut Resolved) {
    // println!("vrl_resolved_initialize");
    unsafe { result.write(Ok(Value::Null)) };
}

#[no_mangle]
pub extern "C" fn vrl_resolved_drop(result: *mut Resolved) {
    // println!("vrl_resolved_drop");
    drop(unsafe { result.read() });
}

#[no_mangle]
pub extern "C" fn vrl_resolved_err_into_ok(result: &mut Resolved) {
    // println!("vrl_resolved_err_into_ok");
    *result = Ok(match result {
        Err(error) => error.to_string().into(),
        _ => panic!(r#"expected value "{:?}" to be an error"#, result),
    })
}

#[no_mangle]
pub extern "C" fn vrl_resolved_is_ok(result: &mut Resolved) -> bool {
    // println!("vrl_resolved_is_ok");
    result.is_ok()
}

#[no_mangle]
pub extern "C" fn vrl_resolved_is_err(result: &mut Resolved) -> bool {
    // println!("vrl_resolved_is_err");
    result.is_err()
}

#[no_mangle]
pub extern "C" fn vrl_resolved_boolean_is_true(result: &Resolved) -> bool {
    // println!("vrl_resolved_boolean_is_true");
    result
        .as_ref()
        .expect("VRL result must not contain an error")
        .as_boolean()
        .expect("VRL value must be boolean")
}

#[no_mangle]
pub extern "C" fn vrl_btree_map_initialize(map: *mut BTreeMap<String, Value>) {
    // println!("vrl_btree_map_initialize");
    unsafe { map.write(Default::default()) };
}

#[no_mangle]
pub extern "C" fn vrl_btree_map_drop(map: *mut BTreeMap<String, Value>) {
    // println!("vrl_btree_map_drop");
    drop(unsafe { map.read() });
}

#[no_mangle]
pub extern "C" fn vrl_target_assign(value: &Resolved, target: &mut Resolved) {
    // println!("vrl_btree_map_drop");
    *target = value.clone()
}

#[no_mangle]
pub extern "C" fn vrl_btree_map_insert(
    map: &mut BTreeMap<String, Value>,
    key: &String,
    result: &mut Resolved,
) {
    // println!("vrl_btree_map_insert");
    let result = {
        let mut moved = Ok(Value::Null);
        std::mem::swap(result, &mut moved);
        moved
    };
    map.insert(
        key.clone(),
        result.expect("VRL result must not contain an error"),
    );
}

#[no_mangle]
pub extern "C" fn vrl_expression_abort_impl(
    span: &Span,
    message: &Resolved,
    result: &mut Resolved,
) {
    // println!("vrl_expression_abort_impl");
    let message = match message {
        Ok(Value::Null) => None,
        Ok(Value::Bytes(bytes)) => Some(String::from_utf8_lossy(bytes).into()),
        _ => panic!(r#"abort message "{:?}" is not a string"#, message),
    };
    *result = Err(ExpressionError::Abort {
        span: *span,
        message,
    });
}

#[no_mangle]
pub extern "C" fn vrl_expression_assignment_target_insert_internal_path_impl(
    value: &Resolved,
    path: &LookupBuf,
    target: &mut Resolved,
) {
    // println!("vrl_expression_assignment_target_insert_internal_path_impl");
    let value = value
        .as_ref()
        .expect("assignment value must not contain an error");
    let target = target
        .as_mut()
        .expect("assignment target must not contain an error");
    target.insert_by_path(path, value.clone())
}

#[no_mangle]
pub extern "C" fn vrl_expression_assignment_target_insert_external_impl(
    value: &Resolved,
    path: &LookupBuf,
    ctx: &mut Context,
) {
    // println!("vrl_expression_assignment_target_insert_external_impl");
    let value = value
        .as_ref()
        .expect("assignment value must not contain an error");
    let _ = ctx.target.target_insert(path, value.clone());
}

#[no_mangle]
pub extern "C" fn vrl_expression_literal_impl(value: &Value, result: &mut Resolved) {
    // println!("vrl_expression_literal_impl");
    *result = Ok(value.clone());
}

#[no_mangle]
pub extern "C" fn vrl_expression_not_impl(result: &mut Resolved) {
    // println!("vrl_expression_not_impl");
    *result = Ok((!result
        .as_ref()
        .expect("VRL result must not contain an error")
        .as_boolean()
        .expect("VRL value must be boolean"))
    .into());
}

#[no_mangle]
pub extern "C" fn vrl_expression_object_set_result_impl(
    map: &mut BTreeMap<String, Value>,
    result: &mut Resolved,
) {
    // println!("vrl_expression_object_set_result_impl");
    let map = {
        let mut moved = Default::default();
        std::mem::swap(map, &mut moved);
        moved
    };
    *result = Ok(Value::Object(map));
}

#[no_mangle]
pub extern "C" fn vrl_expression_op_add_impl(rhs: &mut Resolved, result: &mut Resolved) {
    // println!("vrl_expression_op_add_impl");
    let rhs = {
        let mut moved = Ok(Value::Null);
        std::mem::swap(rhs, &mut moved);
        moved
    };
    match (&result, rhs) {
        (Ok(lhs), Ok(rhs)) => {
            *result = lhs.clone().try_add(rhs).map_err(Into::into);
        }
        (Err(_), _) => (),
        (_, Err(error)) => {
            *result = Err(error);
        }
    }
}

#[no_mangle]
pub extern "C" fn vrl_expression_op_eq_impl(rhs: &mut Resolved, result: &mut Resolved) {
    // println!("vrl_expression_op_eq_impl");
    let rhs = {
        let mut moved = Ok(Value::Null);
        std::mem::swap(rhs, &mut moved);
        moved
    };
    match (&result, rhs) {
        (Ok(lhs), Ok(rhs)) => {
            *result = Ok(lhs.eq_lossy(&rhs).into());
        }
        (Err(_), _) => (),
        (_, Err(error)) => {
            *result = Err(error);
        }
    }
}

#[no_mangle]
pub extern "C" fn vrl_expression_query_target_external_impl(
    context: &mut Context,
    path: &LookupBuf,
    result: &mut Resolved,
) {
    // println!("vrl_expression_query_target_external_impl");
    *result = Ok(context
        .target
        .target_get(path)
        .ok()
        .flatten()
        .unwrap_or(Value::Null));
}

#[no_mangle]
pub extern "C" fn vrl_expression_query_target_impl(path: &LookupBuf, result: &mut Resolved) {
    // println!("vrl_expression_query_target_impl");
    *result = Ok(result
        .as_ref()
        .expect("VRL result must not contain an error")
        .target_get(path)
        .ok()
        .flatten()
        .unwrap_or(Value::Null));
}
