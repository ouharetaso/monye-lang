

use monye_syntax::{
    lexer::PrimitiveType::*,
    parser::{
        Expression,
        Spanned,
        TypeName::{self, *},
        UniOp,
    }
};
use crate::instruction::{
    Instruction,
    OpCode::*,
    BinOpExt,
};
use crate::translate::error::{
    TranslateError,
    ErrorKind,
    SyntaxError
};
use crate::translate::lir::{
    ConstId,
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
use crate::translate::compiler::block::translate_block;
use crate::translate::compiler::logic::translate_logic_expr;


pub(crate) fn translate_expr(
    global_env: &mut GlobalEnv,
    local_env: &LocalEnv,
    constants: &mut Vec<u64>,
    target_reg: Reg,
    spanned_expr: &Spanned<Expression>,
    expected_ty: Option<&TypeName>
) -> Result<(Vec<Instruction>, TypeName), TranslateError> {
    let expr = spanned_expr.node();
    let span = spanned_expr.span();

    fn add_const(constants: &mut Vec<u64>, n: u64) -> ConstId {
        if let Some(i) = constants.iter().enumerate()
            .find(|(_i, x)| *x == &n)
            .map(|(i, _x)| i)
        {
            ConstId(i as u16)
        }
        else {
            constants.push(n);
            ConstId((constants.len() - 1) as u16)
        }
    }

    match expr {
        Expression::Assign {
            lhs,
            expr
        } => {
            let Expression::Value(assigned_to) = lhs.node() else {
                return Err(TranslateError(
                    ErrorKind::SyntaxError(SyntaxError::InvalidAssignment), 
                    lhs.span()
                ));
            };
            let (lhs_type, assigned_to) = local_env.get_variable(assigned_to)
                .ok_or(TranslateError(ErrorKind::UndefinedVariable(assigned_to.clone()), lhs.span()))?;
            let (mut result, ref rhs_type) = translate_expr(
                global_env,
                local_env,
                constants,
                target_reg,
                expr,
                Some(lhs_type)
            )?;

            let expr_type = match lhs_type.try_cast(&rhs_type) {
                Ok(ty) => ty,
                Err(e) => return Err(TranslateError(
                    e,
                    span
                )),
            };

            result.extend(vec![
                Instruction(Mov, assigned_to.0, target_reg.0, 0)
            ]);

            Ok((result, expr_type))
        }
        Expression::BinOp {
            lhs,
            rhs,
            op 
        } => {
            let mut result = Vec::new();
            let (lhs_result, lhs_type) = translate_expr(
                global_env,
                local_env,
                constants,
                target_reg,
                lhs,
                expected_ty
            )?;
            let (rhs_result, rhs_type) = translate_expr(
                global_env,
                local_env,
                constants,
                target_reg + 1,
                rhs,
                expected_ty
            )?;
            
            result.extend(lhs_result);
            result.extend(rhs_result);

            let expr_type = match lhs_type.try_cast(&rhs_type) {
                Ok(ty) => ty,
                Err(e) => return Err(TranslateError(
                    e,
                    span
                )),
            };

            #[allow(irrefutable_let_patterns)]
            let TypeName::Primitive(ty) = expr_type else {
                unreachable!(); // 今のところ
                #[allow(unreachable_code)]
                return Err(TranslateError(
                    ErrorKind::OperationUndefined(expr_type),
                    span
                ))
            };

            let ty = if ty == Integer {
                DEFAULT_FALLBACK_TYPE
            }
            else {
                ty
            };

            // 多分どの場合でも変換できる
            let op = op.to_typed_op(ty).unwrap();

            result.extend(vec![
                Instruction(op, target_reg.0, target_reg.0, target_reg.0 + 1)
            ]);

            Ok((result, expr_type))
        }
        Expression::FnCall {
            name,
            args
        } => {
            let mut result = Vec::new();
            let arg_base = target_reg + 1;
            let (signature, func_id) = global_env.get_func(name)
                .map(|(signature, func_id)| (signature.clone(), func_id))
                .ok_or(TranslateError(ErrorKind::UndefinedFunction(name.clone()), span))?;

            let argc = signature.params().len() as u16;
            let ret_ty = signature.ret_ty().clone();

            if signature.params().len() != args.len() {
                return Err(TranslateError(ErrorKind::InvalidArgumentCount(signature.params().len(), args.len()), span));
            }

            for (dest_reg, param_type, param) in signature
                .params().iter().zip(args)
                .enumerate()
                .map(|(i, remainder)| (arg_base + i as u16, remainder.0, remainder.1))
            {
                let (insn_seq, ty) = translate_expr(
                    global_env,
                    local_env,
                    constants,
                    dest_reg,
                    &param,
                    None
                )?;
                match param_type.try_cast(&ty) {
                    Ok(_ty) => (),
                    Err(_) => return Err(TranslateError(
                        ErrorKind::InvalidArgumentType(param_type.clone(), ty),
                        span
                    )),
                };
                result.extend(insn_seq);
            }

            result.extend(vec![
                Instruction(FnCall, func_id.0, target_reg.0, argc)
            ]);

            Ok((result, ret_ty))
        },
        Expression::Number(n) => {
            let mut result = Vec::new();
            let const_id = add_const(constants, *n);
            result.extend(vec![
                Instruction(Const, target_reg.0, const_id.0, 0)
            ]);

            Ok((
                result,
                if let Some(ty) = expected_ty {
                    ty.clone()
                }
                else {
                    TypeName::Primitive(Integer)
                }
            ))
        },
        Expression::UniOp {
            operand,
            op
        } => {
            let mut result = Vec::new();

            if let Expression::Number(n) = operand.node()
                && op == &UniOp::Neg
            {
                match expected_ty.unwrap_or(&Primitive(DEFAULT_FALLBACK_TYPE)) {
                    ty @ Primitive(I8 | I16 | I32 | I64 | Integer) => {
                        let const_id = add_const(constants, (-(*n as i64)) as u64);
                        result.extend(vec![
                            Instruction(Const, target_reg.0, const_id.0, 0)
                        ]);
                        return Ok((result, ty.clone()))
                    },
                    Primitive(prim_ty @ _) => {
                        return Err(TranslateError(
                            ErrorKind::CannotNegate(*prim_ty),
                            span
                        ))
                    },
                    ty @ (Unit | Never) => {
                        return Err(TranslateError(
                            ErrorKind::OperationUndefined(ty.clone()),
                            span
                        ))
                    }
                }
            }

            let (insn_seq, ty) = translate_expr(
                global_env,
                local_env,
                constants,
                target_reg,
                operand,
                expected_ty
            )?;

            result.extend(insn_seq);

            let ty = match (&ty, expected_ty) {
                (_, Some(expected)) => {
                    ty.try_cast(expected).map_err(|e|{
                        TranslateError(e, span)
                    })?
                },
                (t, None) => t.clone()
            };
            
            #[allow(irrefutable_let_patterns)]
            let TypeName::Primitive(ty) = ty else {
                unreachable!(); // 今のところ
                #[allow(unreachable_code)]
                return Err(TranslateError(
                    ErrorKind::OperationUndefined(ty),
                    span
                ))
            };

            let ty = if ty == Integer {
                DEFAULT_FALLBACK_TYPE
            }
            else {
                ty
            };

            let op = match (op, ty) {
                (UniOp::Neg, I8)  => NegI8,
                (UniOp::Neg, I16) => NegI16,
                (UniOp::Neg, I32) => NegI32,
                (UniOp::Neg, I64) => NegI64,
                (UniOp::Neg, U8| U16 | U32 | U64) => return Err(TranslateError(
                    ErrorKind::CannotNegate(ty),
                    span
                )),
                (_, _) => return Err(TranslateError(
                    ErrorKind::OperationUndefined(Primitive(ty)),
                    span
                ))
            };

            result.extend(vec![
                Instruction(op, target_reg.0, target_reg.0, 0)
            ]);

            Ok((result, Primitive(ty)))
        },
        Expression::Value(name) => {
            let mut result = Vec::new();

            let Some((ty, reg)) = local_env.get_variable(name) else {
                return Err(TranslateError(ErrorKind::UndefinedVariable(name.clone()), span))
            };

            result.extend(vec![
                Instruction(Mov, target_reg.0, reg.0, 0)
            ]);

            Ok((result, ty.clone()))
        },
        Expression::Bool(b) => {
            let mut result = Vec::new();
            let const_id = add_const(constants, *b as u64);

            let ty = match expected_ty {
                Some(Primitive(Bool)) => Primitive(Bool),
                None => Primitive(Bool),
                Some(ty @ _) => return Err(TranslateError(
                    ErrorKind::MismatchedTypes(Primitive(Bool), ty.clone()),
                    span
                ))
            };
            
            result.extend(vec![
                Instruction(Const, target_reg.0, const_id.0, 0)
            ]);

            Ok((result, ty))
        },
        Expression::Unit => {
            Ok((Vec::new(), TypeName::Unit))
        },
        Expression::If(
            first,
            else_ifs,
            else_clause
        ) => {
            let mut result = Vec::<Instruction>::new();
            let mut expr_ty = expected_ty.cloned();

            let main_branch = std::iter::once(first).chain(else_ifs)
                .map(|spanned_if|{
                    translate_logic_expr(
                        global_env,
                        local_env,
                        constants,
                        target_reg,
                        spanned_if.node().cond(),
                    )
                    .and_then(|(cond, cond_ty)|{
                        translate_block(
                            global_env,
                            Some(local_env.clone()),
                            constants,
                            target_reg,
                            spanned_if.node().body(), 
                            expected_ty
                        )
                        .map(|(body, body_ty)|{
                            (
                                cond,
                                cond_ty,
                                spanned_if.node().cond().span(),
                                body,
                                body_ty,
                                spanned_if.span()
                            )
                        })
                    })
                });

            let branches = main_branch.collect::<Result<Vec<_>, _>>()?;

            let else_branch = else_clause.clone()
                .map(|body|{
                    let span = body.span();
                    translate_block(
                        global_env,
                        Some(local_env.clone()),
                        constants,
                        target_reg,
                        &body,
                        expected_ty
                    )
                    .map(|(body, body_ty)|{
                        (body, body_ty, span)
                    })
                })
                .transpose()?;
                        
            let mut exit_jump_insn_indices = Vec::new();

            for (cond, cond_ty, cond_span, body, body_ty, span) in branches {
                if cond_ty != Primitive(Bool) {
                    return Err(TranslateError(ErrorKind::MismatchedTypes(Primitive(Bool), cond_ty), cond_span));
                }

                // + 1 because append exit jump instruction after the body
                let offset = (body.len() + 1) as i32;
                let b = (offset >> 16) as u16;
                let c = (offset & 0xffff) as u16;

                result.extend(cond);
                result.extend(vec![
                    Instruction(JumpZ, target_reg.0, b, c)
                ]);
                result.extend(body);

                // temporary instruction
                // to be rewirted to Jump instruction after this for-loop
                result.extend(vec![
                    Instruction(Nop, 0, 0, 0)
                ]);
                exit_jump_insn_indices.push(result.len() - 1);

                expr_ty  = match (expr_ty, body_ty) {
                    (None, Never) => Some(Never),
                    (None, ty) => Some(ty),

                    (Some(current), Never) => Some(current),
                    (Some(Never), ty) => Some(ty),
                    (Some(current), body_ty) => {
                        if current == body_ty {
                            Some(current)
                        }
                        else {
                            return Err(TranslateError(
                                ErrorKind::MismatchedTypes(current, body_ty),
                                span
                            ))
                        }
                    },
                }
            }

            if let Some((body, body_ty, span)) = else_branch {
                result.extend(body);

                expr_ty = match (expr_ty, body_ty) {
                    (Some(Never), ty) => Some(ty),
                    (Some(current), body_ty) => {
                        if current == body_ty {
                            Some(current)
                        }
                        else {
                            return Err(TranslateError(
                                ErrorKind::MismatchedTypes(current, body_ty),
                                span
                            ))
                        }
                    },
                    (None, _) => unreachable!()
                }
            }
            // there's no else clause
            else {
                // it cannot happen that expr_ty isn't determined even when reached else clause
                // when there's no else clause, type of main branch should be unit or never 
                if let Unit | Never = expr_ty.clone().unwrap() {
                    ()
                }
                else {
                    return Err(TranslateError(
                        ErrorKind::MismatchedTypes(Unit, expr_ty.unwrap()),
                        span
                    ));
                }
            }

            for index in exit_jump_insn_indices {
                let offset = (result.len() - index - 1) as i32;
                let b = (offset >> 16) as u16;
                let c = (offset & 0xffff) as u16;

                result[index] = Instruction(Jump, 0, b, c);
            }
            
            // 流石にどっかで型決定してるだろ
            Ok((result, expr_ty.unwrap()))
        }
    }
}
