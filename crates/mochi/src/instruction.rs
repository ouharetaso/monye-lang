use monye_syntax::lexer::PrimitiveType::{self, *};
use monye_syntax::parser::BinOp::{self, *};


#[repr(C)]
#[derive(Debug)]
pub struct Instruction(
    pub OpCode,
    pub u16,
    pub u16,
    pub u16
);

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Nop    = 0x0000,
    Const  = 0x0001,
    Mov    = 0x0002,
    FnCall = 0xF000,
    Ret    = 0xF001,

    AddU8  = 0x0100,
    AddI8  = 0x0101,
    AddU16 = 0x0102,
    AddI16 = 0x0103,
    AddU32 = 0x0104,
    AddI32 = 0x0105,
    AddU64 = 0x0106,
    AddI64 = 0x0107,
    SubU8  = 0x0108, 
    SubI8  = 0x0109,
    SubU16 = 0x010A, 
    SubI16 = 0x010B,
    SubU32 = 0x010C,
    SubI32 = 0x010D,
    SubU64 = 0x010E,
    SubI64 = 0x010F,
    MulU8  = 0x0110,
    MulI8  = 0x0111,
    MulU16 = 0x0112,
    MulI16 = 0x0113,
    MulU32 = 0x0114,
    MulI32 = 0x0115,
    MulU64 = 0x0116,
    MulI64 = 0x0117,
    DivU8  = 0x0118, 
    DivI8  = 0x0119,
    DivU16 = 0x011A, 
    DivI16 = 0x011B,
    DivU32 = 0x011C, 
    DivI32 = 0x011D,
    DivU64 = 0x011E,
    DivI64 = 0x011F,
    RemU8  = 0x0120,
    RemI8  = 0x0121,
    RemU16 = 0x0122,
    RemI16 = 0x0123,
    RemU32 = 0x0124,
    RemI32 = 0x0125,
    RemU64 = 0x0126,
    RemI64 = 0x0127,
    NegI8  = 0x0128,
    NegI16 = 0x0129,
    NegI32 = 0x012A,
    NegI64 = 0x012B,
    
}


impl Instruction {
    pub fn max_reg_index(&self) -> Option<u16> {
        match self.0 {
            OpCode::Nop => None,
            // 1 operand instruction
            OpCode::Const  | OpCode::Ret    => {
                Some(self.1)
            },
            // 2 operand instruction
            OpCode::NegI8  | OpCode::NegI16 | OpCode::NegI32 | OpCode::NegI64 |
            OpCode::Mov => {
                Some(self.1.max(self.2))
            },
            // 3 operand instruction
            OpCode::AddU8  | OpCode::AddI8  | OpCode::AddU16 | OpCode::AddI16 |
            OpCode::AddU32 | OpCode::AddI32 | OpCode::AddU64 | OpCode::AddI64 |
            OpCode::SubU8  | OpCode::SubI8  | OpCode::SubU16 | OpCode::SubI16 |
            OpCode::SubU32 | OpCode::SubI32 | OpCode::SubU64 | OpCode::SubI64 |
            OpCode::MulU8  | OpCode::MulI8  | OpCode::MulU16 | OpCode::MulI16 |
            OpCode::MulU32 | OpCode::MulI32 | OpCode::MulU64 | OpCode::MulI64 |
            OpCode::DivU8  | OpCode::DivI8  | OpCode::DivU16 | OpCode::DivI16 |
            OpCode::DivU32 | OpCode::DivI32 | OpCode::DivU64 | OpCode::DivI64 |
            OpCode::RemU8  | OpCode::RemI8  | OpCode::RemU16 | OpCode::RemI16 |
            OpCode::RemU32 | OpCode::RemI32 | OpCode::RemU64 | OpCode::RemI64
            => {
                Some(self.1.max(self.2).max(self.3))
            },
            // function call
            OpCode::FnCall => {
                Some(self.2 + self.3)
            },
        }
    }
}


pub trait BinOpExt {
    fn to_typed_op(&self, ty: PrimitiveType) -> Option<OpCode>;
}


impl BinOpExt for BinOp {
    fn to_typed_op(&self, ty: PrimitiveType) -> Option<OpCode> {
        match (self, ty) {
            (Add, U8)  => Some(OpCode::AddU8),
            (Add, I8)  => Some(OpCode::AddI8),
            (Add, U16) => Some(OpCode::AddU16),
            (Add, I16) => Some(OpCode::AddI16),
            (Add, U32) => Some(OpCode::AddU32),
            (Add, I32) => Some(OpCode::AddI32),
            (Add, U64) => Some(OpCode::AddU64),
            (Add, I64) => Some(OpCode::AddI64),
            (Sub, U8)  => Some(OpCode::SubU8),
            (Sub, I8)  => Some(OpCode::SubI8),
            (Sub, U16) => Some(OpCode::SubU16),
            (Sub, I16) => Some(OpCode::SubI16),
            (Sub, U32) => Some(OpCode::SubU32),
            (Sub, I32) => Some(OpCode::SubI32),
            (Sub, U64) => Some(OpCode::SubU64),
            (Sub, I64) => Some(OpCode::SubI64),
            (Mul, U8)  => Some(OpCode::MulU8),
            (Mul, I8)  => Some(OpCode::MulI8),
            (Mul, U16) => Some(OpCode::MulU16),
            (Mul, I16) => Some(OpCode::MulI16),
            (Mul, U32) => Some(OpCode::MulU32),
            (Mul, I32) => Some(OpCode::MulI32),
            (Mul, U64) => Some(OpCode::MulU64),
            (Mul, I64) => Some(OpCode::MulI64),
            (Div, U8)  => Some(OpCode::DivU8),
            (Div, I8)  => Some(OpCode::DivI8),
            (Div, U16) => Some(OpCode::DivU16),
            (Div, I16) => Some(OpCode::DivI16),
            (Div, U32) => Some(OpCode::DivU32),
            (Div, I32) => Some(OpCode::DivI32),
            (Div, U64) => Some(OpCode::DivU64),
            (Div, I64) => Some(OpCode::DivI64),
            (Rem, U8)  => Some(OpCode::RemU8),
            (Rem, I8)  => Some(OpCode::RemI8),
            (Rem, U16) => Some(OpCode::RemU16),
            (Rem, I16) => Some(OpCode::RemI16),
            (Rem, U32) => Some(OpCode::RemU32),
            (Rem, I32) => Some(OpCode::RemI32),
            (Rem, U64) => Some(OpCode::RemU64),
            (Rem, I64) => Some(OpCode::RemI64),
            _ => None,
        }
    }
}