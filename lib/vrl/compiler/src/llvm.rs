use crate::Resolved;
use inkwell::{
    builder::Builder,
    module::Module,
    values::{FunctionValue, GlobalValue, PointerValue},
};
use lookup::LookupBuf;
use parser::ast::Ident;
use std::collections::HashMap;

static PRECOMPILED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/precompiled.bc"));

pub struct Context<'ctx> {
    context: &'ctx inkwell::context::Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    function: FunctionValue<'ctx>,
    context_ref: PointerValue<'ctx>,
    result_ref: PointerValue<'ctx>,
    variable_map: HashMap<Ident, usize>,
    // TODO: Emit code to drop variables when finishing the module.
    variables: Vec<PointerValue<'ctx>>,
    resolved_map: HashMap<Resolved, usize>,
    resolveds: Vec<GlobalValue<'ctx>>,
    lookup_buf_map: HashMap<LookupBuf, usize>,
    lookup_bufs: Vec<GlobalValue<'ctx>>,
}

impl<'ctx> Context<'ctx> {
    pub fn context(&self) -> &'ctx inkwell::context::Context {
        self.context
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn builder(&self) -> &Builder<'ctx> {
        &self.builder
    }

    pub fn function(&self) -> inkwell::values::FunctionValue<'ctx> {
        self.function
    }

    pub fn context_ref(&self) -> inkwell::values::PointerValue<'ctx> {
        self.context_ref
    }

    pub fn result_ref(&self) -> inkwell::values::PointerValue<'ctx> {
        self.result_ref
    }

    pub fn set_result_ref(&mut self, result_ref: inkwell::values::PointerValue<'ctx>) {
        self.result_ref = result_ref
    }

    pub fn get_or_insert_variable_ref(
        &mut self,
        ident: &Ident,
    ) -> inkwell::values::PointerValue<'ctx> {
        let index = self.variable_map.get(ident).cloned().unwrap_or_else(|| {
            let position = self
                .builder
                .get_insert_block()
                .expect("builder must be positioned at block");
            if let Some(instruction) = self
                .function
                .get_first_basic_block()
                .and_then(|block| block.get_first_instruction())
            {
                self.builder.position_before(&instruction);
            }
            let variable = self.build_alloca_resolved(ident);
            self.builder.position_at_end(position);
            let index = self.variables.len();
            self.variables.push(variable);
            self.variable_map.insert(ident.clone(), index);
            index
        });

        self.variables[index]
    }

    pub fn get_variable_ref(&mut self, ident: &Ident) -> inkwell::values::PointerValue<'ctx> {
        let index = self
            .variable_map
            .get(ident)
            .unwrap_or_else(|| panic!(r#"unknown variable "{}""#, ident));
        self.variables[*index]
    }

    pub fn into_const<T: Sized>(&self, value: T, name: &str) -> inkwell::values::GlobalValue<'ctx> {
        let size = std::mem::size_of::<T>();
        let global_type = self.context.i8_type().array_type(size as _);
        let global = self.module.add_global(global_type, None, name);
        global.set_linkage(inkwell::module::Linkage::Private);
        global.set_alignment(std::mem::align_of::<T>() as _);
        let value = self.into_i8_array_value(value);
        global.set_initializer(&value);
        global
    }

    pub fn into_i8_array_value<T: Sized>(&self, value: T) -> inkwell::values::ArrayValue<'ctx> {
        // Rust can't compute the size of generic arguments in a `const` context yet:
        // https://github.com/rust-lang/rust/issues/43408.
        let size = std::mem::size_of::<T>();
        let bytes = {
            // Workaround for not being able to use `std::mem::transmute` here yet:
            // https://github.com/rust-lang/rust/issues/62875
            // https://github.com/rust-lang/rust/issues/61956
            let mut bytes = Vec::<i8>::new();
            bytes.resize(size, 0);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &value as *const _ as *const i8,
                    bytes.as_mut_ptr(),
                    size,
                )
            };
            std::mem::forget(value);
            bytes
        };

        let array = bytes
            .into_iter()
            .map(|byte| self.context.i8_type().const_int(byte as _, false))
            .collect::<Vec<_>>();
        self.context.i8_type().const_array(array.as_slice())
    }

    pub fn into_resolved_const_ref(
        &mut self,
        resolved: Resolved,
    ) -> inkwell::values::PointerValue<'ctx> {
        let index = match self.resolved_map.get(&resolved) {
            Some(index) => *index,
            None => {
                let index = self.resolveds.len();
                let name = format!("{:?}", resolved);
                let global = self.into_const(resolved.clone(), &name);
                self.resolved_map.insert(resolved, index);
                self.resolveds.push(global);
                index
            }
        };

        self.resolveds[index].as_pointer_value()
    }

    pub fn into_lookup_buf_const_ref(
        &mut self,
        lookup_buf: LookupBuf,
    ) -> inkwell::values::PointerValue<'ctx> {
        let index = match self.lookup_buf_map.get(&lookup_buf) {
            Some(index) => *index,
            None => {
                let index = self.lookup_bufs.len();
                let name = format!("{}", lookup_buf);
                let global = self.into_const(lookup_buf.clone(), &name);
                self.lookup_buf_map.insert(lookup_buf, index);
                self.lookup_bufs.push(global);
                index
            }
        };

        self.lookup_bufs[index].as_pointer_value()
    }

    pub fn build_alloca_resolved(&self, name: &str) -> inkwell::values::PointerValue<'ctx> {
        let resolved_type_identifier =
            "std::result::Result<vrl_compiler::Value, vrl_compiler::ExpressionError>";
        let resolved_type = self
            .module
            .get_struct_type(resolved_type_identifier)
            .unwrap_or_else(|| {
                panic!(
                    r#"failed getting type "{}" from module"#,
                    resolved_type_identifier
                )
            });

        self.builder.build_alloca(resolved_type, name)
    }
}
