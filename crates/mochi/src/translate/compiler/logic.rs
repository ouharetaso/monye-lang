

use monye_syntax::{
    lexer::PrimitiveType::*,
    parser::{
        LogicalExpr,
        Spanned,
        TypeName::{self, *},
        LogicalOp::*,
    }
};
use crate::instruction::{
    Instruction,
    OpCode,
    LogicalOpExt,
};
use crate::translate::error::{
    TranslateError,
    ErrorKind,
};
use crate::translate::lir::{
    Reg,
};
use crate::translate::env::{
    GlobalEnv,
    LocalEnv
};
use crate::translate::typing::{
    DEFAULT_FALLBACK_TYPE,
    TypeNameExt
};
use crate::translate::compiler::expr::translate_expr;


pub fn translate_logic_expr(
    global_env: &mut GlobalEnv,
    local_env: &LocalEnv,
    constants: &mut Vec<u64>,
    target_reg: Reg,
    spanned_expr: &Spanned<LogicalExpr>,
) -> Result<(Vec<Instruction>, TypeName), TranslateError> {
    let expr = spanned_expr.node();
    let span = spanned_expr.span();

    match expr {
        LogicalExpr::Factor(expr) => {
            let (insn_seq, ty) = translate_expr(
                global_env,
                local_env,
                constants,
                target_reg,
                expr,
                None
            )?;

            Ok((insn_seq, ty))
        },
        LogicalExpr::LogicalOp {
            lhs,
            rhs,
            op
        } => {
            let mut result = Vec::new();
            let lhs_target_reg = target_reg;
            let rhs_target_reg = target_reg + 1;

            let (lhs_result, lhs_ty) = translate_logic_expr(
                global_env,
                local_env,
                constants,
                lhs_target_reg,
                lhs,
            )?;
            let (rhs_result, rhs_ty) = translate_logic_expr(
                global_env,
                local_env,
                constants,
                rhs_target_reg,
                rhs,
            )?;

            result.extend(lhs_result);
            result.extend(rhs_result);

            let target_ty = lhs_ty.try_cast(&rhs_ty)
                .map(|ty| 
                    if ty == Primitive(Integer) {
                        Primitive(DEFAULT_FALLBACK_TYPE)
                    }
                    else {
                        ty
                    }
                )
                .map_err(|e| TranslateError(e, span))?;

            // target_ty shouldn't be Primitive(Integer) because of the fallback above
            let result_ty = match (op, &target_ty) {
                (LogicalAnd, Primitive(Bool)) => {
                    result.extend(vec![
                        Instruction(OpCode::And, target_reg.0, target_reg.0, target_reg.0 + 1)
                    ]);
                    Primitive(Bool)
                },
                (LogicalOr, Primitive(Bool)) => {
                    result.extend(vec![
                        Instruction(OpCode::Or, target_reg.0, target_reg.0, target_reg.0 + 1)
                    ]);
                    Primitive(Bool)
                },
                (LogicalAnd | LogicalOr, ty @ _) => {
                    return Err(TranslateError(
                        ErrorKind::MismatchedTypes(Primitive(Bool), ty.clone()),
                        span
                    ))
                },
                (LT | LE, Primitive(prim_ty @ _)) => {
                    result.extend(vec![
                        Instruction(op.to_typed_op(prim_ty.clone()).unwrap(), target_reg.0, lhs_target_reg.0, rhs_target_reg.0)
                    ]);
                    Primitive(Bool)
                },
                (GT, Primitive(prim_ty @ _)) => {
                    result.extend(vec![
                        Instruction(LT.to_typed_op(prim_ty.clone()).unwrap(), target_reg.0, rhs_target_reg.0, lhs_target_reg.0)
                    ]);
                    Primitive(Bool)
                },
                (GE, Primitive(prim_ty @ _)) => {
                    result.extend(vec![
                        Instruction(LE.to_typed_op(prim_ty.clone()).unwrap(), target_reg.0, rhs_target_reg.0, lhs_target_reg.0)
                    ]);
                    Primitive(Bool)
                },
                (Equal, _) => {
                    result.extend(vec![
                        Instruction(OpCode::EQ, target_reg.0, lhs_target_reg.0, rhs_target_reg.0)
                    ]);
                    Primitive(Bool)
                },
                (NotEqual, _) => {
                    result.extend(vec![
                        Instruction(OpCode::NE, target_reg.0, lhs_target_reg.0, rhs_target_reg.0)
                    ]);
                    Primitive(Bool)
                },
                (_, ty @ _) => {
                    return Err(TranslateError(ErrorKind::OperationUndefined(ty.clone()), span))
                }
            };

            Ok((result, result_ty))
        }
    }
}