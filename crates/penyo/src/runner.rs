use std::io::{Write, BufWriter};
use mochi::instruction::{
    OpCode::*
};
use mochi::translate::*;

#[derive(Debug)]
pub enum RuntimeError {
    NoEntryPoint,
    PcExceeded,
    DivisionError(i64, i64),
    OutOfCharBounds(u32),
    WriteBufferFailed,
}


impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivisionError(dividend, divisor) => write!(f, "divide error: {}, {}", dividend, divisor),
            Self::NoEntryPoint => write!(f, "no entry point"),
            Self::PcExceeded => write!(f, "PC exceeded"),
            Self::OutOfCharBounds(n) => write!(f, "out out char bounds: 0x{:8x}", n),
            Self::WriteBufferFailed => write!(f, "failed writing buffer")
        }
    }
}


impl std::error::Error for RuntimeError {}


pub fn run(mochi: &Mochi) -> Result<u64, RuntimeError> {
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut writer = BufWriter::new(lock);

    run_with_writer(mochi, &mut writer)
}


pub fn run_with_writer<W: Write>(mochi: &Mochi, writer: &mut W) -> Result<u64, RuntimeError> {
    let entry_func = mochi.functions.iter()
        .find(|func| func.name == mochi.entry_point)
        .ok_or(RuntimeError::NoEntryPoint)?;

    let args = Vec::new();

    eval_func(mochi, entry_func.func_id, args, writer)
}


struct StackFrame<'f> {
    pub func: &'f Function,
    pub pc: usize,
    pub registers: Vec<u64>,
    pub dst: u16,
}


impl<'f> StackFrame<'f> {
    fn new(func: &'f Function, pc: usize, mut registers: Vec<u64>, dst: u16) -> Self {
        registers.resize(func.register_count as usize, 0);
        Self { func, pc, registers, dst }
    }
}


fn eval_func<W: Write>(
    mochi: &Mochi,
    func_id: FuncId,
    args: Vec<u64>,
    writer: &mut W
) -> Result<u64, RuntimeError> {
    let entry_func = &mochi.functions[func_id.0 as usize];
    let mut stack = Vec::new();

    stack.push(StackFrame::new(entry_func, 0, args, 0));

    loop {
        let frame = stack.last_mut().unwrap();
        let pc = &mut frame.pc;
        let func = frame.func;
        let insn = func.code.get(*pc).ok_or(RuntimeError::PcExceeded)?;
        let constants = &func.constants;
        #[allow(unused_mut)]
        let mut registers = &mut frame.registers;
        let opcode = insn.0;
        let a = insn.1 as usize;
        let b = insn.2 as usize;
        let c = insn.3 as usize;

        match opcode {
            Nop    => (),
            Const  => registers[a] = constants[b],
            Mov    => registers[a] = registers[b],
            Ret    => {
                let result = registers[a];
                let current_frame = stack.pop().unwrap();
                let dst = current_frame.dst;

                match stack.last_mut() {
                    None => return Ok(result),
                    Some(frame) => {
                        frame.registers[dst as usize] = result;
                    }
                }
                continue;
            }
            FnCall => {
                let func = &mochi.functions[a];

                if let Some(f) = HOST_FUNCTIONS.iter()
                    .find(|f|f.func_id.0 == a as u16)
                {
                    match f.func_id.0 {
                        0x0000 => {
                            writer.write_all(format!("{}", char::from_u32(registers[b+1] as u32)
                                .ok_or(RuntimeError::OutOfCharBounds(registers[b] as u32))?).as_bytes())
                                .or_else(|_| Err(RuntimeError::WriteBufferFailed))?
                        },
                        _ => unimplemented!("no such a host function")
                    }
                    *pc += 1;
                    continue;
                }

                let dest = b as u16;
                let argc = c;
                let args = registers[(b+1)..(b+1+argc)].to_vec();

                let frame = StackFrame::new(func, 0, args, dest);
                *pc += 1;
                stack.push(frame);
                continue;
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
            Jump => {
                let offset = ((b as i32) << 16) + c as i32;
                *pc = (*pc + 1).strict_add_signed(offset as isize);
                continue;
            },
            JumpZ => {
                let offset = ((b as i32) << 16) + c as i32;
                *pc = if registers[a] == 0 {
                    (*pc + 1).strict_add_signed(offset as isize)
                }
                else {
                    *pc + 1
                };
                continue;
            },
            JumpNZ => {
                let offset = ((b as i32) << 16) + c as i32;
                *pc = if registers[a] != 0 {
                    (*pc + 1).strict_add_signed(offset as isize)
                }
                else {
                    *pc + 1
                };
                continue;
            },
            EQ => {
                registers[a] = (registers[b] == registers[c]) as u64;
            },
            NE => {
                registers[a] = (registers[b] != registers[c]) as u64;
            },
            Inv => {
                registers[a] = !registers[b];
            },
            And => {
                registers[a] = registers[b] & registers[c];
            },
            Or => {
                registers[a] = registers[b] | registers[c];
            },
            Xor => {
                registers[a] = registers[b] ^ registers[c];
            },
            LTI8 => {
                registers[a] = ((registers[b] as i8)  < (registers[c] as i8))  as u64;
            },
            LTI16 => {
                registers[a] = ((registers[b] as i16) < (registers[c] as i16)) as u64;
            },
            LTI32 => {
                registers[a] = ((registers[b] as i32) < (registers[c] as i32)) as u64;
            },
            LTI64 => {
                registers[a] = ((registers[b] as i64) < (registers[c] as i64)) as u64;
            },
            LTU8 => {
                registers[a] = ((registers[b] as u8)  < (registers[c] as u8))  as u64;
            },
            LTU16 => {
                registers[a] = ((registers[b] as u16) < (registers[c] as u16)) as u64;
            },
            LTU32 => {
                registers[a] = ((registers[b] as u32) < (registers[c] as u32)) as u64;
            },
            LTU64 => {
                registers[a] = ((registers[b] as u64) < (registers[c] as u64)) as u64;
            },
            LEI8 => {
                registers[a] = ((registers[b] as i8)  <= (registers[c] as i8))  as u64;
            },
            LEI16 => {
                registers[a] = ((registers[b] as i16) <= (registers[c] as i16)) as u64;
            },
            LEI32 => {
                registers[a] = ((registers[b] as i32) <= (registers[c] as i32)) as u64;
            },
            LEI64 => {
                registers[a] = ((registers[b] as i64) <= (registers[c] as i64)) as u64;
            },
            LEU8 => {
                registers[a] = ((registers[b] as u8)  <= (registers[c] as u8))  as u64;
            },
            LEU16 => {
                registers[a] = ((registers[b] as u16) <= (registers[c] as u16)) as u64;
            },
            LEU32 => {
                registers[a] = ((registers[b] as u32) <= (registers[c] as u32)) as u64;
            },
            LEU64 => {
                registers[a] = ((registers[b] as u64) <= (registers[c] as u64)) as u64;
            },
        }

        *pc += 1;
    }
}