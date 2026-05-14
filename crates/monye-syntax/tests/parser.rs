use monye_syntax::{
    lexer::{
        Lexer,
        StringLexer,
    },
    parser::{
        parse,
        Program,
        ParseError,
    }
};


fn parse_success(src: &str) -> Program {
    let mut lexer = StringLexer::new(src);
    let mut tokens = lexer.lex().unwrap();
    parse(&mut tokens).unwrap()
}


fn parse_error(src: &str) -> ParseError {
    let mut lexer = StringLexer::new(src);
    let mut tokens = lexer.lex().unwrap();
    parse(&mut tokens).unwrap_err()
}


#[cfg(test)]
mod tests {
    use monye_syntax::parser::{
        BinOp,
        Declaration,
        Expression,
        IfExpr,
        LogicalExpr,
        LogicalOp,
        Spanned,
        Statement
    };
    use super::*;

    fn expect_expr(stmt: &Statement) -> &Expression {
        match stmt {
            Statement::Expression(Spanned(expr, _)) => expr,
            _ => panic!("expected expr")
        }
    }

    fn expect_semicolonned_expr(stmt: &Statement) -> &Expression {
        match stmt {
            Statement::SemicolonnedExpr(Spanned(expr, _)) => expr,
            _ => panic!("expected semicolonned expr")
        }
    }

    fn expect_if_expr<'a>(expr: &'a Expression) -> (&'a IfExpr, &'a Vec<Spanned<IfExpr>>, &'a Option<Spanned<Vec<Spanned<Statement>>>>) {
        match expr {
            Expression::If(
                if_clause,
                else_ifs,
                else_clause
            ) => {
                (if_clause.node(),else_ifs, else_clause)
            },
            _ => panic!("expected if expression")
        }
    }

    fn expect_binop<'a>(expr: &'a Expression, expected_op: BinOp) -> (&'a Expression, &'a Expression) {
        match expr {
            Expression::BinOp {
                lhs,
                rhs,
                op
            } => {
                assert_eq!(expected_op, *op);
                (lhs.node(), rhs.node())
            },
            _ => panic!("expected binary operation")
        }
    }

    fn expect_number(expr: &Expression, expected: u64) {
        match expr {
            Expression::Number(n) => {
                assert_eq!(expected, *n);
            }
            _ => panic!("expected number")
        }
    }

    fn expect_logicalop<'a>(cond: &'a LogicalExpr, expected_op: LogicalOp) -> (&'a LogicalExpr, &'a LogicalExpr) {
        match cond {
            LogicalExpr::LogicalOp {
                lhs,
                rhs,
                op
            } => {
                assert_eq!(expected_op, *op);
                (lhs.node(), rhs.node())
            },
            _ => panic!("expected logical operation")
        }
    }

    fn expect_logical_factor<'a>(cond: &'a LogicalExpr) -> &'a Expression {
        match cond {
            LogicalExpr::Factor(expr) => expr.node(),
            _ => panic!("expected factor of logical operation")
        }
    }

    #[test]
    fn parse_simple_program() {
        let _ = parse_success("fn main() {}");
    }

    #[test]
    fn parse_multiple_fn_defs() {
        let _ = parse_success(r#"
            fn hoge() {}
            fn fuga() {}
            fn main() {}
        "#);
    }

    #[test]
    fn parse_fn_def_with_arg() {
        let _ = parse_success(r#"
            fn none_returned() {}

            fn id(x: i32) -> i32 {
                x
            }

            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            fn multiply_add(a: i32, b: i32, c: i32) -> i32 {
                a * b + c
            }

            fn return_unit(a: i32, b: i32, c: i32, x: i32) -> unit {
                a * x * x + b * x + c;
            }
        "#);
    }

    #[test]
    fn parse_let() {
        let _ = parse_success(r#"
            fn main() {
                let a: i32 = 42;
                let x: i32;
            }
        "#);
    }

    #[test]
    fn parse_expr_with_semicolon() {
        let _ = parse_success(r#"
            fn return_unit(a: i32, b: i32, c: i32, x: i32) -> unit {
                a * x * x + b * x + c;
            }
        "#);
    }

    #[test]
    fn parse_operator_precedence() {
        let program = parse_success(r#"
            fn main() {
                1 + 2 * 3;
                (1 + 2) * 3;
            }
        "#);

        let main = &program.0[0];
        #[allow(irrefutable_let_patterns)]
        let Declaration::FnDecl{name: _, params: _, ret_ty: _, body: spanned_body } = main else {
            panic!()
        };

        // 1 + 2 * 3
        let statement1 = spanned_body.node()[0].node();
        let expr1 = expect_semicolonned_expr(statement1);
        let (lhs1, rhs1) = expect_binop(expr1, BinOp::Add);
        expect_number(lhs1, 1);
        let (mul_lhs, mul_rhs) = expect_binop(rhs1, BinOp::Mul);
        expect_number(mul_lhs, 2);
        expect_number(mul_rhs, 3);

        // (1 + 2) * 3
        let statement2 = spanned_body.node()[1].node();
        let expr2 = expect_semicolonned_expr(statement2);
        let (lhs2, rhs2) = expect_binop(expr2, BinOp::Mul);
        expect_number(rhs2, 3);
        let (add_lhs, add_rhs) = expect_binop(lhs2, BinOp::Add);
        expect_number(add_lhs, 1);
        expect_number(add_rhs, 2);
    }

    #[test]
    fn parse_assignment() {
        let _ = parse_success(r#"
            fn main() {
                let x: i32 = 42;
            }
        "#);
    }

    #[test]
    fn parse_call_with_value() {
        let _ = parse_success(r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            fn main() {
                let x: i32 = 42;
                add(x, 1);
            }
        "#);
    }

    #[test]
    fn parse_call_no_args() {
        let _ = parse_success(r#"
            fn do_nothing() {}

            fn main() {
                do_nothing()
            }
        "#);  
    }

    #[test]
    fn parse_if_no_else() {
        let _ = parse_success(r#"
            fn main() {
                let _: unit = if false {
                    5
                };
            }
        "#);  
    }

    #[test]
    fn parse_if_else() {
        let _ = parse_success(r#"
            fn max(a: i32, b: i32) -> i32 {
                if a > b {
                    a
                }
                else {
                    b
                }
            }

            fn main() {
                max(42, 5);
            }
        "#);  
    }

    #[test]
    fn parse_if_ifelse_else() {
        let _ = parse_success(r#"
            fn max3(a: i32, b: i32, c: i32) -> i32 {
                if a >= b && a >=c {
                    a
                }
                else if b >= a && b >= c{
                    b
                }
                else {
                    c
                }
            }

            fn main() {
                max3(42, 5, 81);
            }
        "#);  
    }

    #[test]
    fn parse_logical_expr_precedence() {
        let program = parse_success(r#"
            fn main() {
                if 1 >= 2 && 3 != 4 || 5 < 6 {}
            }
        "#);

        let main = &program.0[0];
        #[allow(irrefutable_let_patterns)]
        let Declaration::FnDecl{name: _, params: _, ret_ty: _, body: spanned_body } = main else {
            panic!()
        };

        let expr = expect_expr(spanned_body.node()[0].node());
        let (if_clause, _, _) = expect_if_expr(expr);
        let cond = if_clause.cond().node();

        // depth-first
        // lhs1 = "1 >= 2 && 3 != 4"
        // rhs1 = "5 < 6"
        let (lhs1, rhs1) = expect_logicalop(cond, LogicalOp::LogicalOr);
        // lhs2 = "1 >= 2"
        // rhs2 = "3 != 4"
        let (lhs2, rhs2) = expect_logicalop(lhs1, LogicalOp::LogicalAnd);
        // lhs3 = "1"
        // rhs3 = "2"
        let (lhs3, rhs3) = expect_logicalop(lhs2, LogicalOp::GE);
        expect_number(expect_logical_factor(lhs3), 1);
        expect_number(expect_logical_factor(rhs3), 2);
        // rhs2 = "3 != 4"
        // lhs4 = "3"
        // rhs4 = "4"
        let (lhs4, rhs4) = expect_logicalop(rhs2, LogicalOp::NotEqual);
        expect_number(expect_logical_factor(lhs4), 3);
        expect_number(expect_logical_factor(rhs4), 4);
        // rhs1 = "5 < 6"
        // lhs5 = "5"
        // rhs5 = "6"
        let (lhs5, rhs5) = expect_logicalop(rhs1, LogicalOp::LT);
        expect_number(expect_logical_factor(lhs5), 5);
        expect_number(expect_logical_factor(rhs5), 6);
    }

    #[test]
    fn parse_err_unexpected_token() {
        let srcs = [
            r#"fn main(,) {}"#,
            r#"fn main() { let : i32; }"#,
            r#"fn main( -> ) {}"#,
            r#"fn main() {;}"#,
            r#"fn main() { let let: unit; }"#,
        ];

        for src in srcs {
            let err = parse_error(src);
            assert!(matches!(err, ParseError::UnexpectedToken(_)))
        }
    }

    #[test]
    fn parse_err_unexpected_eof() {
        let srcs = [
            r#"fn main() {"#,
            r#"fn main() { let x: i32 ="#,
            r#"fn main"#,
            r#"fn main()"#,
            r#"fn main() ->"#,
        ];

        for src in srcs {
            let err = parse_error(src);
            assert_eq!(err, ParseError::UnexpectedEOF)
        }
    }

    #[test]
    fn parse_err_fn_def() {
        let srcs = [
            r#"fn main(a) {}"#,
            r#"fn main(a: i32, ) {}"#,
        ];

        for src in srcs {
            let err = parse_error(src);
            assert!(matches!(err, ParseError::UnexpectedToken(_)))
        }
    }

    #[test]
    fn parse_err_fncall() {
        let srcs = [
            r#"fn main() { f(, 1); }"#,
            r#"fn main() { f(1, ); }"#,
        ];

        for src in srcs {
            let err = parse_error(src);
            assert!(matches!(err, ParseError::UnexpectedToken(_)))
        }
    }

    #[test]
    fn parse_err_if_else() {
        let srcs = [
            r#"fn main() {
                if false {
                    42
                }
                else 1
            }"#,
            r#"fn main() {
                if false {
                    42
                }
                else {
                    3
                }
                else {
                    7
                }
            }"#,
        ];

        for src in srcs {
            let err = parse_error(src);
            assert!(matches!(err, ParseError::UnexpectedToken(_)))
        }
    }
}