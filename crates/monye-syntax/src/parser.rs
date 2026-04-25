use std::collections::VecDeque;


use crate::lexer::{
    Token::{self, *},
    PrimitiveType,
    Keyword::*
};


type Ident = String;


#[derive(Clone, Debug)]
pub struct Program(pub Vec<Declaration>);


#[derive(Clone, Debug)]
pub enum Declaration {
    FnDecl{
        name: Ident,
        params: Vec<(Ident, TypeName)>,
        ret_ty: TypeName,
        body: Vec<Statement>
    }
}


#[derive(Clone, Debug)]
pub enum Statement {
    Bind {
        name: Ident,
        ty: TypeName,
        initializer: Option<Expression>
    },
    Expression(Expression)
}


#[derive(Clone, Debug)]
pub enum Expression {
    Assign {
        lhs: Box<Expression>,
        expr: Box<Expression>
    },
    BinOp {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        op: BinOp
    },
    UniOp {
        operand: Box<Expression>,
        op: UniOp
    },
    FnCall{
        name: Ident,
        args: Vec<Expression>
    },
    Number(isize),
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
    UnexpectedToken,
    UnexpectedEOF
}


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken => write!(f, "unexpected token"),
            ParseError::UnexpectedEOF => write!(f, "unexpected end of file"),
        }
    }
}


impl std::error::Error for ParseError {}


fn consume(tokens: &mut VecDeque<Token>, expect: Token) -> Result<(), ParseError> {
    if tokens.pop_front() == Some(expect) {
        Ok(())
    }
    else {
        Err(ParseError::UnexpectedToken)
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
        if peek(tokens)? == EOF {
            break;
        }
    }
    consume(tokens, EOF)?;

    Ok(Program(program))
}


fn fn_decl(tokens: &mut VecDeque<Token>) -> Result<Declaration, ParseError> {
    consume(tokens, Keyword(Fn))?;
    let Identifier(name) = next(tokens)? else {
        return Err(ParseError::UnexpectedToken);
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


fn param_list(tokens: &mut VecDeque<Token>) -> Result<Vec<(Ident, TypeName)>, ParseError> {
    let mut result = Vec::new();
    
    if peek(tokens)? == Comma {
        return Err(ParseError::UnexpectedToken);
    }

    loop {
        match peek(tokens)? {
            Identifier(_) => result.push(typed_binding(tokens)?),
            Comma => {
                consume(tokens, Comma)?;
                continue;
            },
            RParen => break,
            _ => return Err(ParseError::UnexpectedToken)
        }
    }
    Ok(result)
}


fn typed_binding(tokens: &mut VecDeque<Token>) -> Result<(Ident, TypeName), ParseError> {
    let Identifier(name) = next(tokens)? else {
        return Err(ParseError::UnexpectedToken);
    };
    consume(tokens, Colon)?;
    let ty = type_name(tokens)?;
    Ok((name, ty))
}


fn type_name(tokens: &mut VecDeque<Token>) -> Result<TypeName, ParseError> {
    match next(tokens)? {
        Type(ty) => Ok(TypeName::Primitive(ty)),
        //Identifier(ident) => Ok(TypeName::UserDefined(ident)),
        _ => Err(ParseError::UnexpectedToken),
    }
}


fn block(tokens: &mut VecDeque<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut result = Vec::new();

    consume(tokens, LBrace)?;

    loop {
        match peek(tokens)? {
            Keyword(Let) | LParen | Minus | Identifier(_) | Number(_) => {
                result.push(statement(tokens)?);
                if peek(tokens)? == Semicolon {
                    consume(tokens, Semicolon)?;
                    continue;
                }
                else {
                    break
                }
            },
            RBrace => break,
            _ => return Err(ParseError::UnexpectedToken)
        }
    }
    consume(tokens, RBrace)?;

    Ok(result)
}


fn statement(tokens: &mut VecDeque<Token>) -> Result<Statement, ParseError> {
    let result = match peek(tokens)? {
        Keyword(Let) => Ok(bind(tokens)?),
        Identifier(_) | Minus | Number(_) | LParen => {
            Ok(Statement::Expression(assign(tokens)?))
        },
        _ => Err(ParseError::UnexpectedToken),
    };
    result
}


fn bind(tokens: &mut VecDeque<Token>) -> Result<Statement, ParseError> {
    consume(tokens, Keyword(Let))?;
    let (name, ty) = typed_binding(tokens)?;
    let initializer = if peek(tokens)? == Equal {
        consume(tokens, Equal)?;
        Some(expr(tokens)?)
    }
    else {
        None
    };

    Ok(Statement::Bind {
        name,
        ty,
        initializer
    })
}


fn expr(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    assign(tokens)
}


fn assign(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    let lhs = operation(tokens)?;

    if peek(tokens)? == Equal {
        consume(tokens, Equal)?;
        Ok(Expression::Assign {
            lhs: Box::new(lhs),
            expr: Box::new(expr(tokens)?)
        })
    }
    else {
        Ok(lhs)
    }
}


fn operation(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    addition(tokens)
}


fn addition(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    let mut lhs = multiply(tokens)?;

    while [Plus, Minus].into_iter().any(|t| Ok(t) == peek(tokens)) {
        match peek(tokens)? {
            Plus => {
                consume(tokens, Plus)?;
                let rhs = multiply(tokens)?;
                lhs = Expression::BinOp {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op: BinOp::Add
                };
            },
            Minus => {
                consume(tokens, Minus)?;
                let rhs = multiply(tokens)?;
                lhs = Expression::BinOp {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op: BinOp::Sub
                };
            },
            _ => unreachable!()
        }
    }
    Ok(lhs)
}


fn multiply(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    let mut lhs = unary_op(tokens)?;

    while [Asterisk, Slash, Percent].into_iter().any(|t| Ok(t) == peek(tokens)) {
        match peek(tokens)? {
            Asterisk => {
                consume(tokens, Asterisk)?;
                let rhs = unary_op(tokens)?;
                lhs = Expression::BinOp {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op: BinOp::Mul
                };
            },
            Slash => {
                consume(tokens, Slash)?;
                let rhs = unary_op(tokens)?;
                lhs = Expression::BinOp {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op: BinOp::Div
                };
            },
            Percent => {
                consume(tokens, Percent)?;
                let rhs = unary_op(tokens)?;
                lhs = Expression::BinOp {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op: BinOp::Rem
                };
            },
            _ => unreachable!()
        }
    }
    Ok(lhs)
}


fn unary_op(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    if peek(tokens)? == Minus {
        consume(tokens, Minus)?;
        Ok(Expression::UniOp{
            operand: Box::new(factor(tokens)?),
            op: UniOp::Neg
        })
    }
    else {
        factor(tokens)
    }
}


fn factor(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    match peek(tokens)? {
        Number(n) => {
            next(tokens)?;
            Ok(Expression::Number(n))
        },
        LParen => {
            consume(tokens, LParen)?;
            let expr = expr(tokens)?;
            consume(tokens, RParen)?;
            Ok(expr)
        }
        Identifier(_) => fn_call(tokens),
        _ => Err(ParseError::UnexpectedToken)
    }
}


fn fn_call(tokens: &mut VecDeque<Token>) -> Result<Expression, ParseError> {
    let Identifier(ident) = next(tokens)? else {
        return Err(ParseError::UnexpectedToken)
    };

    if peek(tokens)? == LParen {
        let mut args = Vec::new();
        consume(tokens, LParen)?;
        if peek(tokens)? == Comma {
            return Err(ParseError::UnexpectedToken);
        }
        loop {
            match peek(tokens)? {
                RParen => break,
                LParen | Number(_) | Identifier(_) | Minus => {
                    args.push(expr(tokens)?);
                },
                Comma => {
                    consume(tokens, Comma)?;
                    continue;
                }
                _ => return Err(ParseError::UnexpectedToken)
            }
        }
        consume(tokens, RParen)?;
        Ok(Expression::FnCall {
            name: ident,
            args
        })
    }
    else {
        Ok(Expression::Value(ident))
    }
}
