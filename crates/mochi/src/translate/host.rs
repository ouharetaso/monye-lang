use std::sync::LazyLock;
use monye_syntax::{
    lexer::{
        PrimitiveType::*
    },
    parser::TypeName::*
};

use crate::translate::{
    Function,
    Signature,
    FuncId
};


pub static HOST_FUNCTIONS: LazyLock<Vec<Function>> = LazyLock::new(|| {
    let host_func_defs = vec![
        ("putc", Signature{params: vec![Primitive(U32)], ret_ty: Unit})
    ];

    host_func_defs.iter().enumerate()
        .map(|(i, (name, signature))|{
            Function::new(
                name,
                FuncId(i as u16),
                signature,
                Vec::new(),
                Vec::new()
            )
        })
        .collect()
});
