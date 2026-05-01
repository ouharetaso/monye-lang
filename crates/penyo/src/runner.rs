use mochi::instruction::{
    OpCode::*,
};
use mochi::translate::*;

#[derive(Debug)]
pub enum RuntimeError {
    NoEntryPoint,
    PcExceeded,
    DivisionError(i64, i64),
}


impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivisionError(dividend, divisor) => write!(f, "divide error: {}, {}", dividend, divisor),
            Self::NoEntryPoint => write!(f, "no entry point"),
            Self::PcExceeded => write!(f, "PC exceeded"),
        }
    }
}


impl std::error::Error for RuntimeError {}


pub fn run(mochi: &Mochi) -> Result<(), RuntimeError> {
    let entry_func = mochi.functions.iter()
        .find(|func| func.name == mochi.entry_point)
        .ok_or(RuntimeError::NoEntryPoint)?;

    let args = Vec::new();

    let result = eval_func(mochi, entry_func.func_id, args)?;

    println!("{}", result);

    Ok(())
}


fn eval_func(mochi: &Mochi, func_id: FuncId, args: Vec<u64>) -> Result<u64, RuntimeError> {
    let func = &mochi.functions[func_id.0 as usize];
    let constants = &func.constants;
    
    let mut registers = args;
    registers.resize(func.register_count as usize, 0);

    let mut pc = 0usize;

    loop {
        let insn = &func.code.get(pc).ok_or(RuntimeError::PcExceeded)?;
        let opcode = insn.0;        
        let a = insn.1 as usize;
        let b = insn.2 as usize;
        let c = insn.3 as usize;

        match opcode {
            Nop    => (),
            Const  => registers[a] = constants[b],
            Mov    => registers[a] = registers[b],
            Ret    => return Ok(registers[a]),
            FnCall => {
                let func_id = FuncId(insn.1);
                let _dst = b;
                let argc = c;
                let args = registers[(b+1)..(b+1+argc)].to_vec();
                registers[b] = eval_func(mochi, func_id, args)?;
            },
            AddI8  | AddI16 | AddI32 | AddI64 |
            AddU8  | AddU16 | AddU32 | AddU64
            => {
                registers[a] = registers[b].wrapping_add(registers[c])
            },
            SubI8  | SubI16 | SubI32 | SubI64 |
            SubU8  | SubU16 | SubU32 | SubU64
            => {
                registers[a] = registers[b].wrapping_sub(registers[c])
            },
            MulI8  | MulI16 | MulI32 | MulI64 |
            MulU8  | MulU16 | MulU32 | MulU64
            => {
                registers[a] = registers[b].wrapping_mul(registers[c])
            },
            DivI8  => {
                let (dividend, divisor) = ((registers[b] as i8), (registers[c] as i8));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            DivU8  => {
                let (dividend, divisor) = ((registers[b] as u8), (registers[c] as u8));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            DivI16  => {
                let (dividend, divisor) = ((registers[b] as i16), (registers[c] as i16));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            DivU16 => {
                let (dividend, divisor) = ((registers[b] as u16), (registers[c] as u16));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            DivI32  => {
                let (dividend, divisor) = ((registers[b] as i32), (registers[c] as i32));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            DivU32 => {
                let (dividend, divisor) = ((registers[b] as u32), (registers[c] as u32));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            DivI64  => {
                let (dividend, divisor) = ((registers[b] as i64), (registers[c] as i64));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            DivU64 => {
                let (dividend, divisor) = ((registers[b] as u64), (registers[c] as u64));
                registers[a] = dividend.checked_div(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemI8  => {
                let (dividend, divisor) = ((registers[b] as i8), (registers[c] as i8));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemU8  => {
                let (dividend, divisor) = ((registers[b] as u8), (registers[c] as u8));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemI16  => {
                let (dividend, divisor) = ((registers[b] as i16), (registers[c] as i16));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemU16 => {
                let (dividend, divisor) = ((registers[b] as u16), (registers[c] as u16));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemI32  => {
                let (dividend, divisor) = ((registers[b] as i32), (registers[c] as i32));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemU32 => {
                let (dividend, divisor) = ((registers[b] as u32), (registers[c] as u32));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemI64  => {
                let (dividend, divisor) = ((registers[b] as i64), (registers[c] as i64));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            RemU64 => {
                let (dividend, divisor) = ((registers[b] as u64), (registers[c] as u64));
                registers[a] = dividend.checked_rem(divisor)
                    .ok_or(RuntimeError::DivisionError(dividend as i64, divisor as i64))? as u64;
            },
            NegI8  | NegI16 | NegI32 | NegI64 => {
                registers[a] = (!registers[b]) + 1
            },
            _ => unimplemented!()
        }

        pc += 1;
    }
}