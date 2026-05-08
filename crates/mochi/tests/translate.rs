use monye_syntax::{
    lexer::{
        Lexer,
        StringLexer,
        PrimitiveType::*
    },
    parser::{
        parse,
        TypeName::*,
    }
};
use mochi::translate::{
    translate,
    Mochi,
    ErrorKind
};


fn compile(src: &str) -> Mochi {
    let mut lexer = StringLexer::new(src);
    let mut tokens = lexer.lex().unwrap();
    let ast = parse(&mut tokens).unwrap();
    translate(ast).unwrap()
}


fn compile_err(src: &str) -> mochi::translate::TranslateError {
    let mut lexer = StringLexer::new(src);
    let mut tokens = lexer.lex().unwrap();
    let ast = parse(&mut tokens).unwrap();
    translate(ast).unwrap_err()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_fn() {
        let srcs = [
            "fn main() {}",
            "fn main() { unit }",
        ];

        for src in srcs {
            let mochi = compile(src);
            
            let main = mochi.functions.iter().find(|f|f.name == "main").unwrap();
            assert_eq!(main.signature.ret_ty(), &Unit)
        }
    }


    #[test]
    fn missing_return_value() {
        let srcs = [
            r#"
            fn main() -> i32 {
                let x: i32 = 42;
            }"#,
            r#"
            fn main() -> i32 {
                unit
            }"#,
        ];

        for src in srcs {
            let err = compile_err(src);

            assert_eq!(
                err.kind(),
                &ErrorKind::MismatchedTypes(Primitive(I32), Unit)
            );
        }
    }

    #[test]
    fn if_same_types() {
        let srcs = [
            r#"
            fn main() -> i32 {
                if 1 > 0 {
                    -42
                }
                else {
                    53
                }
            }"#,
            r#"
            fn main() -> u32 {
                if 1 > 0 {
                    42
                }
                else {
                    53
                }
            }"#,
        ];

        for src in srcs {
            compile(src);
        }
    }

    #[test]
    fn if_mismatched_types() {
        let srcs = [
            r#"
            fn main() -> i32 {
                if 1 > 0 {
                    -42
                }
                else {
                    unit
                }
            }"#,
            r#"
            fn main() -> u32 {
                if 1 > 0 {
                    42
                }
                else {
                    let x: u64 = 53;
                    x
                }
            }"#,
        ];
        for src in srcs {
            let err = compile_err(src);

            assert!(matches!(
                err.kind(),
                ErrorKind::MismatchedTypes(_, _)
            ));
        }
    }


    #[test]
    fn invalid_argument_type() {
        let srcs = [
            r#"
            fn f(x: u32) {
                unit
            }
            fn main() {
                f(false)
            }"#,
            r#"
            fn f(cond: bool) {
                unit
            }
            fn main() {
                f(42)
            }"#,
        ];
        for src in srcs {
            let err = compile_err(src);

            assert!(matches!(
                err.kind(),
                ErrorKind::InvalidArgumentType(_, _)
            ));
        }
    }

    #[test]
    fn undefined_variable() {
        let srcs = [
            r#"
            fn main() {
                x
            }"#,
        ];

        for src in srcs {
            let err = compile_err(src);

            assert!(matches!(
                err.kind(),
                ErrorKind::UndefinedVariable(_)
            ));
        }
    }

    #[test]
    fn undefined_function() {
        let srcs = [
            r#"
            fn main() {
                hoge(x);
            }"#,
        ];

        for src in srcs {
            let err = compile_err(src);

            assert!(matches!(
                err.kind(),
                ErrorKind::UndefinedFunction(_)
            ));
        }
    }
}