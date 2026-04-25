

#[repr(C)]
#[derive(Debug)]
pub struct Instruction {
    op: OpCode,
    a: u16,
    b: u16,
    c: u16,
}

#[repr(u16)]
#[derive(Debug)]
pub enum OpCode {
    Const,
    Mov,

    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Neg,

    Call,
    Ret,
}


