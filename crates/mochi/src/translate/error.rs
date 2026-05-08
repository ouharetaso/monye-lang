use monye_syntax::{
    lexer::{
        Span,
        PrimitiveType
    },
    parser::TypeName
};


#[derive(Debug, Clone, PartialEq)]
pub struct TranslateError(pub(crate) ErrorKind, pub(crate) Span);


impl TranslateError {
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    pub fn span(&self) -> Span {
        self.1
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    SyntaxError(SyntaxError),
    UndefinedVariable(String),
    MismatchedTypes(TypeName, TypeName),
    UndefinedFunction(String),
    InvalidArgumentCount(usize, usize),
    InvalidArgumentType(TypeName, TypeName),
    OperationUndefined(TypeName),
    CannotNegate(PrimitiveType),
    NothingReturned,
}


#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxError {
    InvalidAssignment
}


impl std::fmt::Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ErrorKind::SyntaxError(SyntaxError::InvalidAssignment) => {
                write!(f, "invalid assignment")
            },
            ErrorKind::UndefinedVariable(name) => {
                write!(f, "undefined variable \"{}\"", name)
            },
            ErrorKind::MismatchedTypes(expected, actual) => {
                write!(f, "mismatched types; expected {:?}, found {:?}", expected, actual)
            },
            ErrorKind::UndefinedFunction(name) => {
                write!(f, "undefined function \"{}\"", name)
            },
            ErrorKind::InvalidArgumentCount(expected, actual) => {
                write!(f, "invalid argument count; expected {}, found {}", expected, actual)
            },
            ErrorKind::InvalidArgumentType(lhs, rhs) => {
                write!(f, "invalid argument type; expected {:?}, found {:?}", lhs, rhs)
            },
            ErrorKind::NothingReturned => {
                write!(f, "nothing returned")
            },
            ErrorKind::OperationUndefined(ty) => {
                write!(f, "operation is undefined for {:?}", ty)
            }
            ErrorKind::CannotNegate(prim_ty) => {
                write!(f, "cannot negate type \"{:?}\"", prim_ty)
            }
        }
    }
}


impl std::error::Error for TranslateError {}
