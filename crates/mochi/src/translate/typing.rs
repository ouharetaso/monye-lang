use monye_syntax::{
    lexer::PrimitiveType::{self, *},
    parser::TypeName::{self, *},
};
use crate::translate::error::ErrorKind;


pub(crate) const DEFAULT_FALLBACK_TYPE: PrimitiveType = I32;

pub(crate) trait TypeNameExt {
    fn try_cast(&self, other: &Self) -> Result<TypeName, ErrorKind>;
}


impl TypeNameExt for TypeName {
    #[allow(unreachable_patterns)]
    fn try_cast(&self, other: &Self) -> Result<Self, ErrorKind> {
        match (self, other) {
            (expected, Never) => {
                Ok(expected.clone())
            },
            (Never, actual) => {
                Err(ErrorKind::MismatchedTypes(self.clone(), actual.clone()))
            },
            (Primitive(lhs), Primitive(rhs)) => {
                lhs.try_cast(rhs)
                    .map(|pt| Primitive(pt)).
                    ok_or(ErrorKind::MismatchedTypes(
                        self.clone(),
                        other.clone()
                    ))
            },
            (_, _) => if self == other {
                Ok(self.clone())
            }
            else {
                Err(ErrorKind::MismatchedTypes(
                    self.clone(),
                    other.clone()
                ))
            }
        }
    }
}
