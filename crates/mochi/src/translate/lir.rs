use monye_syntax::{
    parser::TypeName
};
use crate::instruction::*;
use crate::translate::host::HOST_FUNCTIONS;



#[derive(Debug)]
pub struct Mochi {
    pub functions: Vec<Function>,
    pub entry_point: String,
}


impl Mochi {
    pub fn new(functions: Vec<Function>) -> Self {
        let functions = HOST_FUNCTIONS.iter()
            .chain(functions.iter())
            .cloned()
            .collect();
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


#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub func_id: FuncId,
    pub signature: Signature,
    pub code: Vec<Instruction>,
    pub register_count: u16,
    pub constants: Vec<u64>,
}


impl Function {
    pub fn new(name: &str, func_id: FuncId, signature: &Signature, code: Vec<Instruction>, constants: Vec<u64>) -> Self {
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
    pub params: Vec<TypeName>,
    pub ret_ty: TypeName,
}


impl Signature {
    pub fn new(params: &Vec<TypeName>, ret_ty: &TypeName) -> Self {
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
