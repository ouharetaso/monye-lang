mod error;
mod lir;
mod host;
mod env;
mod typing;
mod compiler;

pub use error::{
    TranslateError,
    ErrorKind,
    SyntaxError
};
pub use lir::{
    Mochi,
    Function,
    Signature,
    FuncId,
    Reg,
    ConstId
};
pub use host::HOST_FUNCTIONS;
pub use compiler::translate_unit::translate;