use std::collections::HashMap;
use monye_syntax::{
    lexer::{
        PrimitiveType::self,
        Span
    },
    parser::{
        TypeName::{self, *},
        Spanned,
        Program,
        Declaration,
        Statement,
        Expression,
        BinOp,
        UniOp,
    }
};
use crate::instruction::{
    Instruction,
    OpCode::*
};


#[derive(Debug)]
pub struct Mochi {
    constants: Vec<u64>,
    functions: Vec<Function>,
    entry_point: String,
}


impl Mochi {
    fn new(constants: Vec<u64>, functions: Vec<Function>) -> Self {
        Self {
            constants,
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
struct Function {
    name: String,
    signature: Signature,
    code: Vec<Instruction>,
}


impl Function {
    fn new(name: &str, signature: &Signature, code: Vec<Instruction>) -> Self {
        Self {
            name: name.to_string(),
            signature: signature.clone(),
            code
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Signature {
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

    fn params(&self) -> &Vec<TypeName> {
        &self.params
    }

    fn ret_ty(&self) -> &TypeName {
        &self.ret_ty
    }
}


#[derive(Debug)]
struct GlobalEnv {
    func_defs: HashMap<String, (Signature, FuncId)>,
    consts: Vec<u64>,
}


impl GlobalEnv {
    fn new() -> Self {
        Self {
            func_defs: HashMap::new(),
            consts: Vec::new(),
        }
    }

    fn add_func(&mut self, name: &str, signature: &Signature) -> FuncId {
        let func_id = self.func_defs
            .values()
            .map(|v| v.1)
            .max()
            .map(|FuncId(i)| FuncId(i + 1))
            .unwrap_or(FuncId(0));

        self.func_defs.insert(name.to_string(), (signature.clone(), func_id));

        func_id        
    }

    fn get_func(&self, name: &str) -> Option<&(Signature, FuncId)> {
        self.func_defs.get(name)
    }

    fn add_const(&mut self, n: u64) -> u16 {
        if let Some(i) = self.consts.iter()
            .enumerate()
            .find(|(_i, x)| &n == *x)
            .map(|(i, _)| i)
        {
            i as u16
        }
        else {
            self.consts.push(n);
            (self.consts.len() - 1) as u16
        }
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

    fn get_reg(&self, name: &str) -> Option<Reg> {
        self.variables.get(name).map(|&(_, reg)| reg)
    }

    fn get_type(&self, name: &str) -> Option<&TypeName> {
        self.variables.get(name).map(|(ty, _)| ty)
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

    // register functions
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
                for (param, ty) in spanned_params {
                    local_env.add_variable(param.node(), ty.node());
                }

                let (insn_seq, ty) = translate_block(
                    &mut global_env,
                    Some(local_env),
                    spanned_body
                )?;

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

                let function = Function::new(
                    spanned_name.node(),
                    &signature,
                    insn_seq
                );
                functions.push(function);
            },
        }
    }
    
    Ok(Mochi::new(global_env.consts, functions))
}


fn translate_block(
    global_env: &mut GlobalEnv,
    local_env: Option<LocalEnv>,
    block: &Spanned<Vec<Spanned<Statement>>>
) -> Result<(Vec<Instruction>, TypeName), TranslateError> {
    let mut result = Vec::new();
    let mut local_env = local_env.unwrap_or(LocalEnv::new());
    let mut last_expr_type_reg = None as Option<(TypeName, Reg)>;

    for statement in block.node() {
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
                        reg,
                        spanned_expr
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
                    target_reg,
                    spanned_expr
                )?;

                result.extend(insn_seq);
                last_expr_type_reg = Some((expr_type, target_reg))
            }
        }
    }

    if let Some((ty, reg)) = last_expr_type_reg {
        result.push(Instruction(
            Ret, reg.0, 0, 0
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
    target_reg: Reg,
    spanned_expr: &Spanned<Expression>
) -> Result<(Vec<Instruction>, TypeName), TranslateError> {
    let expr = spanned_expr.node();
    let span = spanned_expr.span();

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
                target_reg,
                expr
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
                target_reg,
                lhs
            )?;
            let (rhs_result, rhs_type) = translate_expr(
                global_env,
                local_env,
                target_reg + 1,
                rhs
            )?;
            
            result.extend(lhs_result);
            result.extend(rhs_result);

            let op = match op {
                BinOp::Add => Add,
                BinOp::Sub => Sub,
                BinOp::Mul => Mul,
                BinOp::Div => Div,
                BinOp::Rem => Rem,
            };

            let expr_type = match lhs_type.try_cast(&rhs_type) {
                Ok(ty) => ty,
                Err(e) => return Err(TranslateError(
                    e,
                    span
                )),
            };

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
                .cloned()
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
                    dest_reg,
                    &param
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
            let const_index = global_env.add_const(*n);
            result.extend(vec![
                Instruction(Const, target_reg.0, const_index, 0)
            ]);

            Ok((result, TypeName::Primitive(PrimitiveType::Integer)))
        },
        Expression::UniOp {
            operand,
            op
        } => {
            let mut result = Vec::new();

            let (insn_seq, ty) = translate_expr(
                global_env,
                local_env,
                target_reg,
                operand
            )?;

            result.extend(insn_seq);

            let op = match op {
                UniOp::Neg => Neg
            };

            result.extend(vec![
                Instruction(op, target_reg.0, target_reg.0, 0)
            ]);

            Ok((result, ty))
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
        }
    }
}