use std::collections::HashMap;
use std::ops::Deref;
use monye_syntax::{
    parser::TypeName
};
use crate::translate::lir::{
    Signature,
    FuncId,
    Reg
};
use crate::translate::host::HOST_FUNCTIONS;



#[derive(Debug)]
pub(crate) struct GlobalEnv {
    func_defs: Vec<(String, Signature)>,
}


impl GlobalEnv {
    pub fn new() -> Self {
        let mut func_defs = Vec::new();

        for host_func in HOST_FUNCTIONS.deref() {
            func_defs.push((host_func.name.clone(), host_func.signature.clone()));
        }

        Self {
            func_defs,
        }
    }

    pub fn add_func(&mut self, name: &str, signature: &Signature) -> FuncId {
        let func_id = FuncId(self.func_defs.len() as u16);

        self.func_defs.push((name.to_string(), signature.clone()));

        func_id
    }

    pub fn get_func(&self, name: &str) -> Option<(&Signature, FuncId)> {
        self.func_defs.iter().enumerate()
            .find(|(_i, (func_name, _signature))|{
                func_name == name
            })
            .map(|(i, (_func_name, signature))| (signature, FuncId(i as u16)))
    }
}


#[derive(Debug, Clone)]
pub(crate) struct LocalEnv{
    variables: HashMap<String, (TypeName, Reg)>
}


impl LocalEnv {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new()
        }
    }

    pub fn add_variable(&mut self, name: &str, ty: &TypeName) -> Reg {
        let reg = self.available_reg();

        self.variables.insert(name.to_string(), (ty.clone(), reg));

        reg
    }

    pub fn get_variable(&self, name: &str) -> Option<(&TypeName, Reg)> {
        self.variables.get(name).map(|(ty, reg)| (ty, *reg))
    }

    pub fn available_reg(&self) -> Reg {
        (0..0xffffu16).into_iter()
            .map(|i| Reg(i))
            .find(|reg|{
                self.variables.values()
                    .all(|(_, allocated_reg)| allocated_reg != reg)
            })
            .unwrap_or(Reg(0))
    }
}
