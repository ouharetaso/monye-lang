use std::ops::Add;

use mochi::instruction::{
    OpCode::*,
};
use mochi::translate::*;

pub enum RuntimeError {
    NoEntryPoint
}


pub fn run(mochi: &Mochi) -> Result<(), RuntimeError> {
    let entry_func = mochi.functions.iter()
        .find(|func| func.name == mochi.entry_point)
        .ok_or(RuntimeError::NoEntryPoint)?;




    todo!()
}


fn eval_func(mochi: &Mochi, func_id: FuncId, args: Vec<u64>) -> Result<(), RuntimeError> {
    let func = mochi.functions.iter().find(|f|f.func_id == func_id).unwrap();
    
    let mut registers = args;
    registers.resize(func.register_count as usize, 0);

    for insn in &func.code {
        let a = insn.1 as usize;
        let b = insn.2 as usize;
        let c = insn.3 as usize;
    }

    todo!()
}