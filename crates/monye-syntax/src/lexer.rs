use std::str::{CharIndices, FromStr};
use std::iter::Peekable;
use std::collections::VecDeque;


pub trait Lexer<'a> {
    fn lex(&mut self) -> Result<VecDeque<Token>, LexError>;
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Span(pub usize, pub usize);

impl Span {
    pub fn start(&self) -> usize {
        self.0
    }

    pub fn end(&self) -> usize {
        self.1
    }
}

#[derive(Debug, Clone)]
pub struct Token(pub TokenKind, pub Span);

impl Token {
    pub fn kind(&self) -> &TokenKind {
        &self.0
    }

    pub fn span(&self) -> Span {
        self.1
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    Number(u64),
    Keyword(Keyword),
    Identifier(String),
    Type(PrimitiveType),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Colon,
    Semicolon,
    VBar,
    Ampersand,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Equal,
    Arrow,
    EOF
}

impl PartialEq<TokenKind> for &TokenKind  {
    fn eq(&self, other: &TokenKind) -> bool {
        <TokenKind as PartialEq>::eq(*self, other)
    }
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



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    Integer
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
            PrimitiveType::I8 => "i8",
            PrimitiveType::U8 => "u8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::U16 => "u16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::U32 => "u32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::U64 => "u64",
            PrimitiveType::Integer => "integer"
        }
    }

    pub fn try_cast(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            (Self::Integer, Self::Integer) => Some(Self::Integer),
            (Self::Integer, cast_to @ _) => Some(*cast_to),
            (cast_to @ _, Self::Integer) => Some(*cast_to),
            (_, _) => if self == other {
                Some(*self)
            }
            else {
                None
            }
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



#[derive(PartialEq, Debug, Clone)]
pub enum LexError {
    UnsupportedToken,
    UnknownKeyword,
    ParseNumberError
}


impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnsupportedToken => write!(f, "Unsupported token"),
            LexError::UnknownKeyword => write!(f, "Unknown keyword"),
            LexError::ParseNumberError => write!(f, "Failed to parse number"),
        }
    }
}


impl std::error::Error for LexError {}


pub struct StringLexer<'a> {
    char_indices: Peekable<CharIndices<'a>>
}


impl<'a> StringLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            char_indices: input.char_indices().peekable()
        }
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        while let Some(&(_i, c)) = self.char_indices.peek() {
            if c.is_ascii_whitespace() {
                self.char_indices.next();
                continue;
            }
            else {
                break;
            }
        }
        let mut start = 0;
        let mut len = 0;
        let kind = if let Some((i, c)) = self.char_indices.next() {
            start = i;
            len = c.len_utf8();
            match c {
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                ':' => TokenKind::Colon,
                ';' => TokenKind::Semicolon,
                '&' => TokenKind::Ampersand,
                '|' => TokenKind::VBar,
                '+' => TokenKind::Plus,
                '*' => TokenKind::Asterisk,
                '/' => TokenKind::Slash,
                '%' => TokenKind::Percent,
                '=' => TokenKind::Equal,
                '-' => {
                    match self.char_indices.peek() {
                        Some(&(_ii, '>')) => {
                            self.char_indices.next();
                            len += '>'.len_utf8();
                            TokenKind::Arrow
                        },
                        _ => TokenKind::Minus
                    }
                },
                '0' => {
                    match self.char_indices.peek() {
                        Some((_ii, '0'..'9')) => return Err(LexError::ParseNumberError),
                        _ => TokenKind::Number(0)
                    }
                },
                first @ '1'..='9' => {
                    let digits = std::iter::successors(Some(first), |_|{
                        self.char_indices.next_if(|&(_, c)|{
                            c.is_numeric()
                        })
                        .map(|(_, c)| c)
                    })
                    .collect::<String>();
                    len = digits.len();
                    TokenKind::Number(digits.parse().map_err(|_|LexError::ParseNumberError)?)
                }
                first @ ('a'..='z' | 'A'..='Z' | '_') => {
                    let identifier = std::iter::successors(Some(first), |_|{
                        self.char_indices.next_if(|&(_, c)| {
                            c.is_alphanumeric() || c == '_'
                        })
                        .map(|(_, c)| c)
                    })
                    .collect::<String>();

                    len = identifier.len();

                    if let Ok(keyword) = Keyword::from_str(&identifier) {
                        TokenKind::Keyword(keyword)
                    }
                    else if let Ok(ty) = PrimitiveType::from_str(&identifier) {
                        TokenKind::Type(ty)
                    } 
                    else {
                        TokenKind::Identifier(identifier)
                    }
                },
                _ => return Err(LexError::UnsupportedToken)
            }
        }
        else {
            TokenKind::EOF
        };

        Ok(Token(kind, Span(start, start + len)))
    }
}


impl<'a> Lexer<'a> for StringLexer<'a> {
    fn lex(&mut self) -> Result<VecDeque<Token>, LexError> {
        let mut result = VecDeque::new();

        loop {
            let token = self.next_token()?;

            result.push_back(token.clone());
            if token.0 == TokenKind::EOF {
                break;
            }
            else {
                continue;
            }
        }

        Ok(result)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn get_token_kinds(input: &str) -> Result<Vec<TokenKind>, LexError> {
        let mut lexer = StringLexer::new(input);
        lexer.lex().map(|tokens| tokens.into_iter().map(|t| t.0).collect())
    }

    #[test]
    fn test_lex_symbols() {
        let input = ":(){ :|:& };: -> -";
        let expect = vec![
            TokenKind::Colon,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Colon,
            TokenKind::VBar,
            TokenKind::Colon,
            TokenKind::Ampersand,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Colon,
            TokenKind::Arrow,
            TokenKind::Minus,
            TokenKind::EOF
        ];

        assert_eq!(get_token_kinds(input).unwrap(), expect);
    }

    #[test]
    fn test_lex_number() {
        let patterns = vec![
            ("32767", Ok(vec![TokenKind::Number(32767), TokenKind::EOF])),
            ("0", Ok(vec![TokenKind::Number(0), TokenKind::EOF])),
            ("-42", Ok(vec![TokenKind::Minus, TokenKind::Number(42), TokenKind::EOF])),
            ("042", Err(LexError::ParseNumberError)),
            ("-042", Err(LexError::ParseNumberError)),
        ];

        for (input, expect) in patterns {
            assert_eq!(get_token_kinds(input), expect);
        }
    }

    #[test]
    fn test_lex_identifiers_and_keywords() {
        let input = "fn main let i32 u64 _variable_name";
        let expect = vec![
            TokenKind::Keyword(Keyword::Fn),
            TokenKind::Identifier("main".to_string()),
            TokenKind::Keyword(Keyword::Let),
            TokenKind::Type(PrimitiveType::I32),
            TokenKind::Type(PrimitiveType::U64),
            TokenKind::Identifier("_variable_name".to_string()),
            TokenKind::EOF,
        ];

        assert_eq!(get_token_kinds(input).unwrap(), expect);
    }

    #[test]
    fn test_lex_complex() {
        let input = "fn add(a: i32, b: i32) -> i32 { let sum = a + b; sum }";
        let expect = vec![
            TokenKind::Keyword(Keyword::Fn),
            TokenKind::Identifier("add".to_string()),
            TokenKind::LParen,
            TokenKind::Identifier("a".to_string()),
            TokenKind::Colon,
            TokenKind::Type(PrimitiveType::I32),
            TokenKind::Comma,
            TokenKind::Identifier("b".to_string()),
            TokenKind::Colon,
            TokenKind::Type(PrimitiveType::I32),
            TokenKind::RParen,
            TokenKind::Arrow,
            TokenKind::Type(PrimitiveType::I32),
            TokenKind::LBrace,
            TokenKind::Keyword(Keyword::Let),
            TokenKind::Identifier("sum".to_string()),
            TokenKind::Equal,
            TokenKind::Identifier("a".to_string()),
            TokenKind::Plus,
            TokenKind::Identifier("b".to_string()),
            TokenKind::Semicolon,
            TokenKind::Identifier("sum".to_string()),
            TokenKind::RBrace,
            TokenKind::EOF,
        ];

        assert_eq!(get_token_kinds(input).unwrap(), expect);
    }
}