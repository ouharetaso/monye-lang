

#[repr(C)]
#[derive(Debug)]
pub struct Instruction(
    pub OpCode,
    pub u16,
    pub u16,
    pub u16
);

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

    FnCall,
    Ret,
}


