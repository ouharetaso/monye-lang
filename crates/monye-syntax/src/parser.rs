use std::collections::VecDeque;


use crate::lexer::{
    Token,
    TokenKind::{self, *},
    Span,
    PrimitiveType,
    Keyword::*
};


type Ident<'src> = &'src str;


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
pub struct Program<'src>(pub Vec<Declaration<'src>>);


#[derive(Clone, Debug)]
pub enum Declaration<'src> {
    FnDecl{
        name: Spanned<Ident<'src>>,
        params: Vec<(Spanned<Ident<'src>>, Spanned<TypeName>)>,
        ret_ty: Spanned<TypeName>,
        body: Spanned<Vec<Spanned<Statement<'src>>>>
    }
}


#[derive(Clone, Debug)]
pub enum Statement<'src> {
    Bind {
        name: Spanned<Ident<'src>>,
        ty: Spanned<TypeName>,
        initializer: Option<Spanned<Expression<'src>>>
    },
    SemicolonnedExpr(Spanned<Expression<'src>>),
    Expression(Spanned<Expression<'src>>),
}


#[derive(Clone, Debug)]
pub enum Expression<'src> {
    Assign {
        lhs: Box<Spanned<Expression<'src>>>,
        expr: Box<Spanned<Expression<'src>>>
    },
    BinOp {
        lhs: Box<Spanned<Expression<'src>>>,
        rhs: Box<Spanned<Expression<'src>>>,
        op: BinOp
    },
    UniOp {
        operand: Box<Spanned<Expression<'src>>>,
        op: UniOp
    },
    FnCall{
        name: Ident<'src>,
        args: Vec<Spanned<Expression<'src>>>
    },
    Number(u64),
    Value(Ident<'src>),
    Bool(bool),
    Unit,
    If(Box<Spanned<IfExpr<'src>>>, Vec<Spanned<IfExpr<'src>>>, Option<Spanned<Vec<Spanned<Statement<'src>>>>>),
    Block(Spanned<Vec<Spanned<Statement<'src>>>>)
}


#[derive(Clone, Debug)]
pub struct IfExpr<'src>(pub Spanned<Expression<'src>>, pub Spanned<Vec<Spanned<Statement<'src>>>>);


impl<'src> IfExpr<'src> {
    pub fn cond(&self) -> &Spanned<Expression<'src>> {
        &self.0
    }

    pub fn body(&self) -> &Spanned<Vec<Spanned<Statement<'src>>>> {
        &self.1
    }
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
    LogicalOr,
    LogicalAnd,
    Equal,
    NotEqual,
    LT,
    GT,
    LE,
    GE,
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


fn unexpected_token<'src, T>(token: Token<'src>) -> Result<T, ParseError> {
    if token.kind() == EOF {
        Err(ParseError::UnexpectedEOF)
    }
    else {
        Err(ParseError::UnexpectedToken(token.span()))
    }
}


fn consume<'src>(tokens: &mut VecDeque<Token<'src>>, expect: TokenKind<'src>) -> Result<Span, ParseError> {
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


fn next<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Token<'src>, ParseError> {
    tokens.pop_front().ok_or(ParseError::UnexpectedEOF)
}


fn peek<'src>(tokens: &VecDeque<Token<'src>>) -> Result<Token<'src>, ParseError> {
    tokens.get(0).cloned().ok_or(ParseError::UnexpectedEOF)
}


fn is_expr_first_token<'src>(token: &Token<'src>) -> bool {
    matches!(
        token.kind(),
        Number(_)   | Identifier(_) | LParen | Minus | Exclamation | 
        Keyword(If | True | False | Unit) | LBrace
    )
}


pub fn parse<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Program<'src>, ParseError> {
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


fn fn_decl<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Declaration<'src>, ParseError> {
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


fn param_list<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Vec<(Spanned<Ident<'src>>, Spanned<TypeName>)>, ParseError> {
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


fn typed_binding<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<(Spanned<Ident<'src>>, Spanned<TypeName>), ParseError> {
    let token = next(tokens)?;
    let Token(kind, span) = token.clone();

    let Identifier(name) = kind else {
        return unexpected_token(token)
    };

    consume(tokens, Colon)?;
    let ty = type_name(tokens)?;
    Ok((Spanned(name, span), ty))
}


fn type_name<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<TypeName>, ParseError> {
    match next(tokens)? {
        Token(Type(ty), span) => Ok(Spanned(TypeName::Primitive(ty), span)),
        Token(Keyword(Unit), span) => Ok(Spanned(TypeName::Unit, span)),
        Token(Keyword(Never), span) => Ok(Spanned(TypeName::Never, span)),
        //Identifier(ident) => Ok(TypeName::UserDefined(ident)),
        token @ _ => return unexpected_token(token),
    }
}


fn block<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Vec<Spanned<Statement<'src>>>>, ParseError> {
    let mut result = Vec::new();

    let start = peek(tokens)?.span().start();
    consume(tokens, LBrace)?;

    loop {
        match peek(tokens)? {
            Token(Keyword(Let), _) => {
                result.push(statement(tokens)?);
                continue;
            },
            token @ Token(_, _) if is_expr_first_token(&token) => {
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


fn statement<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Statement<'src>>, ParseError> {
    match peek(tokens)? {
        Token(Keyword(Let), _) => {
            let bind = bind(tokens)?;
            consume(tokens, Semicolon)?;
            Ok(bind)
        },
        token @ Token(_, span_start) if is_expr_first_token(&token)=> {
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


fn bind<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Statement<'src>>, ParseError> {
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


fn expr<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    assign(tokens)
}


fn assign<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    let spanned_lhs = logic_expr(tokens)?;
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


fn operation<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    addition(tokens)
}


#[allow(unused_assignments)]
fn addition<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
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
fn multiply<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
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


fn unary_op<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
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


fn factor<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
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
        Token(LBrace, _) => {
            let block = block(tokens)?;
            let span = block.span();
            Ok(Spanned(Expression::Block(block), span))
        },
        token @ _ => return unexpected_token(token)
    }
}


fn fn_call<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
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
            token @ Token(_, _) if is_expr_first_token(&token) => {
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
fn if_expr<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
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
            Box::new(first),
            else_ifs,
            else_clause
        ),
        Span(start, end)
    ))
}


fn logic_expr<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    logical_or(tokens)
}


#[allow(unused_assignments)]
fn logical_or<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    let mut lhs = logical_and(tokens)?;
    let Span(start, mut end) = lhs.span();

    while &DoubleVbar == peek(tokens)?.kind() {
        consume(tokens, DoubleVbar)?;
        let spanned_rhs = logical_and(tokens)?;
        end = spanned_rhs.span().end();
        lhs = Spanned(
            Expression::BinOp {
                lhs: Box::new(lhs),
                rhs: Box::new(spanned_rhs),
                op: BinOp::LogicalOr
            },
            Span(start, end)
        )
    }
    Ok(lhs)
}


#[allow(unused_assignments)]
fn logical_and<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    let mut lhs = equality(tokens)?;
    let Span(start, mut end) = lhs.span();

    while &DoubleAmpersand == peek(tokens)?.kind() {
        consume(tokens, DoubleAmpersand)?;
        let spanned_rhs = equality(tokens)?;
        end = spanned_rhs.span().end();
        lhs = Spanned(
            Expression::BinOp {
                lhs: Box::new(lhs),
                rhs: Box::new(spanned_rhs),
                op: BinOp::LogicalAnd
            },
            Span(start, end)
        )
    }
    Ok(lhs)
}


#[allow(unused_assignments)]
fn equality<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    let mut lhs = order(tokens)?;
    let Span(start, mut end) = lhs.span();

    while let DoubleEqual | NE = peek(tokens)?.kind() {
        match next(tokens)?.kind() {
            DoubleEqual => {
                let spanned_rhs = order(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::Equal
                    },
                    Span(start, end)
                )
            },
            NE => {
                let spanned_rhs = order(tokens)?;
                end = spanned_rhs.span().end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::NotEqual
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
fn order<'src>(tokens: &mut VecDeque<Token<'src>>) -> Result<Spanned<Expression<'src>>, ParseError> {
    let spanned_expr = operation(tokens)?;
    let Span(start, mut end) = spanned_expr.span();
    let mut lhs = spanned_expr;

    while let LT | GT | LE | GE = peek(tokens)?.kind() {
        match next(tokens)?.kind() {
            LT => {
                let spanned_rhs = operation(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::LT
                    },
                    Span(start, end)
                )
            },
            GT => {
                let spanned_rhs = operation(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::GT
                    },
                    Span(start, end)
                )
            },
            LE => {
                let spanned_rhs = operation(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::LE
                    },
                    Span(start, end)
                )
            },
            GE => {
                let spanned_rhs = operation(tokens)?;
                let rhs_span = spanned_rhs.span();
                end = rhs_span.end();
                lhs = Spanned(
                    Expression::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(spanned_rhs),
                        op: BinOp::GE
                    },
                    Span(start, end)
                )
            },
            _ => unreachable!()
        }
    }
    Ok(lhs)
}