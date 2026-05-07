

use monye_syntax::{
    parser::{
        Statement,
        Spanned,
        TypeName::{self, *},
    }
};
use crate::instruction::{
    Instruction,
    OpCode::*
};
use crate::translate::error::{
    TranslateError,
};
use crate::translate::lir::{
    Reg,
};
use crate::translate::env::{
    GlobalEnv,
    LocalEnv
};
use crate::translate::typing::{
    TypeNameExt
};
use crate::translate::compiler::expr::translate_expr;


pub(crate) fn translate_block(
    global_env: &mut GlobalEnv,
    local_env: Option<LocalEnv>,
    constants: &mut Vec<u64>,
    target_reg: Reg,
    block: &Spanned<Vec<Spanned<Statement>>>,
    expected_ty: Option<&TypeName>
) -> Result<(Vec<Instruction>, TypeName), TranslateError> {
    let mut result = Vec::new();
    let mut local_env = local_env.unwrap_or(LocalEnv::new());
    let mut last_expr_type_reg = None as Option<(TypeName, Reg)>;

    if block.node().is_empty() {
        return Ok((Vec::new(), TypeName::Unit))
    }

    for (is_last_statement, statement) in block.node()
        .iter().enumerate()
        .map(|(i, n)| (i == block.node().len() - 1, n))
    {
        match statement.node() {
            Statement::Bind {
                name: spanned_name,
                ty: spanned_ty,
                initializer: spanned_initializer, 
            } => {
                let name = spanned_name.node();
                let ty = spanned_ty.node();

                let reg = local_env.add_variable(name, ty);

                if let Some(spanned_expr) = spanned_initializer {
                    let (insn_seq, expr_type) = translate_expr(
                        global_env,
                        &local_env,
                        constants,
                        reg,
                        spanned_expr,
                        Some(ty)
                    )?;

                    let _ = match ty.try_cast(&expr_type) {
                        Ok(ty) => ty,
                        Err(e) => return Err(TranslateError(
                            e,
                            spanned_expr.span()
                        )),
                    };

                    result.extend(insn_seq);
                    last_expr_type_reg = None;
                }
                else {
                    ()
                }
            },
            Statement::Expression(spanned_expr) => {
                let target_reg = local_env.available_reg();
                let (insn_seq, expr_type) = translate_expr(
                    global_env,
                    &local_env,
                    constants,
                    target_reg,
                    spanned_expr,
                    if is_last_statement {
                        expected_ty
                    }
                    else {
                        None
                    }
                )?;

                result.extend(insn_seq);
                last_expr_type_reg = Some((expr_type, target_reg))
            }
            Statement::SemicolonnedExpr(spanned_expr) => {
                let target_reg = local_env.available_reg();
                let (insn_seq, _expr_type) = translate_expr(
                    global_env,
                    &local_env,
                    constants,
                    target_reg,
                    spanned_expr,
                    None
                )?;

                result.extend(insn_seq);
                last_expr_type_reg = Some((TypeName::Unit, target_reg))
            }
        }
    }

    if let Some((ty, reg)) = last_expr_type_reg {
        result.push(Instruction(
            Mov, target_reg.0, reg.0, 0
        ));
        Ok((result, ty))
    }
    else {
        Ok((result, Unit))
    }
}
