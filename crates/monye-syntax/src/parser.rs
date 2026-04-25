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


#[derive(Clone, Debug)]
pub struct Program(pub Vec<Declaration>);


#[derive(Clone, Debug)]
pub enum Declaration {
    FnDecl{
        name: Spanned<Ident>,
        params: Vec<(Spanned<Ident>, Spanned<TypeName>)>,
        ret_ty: Spanned<TypeName>,
        body: Vec<Spanned<Statement>>
    }
}


#[derive(Clone, Debug)]
pub enum Statement {
    Bind {
        name: Spanned<Ident>,
        ty: Spanned<TypeName>,
        initializer: Option<Spanned<Expression>>
    },
    Expression(Spanned<Expression>)
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
}


#[derive(Clone, Debug)]
pub enum TypeName {
    Primitive(PrimitiveType),
    //UserDefined(Ident),
}


#[derive(Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}


#[derive(Clone, Debug)]
pub enum UniOp {
    Neg
}


#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Span),
    UnexpectedEOF
}


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(span) => write!(f, "unexpected token at : {}-{}", span.0, span.1),
            ParseError::UnexpectedEOF => write!(f, "unexpected end of file"),
        }
    }
}


impl std::error::Error for ParseError {}


fn consume(tokens: &mut VecDeque<Token>, expect: TokenKind) -> Result<Span, ParseError> {
    if let Some(Token(actual, span)) = tokens.pop_front()  {
        if expect == actual {
            Ok(span)
        }
        else {
            Err(ParseError::UnexpectedToken(span))
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
        if peek(tokens)?.0 == EOF {
            break;
        }
    }
    consume(tokens, EOF)?;

    Ok(Program(program))
}


fn fn_decl(tokens: &mut VecDeque<Token>) -> Result<Declaration, ParseError> {
    consume(tokens, Keyword(Fn))?;
    let name = match next(tokens)? {
        Token(Identifier(name), span) => Spanned(name, span),
        Token(_, span) => return Err(ParseError::UnexpectedToken(span))
    };
    consume(tokens, LParen)?;
    let params = param_list(tokens)?;
    consume(tokens, RParen)?;
    consume(tokens, Arrow)?;
    let ret_ty = type_name(tokens)?;
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
    
    let Token(kind, span) = peek(tokens)?;
    if kind == Comma {
        return Err(ParseError::UnexpectedToken(span));
    }

    loop {
        match peek(tokens)? {
            Token(Identifier(_), _) => result.push(typed_binding(tokens)?),
            Token(Comma, _) => {
                consume(tokens, Comma)?;
                continue;
            },
            Token(RParen, _) => break,
            Token(_, span) => return Err(ParseError::UnexpectedToken(span))
        }
    }
    Ok(result)
}


fn typed_binding(tokens: &mut VecDeque<Token>) -> Result<(Spanned<Ident>, Spanned<TypeName>), ParseError> {
    let Token(kind, span) = next(tokens)?;

    let Identifier(name) = kind else {
        return Err(ParseError::UnexpectedToken(span));
    };

    consume(tokens, Colon)?;
    let ty = type_name(tokens)?;
    Ok((Spanned(name, span), ty))
}


fn type_name(tokens: &mut VecDeque<Token>) -> Result<Spanned<TypeName>, ParseError> {
    match next(tokens)? {
        Token(Type(ty), span) => Ok(Spanned(TypeName::Primitive(ty), span)),
        //Identifier(ident) => Ok(TypeName::UserDefined(ident)),
        Token(_, span) => Err(ParseError::UnexpectedToken(span)),
    }
}


fn block(tokens: &mut VecDeque<Token>) -> Result<Vec<Spanned<Statement>>, ParseError> {
    let mut result = Vec::new();

    consume(tokens, LBrace)?;

    loop {
        match peek(tokens)? {
            Token(
                Keyword(Let) | LParen | Minus | Identifier(_) | Number(_),
                _span
            ) => {
                result.push(statement(tokens)?);
                if peek(tokens)?.0 == Semicolon {
                    consume(tokens, Semicolon)?;
                    continue;
                }
                else {
                    break
                }
            },
            Token(RBrace, _span) => break,
            Token(_, span) => return Err(ParseError::UnexpectedToken(span))
        }
    }
    consume(tokens, RBrace)?;

    Ok(result)
}


fn statement(tokens: &mut VecDeque<Token>) -> Result<Spanned<Statement>, ParseError> {
    match peek(tokens)? {
        Token(Keyword(Let), _) => Ok(bind(tokens)?),
        Token(
            Identifier(_) | Minus | Number(_) | LParen,
            span_start
        ) => {
            let spanned_expr = assign(tokens)?;
            let end = spanned_expr.1.1;

            Ok(Spanned(Statement::Expression(spanned_expr), Span(span_start.0, end)))
        },
        Token(_, span) => Err(ParseError::UnexpectedToken(span)),
    }
}


fn bind(tokens: &mut VecDeque<Token>) -> Result<Spanned<Statement>, ParseError> {
    let Span(start, _) = consume(tokens, Keyword(Let))?;
    let (name, ty) = typed_binding(tokens)?;
    let initializer = if peek(tokens)?.0 == Equal {
        consume(tokens, Equal)?;
        Some(expr(tokens)?)
    }
    else {
        None
    };

    let end = if let Some(Spanned(_, ref span)) = initializer {
        span.1
    }
    else {
        ty.1.1
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
    let Span(start, _) = spanned_lhs.1;

    if peek(tokens)?.0 == Equal {
        consume(tokens, Equal)?;
        let spanned_expr = expr(tokens)?;
        let Span(_, end) = spanned_expr.1;
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
    let Span(start, mut end) = lhs.1;

    while let Token(Plus | Minus, _) = peek(tokens)? {
        match peek(tokens)? {
            Token(Plus, _) => {
                consume(tokens, Plus)?;
                let spanned_rhs = multiply(tokens)?;
                end = spanned_rhs.1.1;
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
                end = spanned_rhs.1.1;
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
    let Span(start, mut end) = lhs.1;

    while let Token(Asterisk | Slash | Percent, _) = peek(tokens)? {
        match peek(tokens)? {
            Token(Asterisk, _) => {
                consume(tokens, Asterisk)?;
                let spanned_rhs = unary_op(tokens)?;
                end = spanned_rhs.1.1;
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
                end = spanned_rhs.1.1;
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
                end = spanned_rhs.1.1;
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
    if peek(tokens)?.0 == Minus {
        let Span(start, _) = consume(tokens, Minus)?;
        let spanned_operand = factor(tokens)?;
        let end = spanned_operand.1.1;

        Ok(Spanned(
            Expression::UniOp{
                operand: Box::new(spanned_operand),
                op: UniOp::Neg
            },
            Span(start, end)
        ))
    }
    else {
        factor(tokens)
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
        Token(_, span) => Err(ParseError::UnexpectedToken(span))
    }
}


fn fn_call(tokens: &mut VecDeque<Token>) -> Result<Spanned<Expression>, ParseError> {
    let Token(kind, span) = next(tokens)?;
    let Identifier(ident) = kind else {
        return Err(ParseError::UnexpectedToken(span))
    };

    if peek(tokens)?.0 == LParen {
        let start = span.0;
        let mut args = Vec::new();
        consume(tokens, LParen)?;
        if peek(tokens)?.0 == Comma {
            return Err(ParseError::UnexpectedToken(next(tokens)?.1));
        }
        loop {
            match peek(tokens)? {
                Token(RParen, _) => break,
                Token(
                    LParen | Number(_) | Identifier(_) | Minus,
                    _
                ) => {
                    args.push(expr(tokens)?);
                },
                Token(Comma, _) => {
                    consume(tokens, Comma)?;
                    continue;
                }
                Token(_, span) => return Err(ParseError::UnexpectedToken(span))
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
