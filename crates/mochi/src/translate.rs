use std::{collections::HashMap, vec};
use monye_syntax::{
    lexer::{
        PrimitiveType::{self, *},
        Span
    },
    parser::{
        Declaration,
        Expression,
        LogicalExpr,
        Program,
        Spanned,
        Statement,
        TypeName::{self, *},
        UniOp,
        LogicalOp::*
    }
};
use crate::instruction::{
    BinOpExt, Instruction, LogicalOpExt, OpCode::{self, *}
};


const DEFAULT_FALLBACK_TYPE: PrimitiveType = I32;


#[derive(Debug)]
pub struct Mochi {
    pub functions: Vec<Function>,
    pub entry_point: String,
}


impl Mochi {
    fn new(functions: Vec<Function>) -> Self {
        Self {
            functions,
            entry_point: "main".to_string()
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FuncId(pub u16);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reg(pub u16);


impl std::ops::Add<u16> for Reg {
    type Output = Reg;
    fn add(self, rhs: u16) -> Self::Output {
        Reg(self.0 + rhs)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConstId(pub u16);


#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub func_id: FuncId,
    pub signature: Signature,
    pub code: Vec<Instruction>,
    pub register_count: u16,
    pub constants: Vec<u64>,
}


impl Function {
    fn new(name: &str, func_id: FuncId, signature: &Signature, code: Vec<Instruction>, constants: Vec<u64>) -> Self {
        let max_reg_index = code.iter()
            .map(|insn| insn.max_reg_index().unwrap_or(0))
            .max()
            .unwrap_or(0u16);


        Self {
            name: name.to_string(),
            func_id,
            signature: signature.clone(),
            code,
            register_count: max_reg_index + 1,
            constants
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    params: Vec<TypeName>,
    ret_ty: TypeName,
}


impl Signature {
    fn new(params: &Vec<TypeName>, ret_ty: &TypeName) -> Self {
        Self {
            params: params.clone(),
            ret_ty: ret_ty.clone()
        }
    }

    pub fn params(&self) -> &Vec<TypeName> {
        &self.params
    }

    pub fn ret_ty(&self) -> &TypeName {
        &self.ret_ty
    }
}


#[derive(Debug)]
struct GlobalEnv {
    func_defs: Vec<(String, Signature)>,
}


impl GlobalEnv {
    fn new() -> Self {
        Self {
            func_defs: Vec::new(),
        }
    }

    fn add_func(&mut self, name: &str, signature: &Signature) -> FuncId {
        let func_id = FuncId(self.func_defs.len() as u16);

        self.func_defs.push((name.to_string(), signature.clone()));

        func_id
    }

    fn get_func(&self, name: &str) -> Option<(&Signature, FuncId)> {
        self.func_defs.iter().enumerate()
            .find(|(_i, (func_name, _signature))|{
                func_name == name
            })
            .map(|(i, (_func_name, signature))| (signature, FuncId(i as u16)))

    }
}


#[derive(Debug, Clone)]
struct LocalEnv{
    variables: HashMap<String, (TypeName, Reg)>
}


impl LocalEnv {
    fn new() -> Self {
        Self {
            variables: HashMap::new()
        }
    }

    fn add_variable(&mut self, name: &str, ty: &TypeName) -> Reg {
        let reg = self.available_reg();

        self.variables.insert(name.to_string(), (ty.clone(), reg));

        reg
    }

    fn get_variable(&self, name: &str) -> Option<(&TypeName, Reg)> {
        self.variables.get(name).map(|(ty, reg)| (ty, *reg))
    }

    fn available_reg(&self) -> Reg {
        (0..0xffffu16).into_iter()
            .map(|i| Reg(i))
            .find(|reg|{
                self.variables.values()
                    .all(|(_, allocated_reg)| allocated_reg != reg)
            })
            .unwrap_or(Reg(0))
    }
}


#[derive(Debug, Clone)]
pub struct TranslateError(ErrorKind, Span);


impl TranslateError {
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    pub fn span(&self) -> Span {
        self.1
    }
}


#[derive(Debug, Clone)]
pub enum ErrorKind {
    SyntaxError(SyntaxError),
    UndefinedVariable(String),
    MismatchedTypes(TypeName, TypeName),
    UndefinedFunction(String),
    InvalidArgumentCount(usize, usize),
    InvalidArgumentType(TypeName, TypeName),
    OperantionUndefined(TypeName),
    CannnotNegate(PrimitiveType),
    NothingReturned,
}


#[derive(Debug, Clone)]
pub enum SyntaxError {
    InvalidAssignment
}


impl std::fmt::Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ErrorKind::SyntaxError(SyntaxError::InvalidAssignment) => {
                write!(f, "invalid assignment")
            },
            ErrorKind::UndefinedVariable(name) => {
                write!(f, "undefined variable \"{}\"", name)
            },
            ErrorKind::MismatchedTypes(expected, actual) => {
                write!(f, "mismatched types; expected {:?}, found {:?}", expected, actual)
            },
            ErrorKind::UndefinedFunction(name) => {
                write!(f, "undefined function \"{}\"", name)
            },
            ErrorKind::InvalidArgumentCount(expected, actual) => {
                write!(f, "invalid argument count; expected {}, found {}", expected, actual)
            },
            ErrorKind::InvalidArgumentType(lhs, rhs) => {
                write!(f, "invalid argument type; expected {:?}, found {:?}", lhs, rhs)
            },
            ErrorKind::NothingReturned => {
                write!(f, "nothing returned")
            },
            ErrorKind::OperantionUndefined(ty) => {
                write!(f, "operation is undefined for {:?}", ty)
            }
            ErrorKind::CannnotNegate(prim_ty) => {
                write!(f, "cannot negate type \"{:?}\"", prim_ty)
            }
        }
    }
}


impl std::error::Error for TranslateError {}


trait TypeNameExt {
    fn try_cast(&self, other: &Self) -> Result<TypeName, ErrorKind>;
}


impl TypeNameExt for TypeName {
    #[allow(unreachable_patterns)]
    fn try_cast(&self, other: &Self) -> Result<Self, ErrorKind> {
        match (self, other) {
            (Primitive(lhs), Primitive(rhs)) => {
                lhs.try_cast(rhs)
                    .map(|pt| Primitive(pt)).
                    ok_or(ErrorKind::MismatchedTypes(
                        self.clone(),
                        other.clone()
                    ))
            },
            (_, _) => if self == other {
                Ok(self.clone())
            }
            else {
                Err(ErrorKind::MismatchedTypes(
                    self.clone(),
                    other.clone()
                ))
            }
        }
    }
}


pub fn translate(ast: Program) -> Result<Mochi, TranslateError> {
    let mut global_env = GlobalEnv::new();

    // register functions to global environment
    for decl in &ast.0 {
        match decl {
            Declaration::FnDecl {
                name: spanned_name,
                params: spanned_params,
                ret_ty: spanned_ret_ty,
                body: _ 
            } => {
                let name = spanned_name.node().clone();
                let signature = Signature::new(
                    &spanned_params.iter().map(|(_, ty)|{
                        ty.node().clone()
                    })
                    .collect(),
                    spanned_ret_ty.node()
                );
                global_env.add_func(&name, &signature);
            },
        }
    }

    let mut functions = Vec::new();

    for decl in &ast.0 {
        match decl {
            Declaration::FnDecl{
                name: spanned_name,
                params: spanned_params,
                ret_ty: spanned_ret_ty,
                body: spanned_body 
            } => {
                let mut local_env = LocalEnv::new();
                let mut constants = Vec::new();
                let target_reg = Reg(0);
                for (param, ty) in spanned_params {
                    local_env.add_variable(param.node(), ty.node());
                }

                let (mut insn_seq, ty) = translate_block(
                    &mut global_env,
                    Some(local_env),
                    &mut constants,
                    target_reg,
                    spanned_body,
                    Some(spanned_ret_ty.node())
                )?;

                insn_seq.extend(vec![
                    Instruction(Ret, target_reg.0, 0, 0)
                ]);

                match spanned_ret_ty.node().try_cast(&ty) {
                    Ok(_) => (),
                    Err(e) => return Err(TranslateError(
                        e,
                        spanned_ret_ty.span()
                    )),
                }

                let signature = Signature::new(
                    &spanned_params.iter().map(|(_, ty)|{
                        ty.node().clone()
                    })
                    .collect(),
                    spanned_ret_ty.node()
                );
                // 無いわけない
                let (_, func_id) = global_env.get_func(spanned_name.node()).unwrap();

                let function = Function::new(
                    spanned_name.node(),
                    func_id,
                    &signature,
                    insn_seq,
                    constants
                );
                functions.push(function);
            },
        }
    }
    
    Ok(Mochi::new(functions))
}


fn translate_block(
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
        }
    }

    if let Some((ty, reg)) = last_expr_type_reg {
        result.push(Instruction(
            Mov, target_reg.0, reg.0, 0
        ));
        Ok((result, ty))
    }
    else {
        Err(TranslateError(
            ErrorKind::NothingReturned,
            block.span()
        ))
    }
}


fn translate_expr(
    global_env: &mut GlobalEnv,
    local_env: &LocalEnv,
    constants: &mut Vec<u64>,
    target_reg: Reg,
    spanned_expr: &Spanned<Expression>,
    expected_ty: Option<&TypeName>
) -> Result<(Vec<Instruction>, TypeName), TranslateError> {
    let expr = spanned_expr.node();
    let span = spanned_expr.span();

    fn add_const(constants: &mut Vec<u64>, n: u64) -> u16 {
        if let Some(i) = constants.iter().enumerate()
            .find(|(_i, x)| *x == &n)
            .map(|(i, _x)| i)
        {
            i as u16
        }
        else {
            constants.push(n);
            (constants.len() - 1) as u16
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
                    ErrorKind::OperantionUndefined(expr_type),
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
                    Some(param_type)
                )?;
                match param_type.try_cast(&ty) {
                    Ok(_ty) => (),
                    Err(e) => return Err(TranslateError(
                        e,
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
            let const_index = add_const(constants, *n);
            result.extend(vec![
                Instruction(Const, target_reg.0, const_index, 0)
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
                        let const_index = add_const(constants, (-(*n as i64)) as u64);
                        result.extend(vec![
                            Instruction(Const, target_reg.0, const_index, 0)
                        ]);
                        return Ok((result, ty.clone()))
                    },
                    Primitive(prim_ty @ _) => {
                        return Err(TranslateError(
                            ErrorKind::CannnotNegate(*prim_ty),
                            span
                        ))
                    },
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
                    ErrorKind::OperantionUndefined(ty),
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
                    ErrorKind::CannnotNegate(ty),
                    span
                )),
                (_, _) => return Err(TranslateError(
                    ErrorKind::OperantionUndefined(Primitive(ty)),
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
            let const_index = add_const(constants, *b as u64);

            let ty = match expected_ty {
                Some(Primitive(Bool)) => Primitive(Bool),
                None => Primitive(Bool),
                Some(ty @ _) => return Err(TranslateError(
                    ErrorKind::MismatchedTypes(Primitive(Bool), ty.clone()),
                    span
                ))
            };
            
            result.extend(vec![
                Instruction(Const, target_reg.0, const_index, 0)
            ]);

            Ok((result, ty))
        },
        Expression::If(
            first,
            else_ifs,
            else_clause
        ) => {
            let mut result = Vec::<Instruction>::new();
            let mut ty = expected_ty.cloned();

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
                            (cond, cond_ty, body, body_ty, spanned_if.span())
                        })
                    })
                });

            let mut branches = main_branch.collect::<Result<Vec<_>, _>>()?;

            let else_branch = else_clause.as_ref().into_iter()
                .map(|body|{
                    let span = body.span();
                    let const_index = add_const(constants, 1);
                    let cond = vec![
                        Instruction(Const, target_reg.0, const_index, 0)
                    ];
                    let cond_ty = Primitive(Bool);

                    translate_block(
                        global_env,
                        Some(local_env.clone()),
                        constants,
                        target_reg,
                        &body,
                        expected_ty
                    )
                    .map(|(body, body_ty)|{
                        (cond, cond_ty, body, body_ty, span)
                    })
                });
            branches.extend(else_branch.collect::<Result<Vec<_>, _>>()?);
                
            for (cond, _cond_ty, body, body_ty, span) in branches {
                let offset: i32 = body.len() as i32;
                let b = (offset >> 16) as u16;
                let c = (offset & 0xffff) as u16;

                result.extend(cond);
                result.extend(vec![
                    Instruction(JumpNZ, target_reg.0, b, c)
                ]);
                result.extend(body);

                ty  = ty
                    .and_then(|ty|{
                        if ty != body_ty {
                            Some(Err(TranslateError(
                                ErrorKind::MismatchedTypes(ty.clone(), body_ty.clone()),
                                span
                            )))
                        }
                        else {
                            Some(Ok(ty))
                        }
                    })
                    .or_else(|| Some(Ok(body_ty)))
                    .transpose()?;
            }
            
            // 流石にどっかで型決定してるだろ
            Ok((result, ty.unwrap()))
        }
    }
}


fn translate_logic_expr(
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
                        Instruction(And, target_reg.0, target_reg.0, target_reg.0 + 1)
                    ]);
                    Primitive(Bool)
                },
                (LogicalOr, Primitive(Bool)) => {
                    result.extend(vec![
                        Instruction(Or, target_reg.0, target_reg.0, target_reg.0 + 1)
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
            };

            Ok((result, result_ty))
        }
    }
}