use std::collections::VecDeque;


use crate::lexer::{
    Token,
    TokenKind::{self, *},
    Span,
    PrimitiveType,
    Keyword::*
};


type Ident = String;


#[derive(Clone, Debug)]
pub struct Spanned<T: Clone>(pub T, pub Span);


impl<T: Clone> Spanned<T> {
    pub fn node(&self) -> &T {
        &self.0
    }

    pub fn span(&self) -> Span {
        self.1
    }
}


#[derive(Clone, Debug)]
pub struct Program(pub Vec<Declaration>);


#[derive(Clone, Debug)]
pub enum Declaration {
    FnDecl{
        name: Spanned<Ident>,
        params: Vec<(Spanned<Ident>, Spanned<TypeName>)>,
        ret_ty: Spanned<TypeName>,
        body: Spanned<Vec<Spanned<Statement>>>
    }
}


#[derive(Clone, Debug)]
pub enum Statement {
    Bind {
        name: Spanned<Ident>,
        ty: Spanned<TypeName>,
        initializer: Option<Spanned<Expression>>
    },
    SemicolonnedExpr(Spanned<Expression>),
    Expression(Spanned<Expression>),
}


#[derive(Clone, Debug)]
pub enum Expression {
    Assign {
        lhs: Box<Spanned<Expression>>,
        expr: Box<Spanned<Expression>>
    },
    BinOp {
        lhs: Box<Spanned<Expression>>,
        rhs: Box<Spanned<Expression>>,
        op: BinOp
    },
    UniOp {
        operand: Box<Spanned<Expression>>,
        op: UniOp
    },
    FnCall{
        name: Ident,
        args: Vec<Spanned<Expression>>
    },
    Number(u64),
    Value(Ident),
    Bool(bool),
    Unit,
    If(Spanned<IfExpr>, Vec<Spanned<IfExpr>>, Option<Spanned<Vec<Spanned<Statement>>>>)
}


#[derive(Clone, Debug)]
pub struct IfExpr(pub Spanned<LogicalExpr>, pub Spanned<Vec<Spanned<Statement>>>);


impl IfExpr {
    pub fn cond(&self) -> &Spanned<LogicalExpr> {
        &self.0
    }

    pub fn body(&self) -> &Spanned<Vec<Spanned<Statement>>> {
        &self.1
    }
}


#[derive(Clone, Debug)]
pub enum LogicalExpr {
    Factor(Box<Spanned<Expression>>),
    LogicalOp {
        lhs: Box<Spanned<LogicalExpr>>,
        rhs: Box<Spanned<LogicalExpr>>,
        op: LogicalOp
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum LogicalOp {
    LogicalOr,
    LogicalAnd,
    Equal,
    NotEqual,
    LT,
    GT,
    LE,
    GE,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeName {
    Primitive(PrimitiveType),
    Unit,
    Never,
    //UserDefined(Ident),
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Or,
    And,
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UniOp {
    Neg,
    Inv
}


#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Span),
    UnexpectedEOF
}


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(span) => write!(f, "unexpected token at : {}-{}", span.start(), span.end()),
            ParseError::UnexpectedEOF => write!(f, "unexpected end of file"),
        }
    }
}


impl std::error::Error for ParseError {}


fn unexpected_token<T>(token: Token) -> Result<T, ParseError> {
    if token.kind() == EOF {
        Err(ParseError::UnexpectedEOF)
    }
    else {
        Err(ParseError::UnexpectedToken(token.span()))
    }
}


fn consume(tokens: &mut VecDeque<Token>, expect: TokenKind) -> Result<Span, ParseError> {
    if let Some(token @ Token(_, _)) = tokens.pop_front()  {
        if &expect == token.kind() {
            Ok(token.span())
        }
        else {
            unexpected_token(token)
        }
    }
    else {
        Err(ParseError::UnexpectedEOF)
    }
}


fn next(tokens: &mut VecDeque<Token>) -> Result<Token, ParseError> {
    tokens.pop_front().ok_or(ParseError::UnexpectedEOF)
}


fn peek(tokens: &VecDeque<Token>) -> Result<Token, ParseError> {
    tokens.get(0).cloned().ok_or(ParseError::UnexpectedEOF)
}


pub fn parse(tokens: &mut VecDeque<Token>) -> Result<Program, ParseError> {
    let mut program = Vec::new();
    loop {
        program.push(fn_decl(tokens)?);
        if peek(tokens)?.kind() == EOF {
            break;
        }
    }
    consume(tokens, EOF)?;

    Ok(Program(program))
}


fn fn_decl(tokens: &mut VecDeque<Token>) -> Result<Declaration, ParseError> {
    let Span(start, _) = consume(tokens, Keyword(Fn))?;
    let name = match next(tokens)? {
        Token(Identifier(name), span) => Spanned(name, span),
        token @ _ => return unexpected_token(token)
    };
    consume(tokens, LParen)?;
    let params = param_list(tokens)?;
    let Span(_, end) = consume(tokens, RParen)?;

    let ret_ty = if peek(tokens)?.kind() == Arrow {
        consume(tokens, Arrow)?;
        type_name(tokens)?
    }
    else {
        Spanned(TypeName::Unit, Span(start, end))
    };
    let body = block(tokens)?;
    
    Ok(Declaration::FnDecl{
        name,
        params,
        ret_ty,
        body
    })
}


fn param_list(tokens: &mut VecDeque<Token>) -> Result<Vec<(Spanned<Ident>, Spanned<TypeName>)>, ParseError> {
    let mut result = Vec::new();
    
    let Token(kind, _) = peek(tokens)?;
    if kind == Comma {
        return unexpected_token(peek(tokens)?)
    }

    match peek(tokens)? {
        Token(Identifier(_), _) => {
            result.push(typed_binding(tokens)?);
        },
        Token(RParen, _) => return Ok(result),
        token @ _ => return unexpected_token(token)
    }

    loop {
        match peek(tokens)? {
            Token(Comma, _) => consume(tokens, Comma)?,
            Token(RParen, _) => break,
            token @ _ => return unexpected_token(token)
        };
        match peek(tokens)? {
            Token(Identifier(_), _) => {
                result.push(typed_binding(tokens)?);
            },
            token @ _ => return unexpected_token(token)
        };
    }
    Ok(result)
}


fn typed_binding(tokens: &mut VecDeque<Token>) -> Result<(Spanned<Ident>, Spanned<TypeName>), ParseError> {
    let token = next(tokens)?;
    let Token(kind, span) = token.clone();

    let Identifier(name) = kind else {
        return unexpected_token(token)
    };

    consume(tokens, Colon)?;
    let ty = type_name(tokens)?;
    Ok((Spanned(name, span), ty))
}


fn type_name(tokens: &mut VecDeque<Token>) -> Result<Spanned<TypeName>, ParseError> {
    match next(tokens)? {
        Token(Type(ty), span) => Ok(Spanned(TypeName::Primitive(ty), span)),
        Token(Keyword(Unit), span) => Ok(Spanned(TypeName::Unit, span)),
        Token(Keyword(Never), span) => Ok(Spanned(TypeName::Never, span)),
        //Identifier(ident) => Ok(TypeName::UserDefined(ident)),
        token @ _ => return unexpected_token(token),
    }
}


fn block(tokens: &mut VecDeque<Token>) -> Result<Spanned<Vec<Spanned<Statement>>>, ParseError> {
    let mut result = Vec::new();

    let start = peek(tokens)?.span().start();
    consume(tokens, LBrace)?;

    loop {
        match peek(tokens)? {
            Token(
                Keyword(Let) | LParen | Minus | Identifier(_) | Number(_) | Keyword(If) |
                Keyword(True) | Keyword(False) | Keyword(Unit),
                _span
            ) => {
                result.push(statement(tokens)?);
                continue;
            },
            Token(RBrace, _span) => break,
            token @ _ => return unexpected_token(token)
        }
    }
    let end = peek(tokens)?.span().end();
    consume(tokens, RBrace)?;

    Ok(Spanned(result, Span(start, end)))
}


fn statement(tokens: &mut VecDeque<Token>) -> Result<Spanned<Statement>, ParseError> {
    match peek(tokens)? {
        Token(Keyword(Let), _) => {
            let bind = bind(tokens)?;
            consume(tokens, Semicolon)?;
            Ok(bind)
        },
        Token(
            LParen | Minus | Identifier(_) | Number(_) | Keyword(If) |
            Keyword(True) | Keyword(False) | Keyword(Unit),
            span_start
        ) => {
            let spanned_expr = assign(tokens)?;
            let end = spanned_expr.span().end();

            if peek(tokens)?.kind() == Semicolon {
                let end = consume(tokens, Semicolon)?.end();
                Ok(Spanned(
                    Statement::SemicolonnedExpr(spanned_expr),
                    Span(span_start.start(), end)
                ))
            }
            else {
                Ok(Spanned(Statement::Expression(spanned_expr), Span(span_start.start(), end)))
            }
        },
        token @ _ => return unexpected_token(token)
    }
}


fn bind(tokens: &mut VecDeque<Token>) -> Result<Spanned<Statement>, ParseError> {
    let Span(start, _) = consume(tokens, Keyword(Let))?;
    let (name, ty) = typed_binding(tokens)?;
    let initializer = if peek(tokens)?.kind() == Equal {
        consume(tokens, Equal)?;
        Some(expr(tokens)?)
    }
    else {
        None
    };

    let end = if let Some(Spanned(_, ref span)) = initializer {
        span.end()
    }
    else {
        ty.span().end()
    };

    Ok(Spanned(
        Statement::Bind {
            name,
            ty,
            initializer
        },
        Span(start, end)
    ))
}


fn expr(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    assign(tokens)
}


fn assign(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    let spanned_lhs = operation(tokens)?;
    let Span(start, _) = spanned_lhs.span();

    if peek(tokens)?.kind() == Equal {
        consume(tokens, Equal)?;
        let spanned_expr = expr(tokens)?;
        let Span(_, end) = spanned_expr.span();
        Ok(Spanned(
            Expression::Assign {
                lhs: Box::new(spanned_lhs),
                expr: Box::new(spanned_expr)
            },
            Span(start, end)
        ))
    }
    else {
        Ok(spanned_lhs)
    }
}


fn operation(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    addition(tokens)
}


#[allow(unused_assignments)]
fn addition(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    let mut lhs = multiply(tokens)?;
    let Span(start, mut end) = lhs.span();

    while let Token(Plus | Minus, _) = peek(tokens)? {
        match peek(tokens)? {
            Token(Plus, _) => {
                consume(tokens, Plus)?;
                let spanned_rhs = multiply(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::Add
                    },
                    Span(start, end)
                );
            },
            Token(Minus, _) => {
                consume(tokens, Minus)?;
                let spanned_rhs = multiply(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::Sub
                    },
                    Span(start, end)
                );
            },
            _ => unreachable!()
        }
    }
    Ok(lhs)
}

#[allow(unused_assignments)]
fn multiply(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    let mut lhs = unary_op(tokens)?;
    let Span(start, mut end) = lhs.span();

    while let Token(Asterisk | Slash | Percent, _) = peek(tokens)? {
        match peek(tokens)? {
            Token(Asterisk, _) => {
                consume(tokens, Asterisk)?;
                let spanned_rhs = unary_op(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::Mul
                    },
                    Span(start, end)
                );
            },
            Token(Slash, _) => {
                consume(tokens, Slash)?;
                let spanned_rhs = unary_op(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::Div
                    },
                    Span(start, end)
                );
            },
            Token(Percent, _) => {
                consume(tokens, Percent)?;
                let spanned_rhs = unary_op(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::Rem
                    },
                    Span(start, end)
                );
            },
            _ => unreachable!()
        }
    }
    Ok(lhs)
}


fn unary_op(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    match peek(tokens)?.kind() {
        Minus => {
            let Span(start, _) = consume(tokens, Minus)?;
            let spanned_operand = factor(tokens)?;
            let end = spanned_operand.span().end();

            Ok(Spanned(
                Expression::UniOp{
                    operand: Box::new(spanned_operand),
                    op: UniOp::Neg
                },
                Span(start, end)
            ))
        },
        Exclamation => {
            let Span(start, _) = consume(tokens, Exclamation)?;
            let spanned_operand = factor(tokens)?;
            let end = spanned_operand.span().end();

            Ok(Spanned(
                Expression::UniOp{
                    operand: Box::new(spanned_operand),
                    op: UniOp::Inv
                },
                Span(start, end)
            ))
        },
        _ => factor(tokens)
    }
}


fn factor(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    match peek(tokens)? {
        Token(Number(n), _) => {
            let Token(_, span) = next(tokens)?;
            Ok(Spanned(
                Expression::Number(n),
                span
            ))
        },
        Token(LParen, _) => {
            let Span(start, _) = consume(tokens, LParen)?;
            let Spanned(expr, _) = expr(tokens)?;
            let Span(_, end) = consume(tokens, RParen)?;
            Ok(Spanned(
                expr,
                Span(start, end)
            ))
        }
        Token(Identifier(_), _) => fn_call(tokens),
        Token(Keyword(b @ (True | False)), _) => {
            let Token(_, span) = next(tokens)?;
            Ok(Spanned(
                Expression::Bool(b == True),
                span
            ))
        },
        Token(Keyword(Unit), span) => {
            consume(tokens, Keyword(Unit))?;
            Ok(Spanned(
                Expression::Unit,
                span
            ))
        }
        Token(Keyword(If), _) => if_expr(tokens),
        token @ _ => return unexpected_token(token)
    }
}


fn fn_call(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    let Token(kind, span) = next(tokens)?;
    let Identifier(ident) = kind else {
        return Err(ParseError::UnexpectedToken(span))
    };

    if peek(tokens)?.kind() == LParen {
        let start = span.start();
        let mut args = Vec::new();
        consume(tokens, LParen)?;
        if peek(tokens)?.kind() == Comma {
            return unexpected_token(next(tokens)?);
        }

        match peek(tokens)? {
            Token(RParen, _) => (),
            Token(
                LParen | Number(_) | Identifier(_) | Minus | Keyword(If) |
                Keyword(True| False | Unit),
                _
            ) => {
                args.push(expr(tokens)?);
            },
            token @ _ => return unexpected_token(token)
        }

        loop {
            match peek(tokens)? {
                Token(RParen, _) => break,
                Token(Comma, _) => {
                    consume(tokens, Comma)?;
                    args.push(expr(tokens)?);
                    continue;
                }
                token @ _ => return unexpected_token(token)
            }
        }
        let Span(_, end) = consume(tokens, RParen)?;
        Ok(Spanned(
            Expression::FnCall {
                name: ident,
                args
            },
            Span(start, end)
        ))
    }
    else {
        Ok(Spanned(
            Expression::Value(ident),
            span
        ))
    }
}


#[allow(unused_assignments)]
fn if_expr(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    let start = peek(tokens)?.span().start();
    consume(tokens, Keyword(If))?;
    let cond = logic_expr(tokens)?;
    let body = block(tokens)?;
    let mut end = body.span().end();
    let first = Spanned(IfExpr(cond, body), Span(start, end));

    let mut else_ifs = Vec::new();
    let mut else_clause = None;

    while &Keyword(Else) == peek(tokens)?.kind() {
        consume(tokens, Keyword(Else))?;
        match peek(tokens)?.kind() {
            Keyword(If) => {
                let start = consume(tokens, Keyword(If))?.start();
                let cond = logic_expr(tokens)?;
                let body = block(tokens)?;
                end = body.span().end();
                else_ifs.push(Spanned(IfExpr(cond, body), Span(start, end)));
            },
            LBrace => {
                let body = block(tokens)?;
                end = body.span().end();
                else_clause = Some(body);
                break;
            },
            _ => return unexpected_token(next(tokens)?)
        }
    }

    Ok(Spanned(
        Expression::If(
            first,
            else_ifs,
            else_clause
        ),
        Span(start, end)
    ))
}


fn logic_expr(tokens: &mut VecDeque<Token>) -> Result<Spanned<LogicalExpr>, ParseError> {
    logical_or(tokens)
}


#[allow(unused_assignments)]
fn logical_or(tokens: &mut VecDeque<Token>) -> Result<Spanned<LogicalExpr>, ParseError> {
    let mut lhs = logical_and(tokens)?;
    let Span(start, mut end) = lhs.span();

    while &DoubleVbar == peek(tokens)?.kind() {
        consume(tokens, DoubleVbar)?;
        let spanned_rhs = logical_and(tokens)?;
        end = spanned_rhs.span().end();
        lhs = Spanned(
            LogicalExpr::LogicalOp {
                lhs: Box::new(lhs),
                rhs: Box::new(spanned_rhs),
                op: LogicalOp::LogicalOr
            },
            Span(start, end)
        )
    }
    Ok(lhs)
}


#[allow(unused_assignments)]
fn logical_and(tokens: &mut VecDeque<Token>) -> Result<Spanned<LogicalExpr>, ParseError> {
    let mut lhs = equality(tokens)?;
    let Span(start, mut end) = lhs.span();

    while &DoubleAmpersand == peek(tokens)?.kind() {
        consume(tokens, DoubleAmpersand)?;
        let spanned_rhs = equality(tokens)?;
        end = spanned_rhs.span().end();
        lhs = Spanned(
            LogicalExpr::LogicalOp {
                lhs: Box::new(lhs),
                rhs: Box::new(spanned_rhs),
                op: LogicalOp::LogicalAnd
            },
            Span(start, end)
        )
    }
    Ok(lhs)
}


#[allow(unused_assignments)]
fn equality(tokens: &mut VecDeque<Token>) -> Result<Spanned<LogicalExpr>, ParseError> {
    let mut lhs = order(tokens)?;
    let Span(start, mut end) = lhs.span();

    while let DoubleEqual | NE = peek(tokens)?.kind() {
        match next(tokens)?.kind() {
            DoubleEqual => {
                let spanned_rhs = order(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    LogicalExpr::LogicalOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: LogicalOp::Equal
                    },
                    Span(start, end)
                )
            },
            NE => {
                let spanned_rhs = order(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    LogicalExpr::LogicalOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: LogicalOp::NotEqual
                    },
                    Span(start, end)
                )
            },
            _ => unreachable!()
        }
    }
    Ok(lhs)
}


#[allow(unused_assignments)]
fn order(tokens: &mut VecDeque<Token>) -> Result<Spanned<LogicalExpr>, ParseError> {
    let spanned_expr = expr(tokens)?;
    let Span(start, mut end) = spanned_expr.span();
    let mut lhs = Spanned(LogicalExpr::Factor(Box::new(spanned_expr)), Span(start, end));

    while let LT | GT | LE | GE = peek(tokens)?.kind() {
        match next(tokens)?.kind() {
            LT => {
                let spanned_rhs = expr(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    LogicalExpr::LogicalOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(Spanned(
                            LogicalExpr::Factor(
                                Box::new(spanned_rhs)
                            ),
                            rhs_span
                        )),
                        op: LogicalOp::LT
                    },
                    Span(start, end)
                )
            },
            GT => {
                let spanned_rhs = expr(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    LogicalExpr::LogicalOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(Spanned(
                            LogicalExpr::Factor(
                                Box::new(spanned_rhs)
                            ),
                            rhs_span
                        )),
                        op: LogicalOp::GT
                    },
                    Span(start, end)
                )
            },
            LE => {
                let spanned_rhs = expr(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    LogicalExpr::LogicalOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(Spanned(
                            LogicalExpr::Factor(
                                Box::new(spanned_rhs)
                            ),
                            rhs_span
                        )),
                        op: LogicalOp::LE
                    },
                    Span(start, end)
                )
            },
            GE => {
                let spanned_rhs = expr(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    LogicalExpr::LogicalOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(Spanned(
                            LogicalExpr::Factor(
                                Box::new(spanned_rhs)
                            ),
                            rhs_span
                        )),
                        op: LogicalOp::GE
                    },
                    Span(start, end)
                )
            },
            _ => unreachable!()
        }
    }
    Ok(lhs)
}