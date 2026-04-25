use std::collections::{HashMap, HashSet};
use monye_syntax::parser::*;
use crate::instruction::*;


#[derive(Debug)]
pub struct Mochi {
    constants: Vec<u64>,
    text: Vec<Instruction>
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncId(pub u16);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(pub u16);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstId(pub u16);


#[derive(Debug)]
struct Function {
    name: String,
    params: Vec<TypeName>,
    ret: TypeName,
    register_count: usize,
    code: Vec<Instruction>,
}


#[derive(Debug)]
struct GlobalEnv {}


#[derive(Debug, Clone)]
struct LocalEnv<'g>{
    global: &'g GlobalEnv,

    func_id: FuncId,
    
}


#[derive(Debug, Clone)]
pub enum TranslateError {
    SyntaxError(SyntaxError),
}


#[derive(Debug, Clone)]
pub enum SyntaxError {
    InvalidAssignment
}


fn translate(ast: Program) -> Result<Mochi, TranslateError> {
    for fn_decl in ast.0 {
        match fn_decl {
            Declaration::FnDecl{
                name: spanned_name,
                params: spanned_params,
                ret_ty: spanned_ret_ty,
                body: spanned_body 
            } => {
                let name = spanned_name.0;
                let params = spanned_params.iter()
                    .map(|(p, t)|{
                        (p.0.clone(), t.0.clone())
                    })
                    .collect::<Vec<_>>();
                let ret_ty = spanned_ret_ty.0;

            },
        }
    }
    todo!()
}