use std::fmt;

use crate::value::VrlValueConvert;
use crate::{
    expression::{Block, Expr, Literal, Predicate, Resolved},
    vm::OpCode,
    Context, Expression, State, TypeDef, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub predicate: Predicate,
    pub consequent: Block,
    pub alternative: Option<Block>,
}

impl IfStatement {
    pub(crate) fn noop() -> Self {
        let literal = Literal::Boolean(false);
        let predicate = Predicate::new_unchecked(vec![Expr::Literal(literal)]);

        let literal = Literal::Null;
        let consequent = Block::new(vec![Expr::Literal(literal)]);

        Self {
            predicate,
            consequent,
            alternative: None,
        }
    }
}

impl Expression for IfStatement {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let predicate = self.predicate.resolve(ctx)?.try_boolean()?;

        match predicate {
            true => self.consequent.resolve(ctx),
            false => self
                .alternative
                .as_ref()
                .map(|block| block.resolve(ctx))
                .unwrap_or(Ok(Value::Null)),
        }
    }

    fn type_def(&self, state: &State) -> TypeDef {
        let type_def = self.consequent.type_def(state);

        match &self.alternative {
            None => type_def,
            Some(alternative) => type_def.merge_deep(alternative.type_def(state)),
        }
    }

    fn compile_to_vm(
        &self,
        vm: &mut crate::vm::Vm,
        state: &mut crate::state::Compiler,
    ) -> Result<(), String> {
        // Write the predicate which will leave the result on the stack.
        self.predicate.compile_to_vm(vm, state)?;

        // If the value is false, we want to jump to the alternative block.
        // We need to store this jump as it will need updating when we know where
        // the alternative block actually starts.
        let else_jump = vm.emit_jump(OpCode::JumpIfFalse);
        vm.write_opcode(OpCode::Pop);

        // Write the consequent block.
        self.consequent.compile_to_vm(vm, state)?;

        // After the consequent block we want to jump over the alternative.
        let continue_jump = vm.emit_jump(OpCode::Jump);

        // Update the initial if jump to jump to the current position.
        vm.patch_jump(else_jump);
        vm.write_opcode(OpCode::Pop);

        if let Some(alternative) = &self.alternative {
            // Write the alternative block.
            alternative.compile_to_vm(vm, state)?;
        } else {
            // No alternative resolves to Null.
            let null = vm.add_constant(Value::Null);
            vm.write_opcode(OpCode::Constant);
            vm.write_primitive(null);
        }

        // Update the continue jump to jump to the current position after the else block.
        vm.patch_jump(continue_jump);

        Ok(())
    }

    #[cfg(feature = "llvm")]
    fn emit_llvm<'ctx>(
        &self,
        state: &crate::state::Compiler,
        ctx: &mut crate::llvm::Context<'ctx>,
    ) -> Result<(), String> {
        let function = ctx.function();
        let if_statement_begin_block = ctx
            .context()
            .append_basic_block(function, "if_statement_begin");
        ctx.builder()
            .build_unconditional_branch(if_statement_begin_block);
        ctx.builder().position_at_end(if_statement_begin_block);

        self.predicate.emit_llvm(state, ctx)?;

        let is_bool = {
            let fn_ident = "vrl_resolved_is_boolean";
            let fn_impl = ctx
                .module()
                .get_function(fn_ident)
                .ok_or(format!(r#"failed to get "{}" function"#, fn_ident))?;
            ctx.builder()
                .build_call(fn_impl, &[ctx.result_ref().into()], fn_ident)
                .try_as_basic_value()
                .left()
                .ok_or(format!(r#"result of "{}" is not a basic value"#, fn_ident))?
                .try_into()
                .map_err(|_| format!(r#"result of "{}" is not an int value"#, fn_ident))?
        };

        let function = ctx.function();
        let is_boolean_block = ctx
            .context()
            .append_basic_block(function, "if_statement_predicate_is_boolean");
        let not_boolean_block = ctx
            .context()
            .append_basic_block(function, "if_statement_predicate_not_boolean");

        ctx.builder()
            .build_conditional_branch(is_bool, is_boolean_block, not_boolean_block);

        ctx.builder().position_at_end(is_boolean_block);

        let is_true = {
            let fn_ident = "vrl_resolved_boolean_is_true";
            let fn_impl = ctx
                .module()
                .get_function(fn_ident)
                .ok_or(format!(r#"failed to get "{}" function"#, fn_ident))?;
            ctx.builder()
                .build_call(fn_impl, &[ctx.result_ref().into()], fn_ident)
                .try_as_basic_value()
                .left()
                .ok_or(format!(r#"result of "{}" is not a basic value"#, fn_ident))?
                .try_into()
                .map_err(|_| format!(r#"result of "{}" is not an int value"#, fn_ident))?
        };

        let end_block = ctx
            .context()
            .append_basic_block(function, "if_statement_end");

        let if_branch_block = ctx
            .context()
            .append_basic_block(function, "if_statement_if_branch");
        let else_branch_block = ctx
            .context()
            .append_basic_block(function, "if_statement_else_branch");

        ctx.builder()
            .build_conditional_branch(is_true, if_branch_block, else_branch_block);

        ctx.builder().position_at_end(if_branch_block);
        self.consequent.emit_llvm(state, ctx)?;
        ctx.builder().build_unconditional_branch(end_block);

        ctx.builder().position_at_end(else_branch_block);
        if let Some(alternative) = &self.alternative {
            alternative.emit_llvm(state, ctx)?;
        }
        ctx.builder().build_unconditional_branch(end_block);

        ctx.builder().position_at_end(not_boolean_block);
        ctx.builder().build_unconditional_branch(end_block);

        ctx.builder().position_at_end(end_block);

        Ok(())
    }
}

impl fmt::Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("if ")?;
        self.predicate.fmt(f)?;
        f.write_str(" ")?;
        self.consequent.fmt(f)?;

        if let Some(alt) = &self.alternative {
            f.write_str(" else")?;
            alt.fmt(f)?;
        }

        Ok(())
    }
}
