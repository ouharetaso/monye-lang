use std::str::{Chars, FromStr};
use std::iter::Peekable;


pub trait Lexer<'a> {
    fn lex(&mut self) -> Result<Vec<Token>, LexError>;
}


#[derive(PartialEq, Debug)]
pub enum Token {
    Number(isize),
    Keyword(Keyword),
    Identifier(String),
    Type(PrimitiveType),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Semicolon,
    VBar,
    Ampersand,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Equal,
    Arrow,
    EOF
}

#[derive(PartialEq, Debug, Clone)]
pub enum Keyword {
    Fn,
    Let,
}

impl Keyword {
    const ALL: &'static [Keyword] = &[
        Keyword::Fn,
        Keyword::Let,
    ];

    fn as_str(&self) -> &'static str {
        match self {
            Keyword::Fn => "fn",
            Keyword::Let => "let",
        }
    }
}

pub struct KeywordError {}

impl std::str::FromStr for Keyword {
    type Err = KeywordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.iter()
            .find(|&k| k.as_str() == s)
            .cloned()
            .ok_or(KeywordError{})
    }
}



#[derive(PartialEq, Debug, Clone)]
pub enum PrimitiveType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

impl PrimitiveType {
    const ALL: &'static [PrimitiveType] = &[
        PrimitiveType::I8,
        PrimitiveType::U8,
        PrimitiveType::I16,
        PrimitiveType::U16,
        PrimitiveType::I32,
        PrimitiveType::U32,
        PrimitiveType::I64,
        PrimitiveType::U64,
    ];

    fn as_str(&self) -> &'static str {
        match self {
            PrimitiveType::I8 => "u8",
            PrimitiveType::U8 => "i8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::U16 => "u16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::U32 => "u32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::U64 => "u64",
        }
    }
}

pub struct PrimitiveTypeError{}

impl std::str::FromStr for PrimitiveType {
    type Err = PrimitiveTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.iter()
            .find(|&k| k.as_str() == s)
            .cloned()
            .ok_or(PrimitiveTypeError{})
    }
}



#[derive(PartialEq, Debug)]
pub enum LexError {
    UnsupportedToken,
    UnknownKeyword,
    ParseNumberError
}


pub struct StringLexer<'a> {
    chars: Peekable<Chars<'a>>
}


impl<'a> StringLexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable()
        }
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_whitespace() {
                self.chars.next();
                continue;
            }
            else {
                break;
            }
        }

        if let Some(c) = self.chars.next() {
            match c {
                '(' => Ok(Token::LParen),
                ')' => Ok(Token::RParen),
                '{' => Ok(Token::LBrace),
                '}' => Ok(Token::RBrace),
                ':' => Ok(Token::Colon),
                ';' => Ok(Token::Semicolon),
                '&' => Ok(Token::Ampersand),
                '|' => Ok(Token::VBar),
                '+' => Ok(Token::Plus),
                '*' => Ok(Token::Asterisk),
                '/' => Ok(Token::Slash),
                '=' => Ok(Token::Equal),
                '-' => {
                    match self.chars.next() {
                        Some('>') => Ok(Token::Arrow),
                        Some('0') => {
                            match self.chars.peek() {
                                Some('0'..'9') => Err(LexError::ParseNumberError),
                                _ => Ok(Token::Number(-0))
                            }
                        },
                        first @ Some('1'..='9') =>  {
                            let digits = std::iter::once('-')
                                .chain(std::iter::successors(first, |_|{
                                    self.chars.next_if(|c| c.is_numeric())
                                }))
                                .collect::<String>();

                            Ok(Token::Number(digits.parse().map_err(|_|LexError::ParseNumberError)?))
                        },
                        _ => Ok(Token::Minus)
                    }
                },
                '0' => {
                    match self.chars.peek() {
                        Some('0'..'9') => Err(LexError::ParseNumberError),
                        _ => Ok(Token::Number(0))
                    }
                },
                first @ '1'..='9' => {
                    let digits = std::iter::successors(Some(first), |_|{
                        self.chars.next_if(|c| c.is_numeric())
                    })
                    .collect::<String>();

                    Ok(Token::Number(digits.parse().map_err(|_|LexError::ParseNumberError)?))
                }
                first @ ('a'..='z' | 'A'..='Z' | '_') => {
                    let identifier = std::iter::successors(Some(first), |_|{
                        self.chars.next_if(|&c| c.is_alphanumeric() || c == '_')
                    })
                    .collect::<String>();
                    
                    if let Ok(keyword) = Keyword::from_str(&identifier) {
                        Ok(Token::Keyword(keyword))
                    }
                    else if let Ok(ty) = PrimitiveType::from_str(&identifier) {
                        Ok(Token::Type(ty))
                    } 
                    else {
                        Ok(Token::Identifier(identifier))
                    }
                },
                _ => Err(LexError::UnsupportedToken)
            }
        }
        else {
            Ok(Token::EOF)
        }
    }
}


impl<'a> Lexer<'a> for StringLexer<'a> {
    fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut result = Vec::new();

        loop {
            let token = self.next_token()?;

            if token == Token::EOF {
                result.push(Token::EOF);
                break;
            }
            else {
                result.push(token)
            }
        }

        Ok(result)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_symbols() {
        let input = ":(){ :|:& };:";
        let mut lexer = StringLexer::new(input);
        let expect = vec![
            Token::Colon,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::Colon,
            Token::VBar,
            Token::Colon,
            Token::Ampersand,
            Token::RBrace,
            Token::Semicolon,
            Token::Colon,
            Token::EOF
        ];

        let Ok(tokens) = lexer.lex() else { panic!() };

        assert_eq!(tokens, expect);
    }

    #[test]
    fn test_lex_number() {
        let patterns = vec![
            (
                "32767",
                Ok(vec![
                    Token::Number(32767),
                    Token::EOF
                ])
            ),
            (
                "0",
                Ok(vec![
                    Token::Number(0),
                    Token::EOF
                ])
            ),
            (
                "-42",
                Ok(vec![
                    Token::Number(-42),
                    Token::EOF
                ])
            ),
            (
                "042",
                Err(LexError::ParseNumberError)
            ),
            (
                "-042",
                Err(LexError::ParseNumberError)
            ),
        ];


        for (input, expect) in patterns {
            let mut lexer = StringLexer::new(input);
            assert_eq!(lexer.lex(), expect);
        }
    }
}