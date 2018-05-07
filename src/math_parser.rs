use std::fmt::{self, Display};
use std::error::Error;
use std::str::FromStr;

pub type MathParseResult = Result<f64, MathParseError>;
pub type MathScanResult = Result<Vec<Token>, MathParseError>;

pub fn eval(expr: &str) -> MathParseResult {
    let tokens = scan(expr)?;
    let mut parser = Parser::new(tokens);
    let res = parser.expression();
    if parser.is_at_end() {
        res
    } else {
        Err(MathParseError::ExpectedButGot(String::from("end of file"), parser.current_token().unwrap()))
    }
}

pub fn scan(expr: &str) -> MathScanResult {
    use self::Op::*;

    let mut tokens: Vec<Token> = Vec::new();
    for subs in expr.split_whitespace() {
        tokens.push(match subs {
            "+" => Token::Operator(Plus),
            "-" => Token::Operator(Minus),
            "*" => Token::Operator(Times),
            "/" => Token::Operator(Slash),
            "(" => Token::Operator(LeftParen),
            ")" => Token::Operator(RightParen),
            c => match f64::from_str(c) {
                Ok(n) => Token::Number(n),
                Err(_) => return Err(MathParseError::ScanError)
            },
        });
    }
    Ok(tokens)
}

pub fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

#[derive(Debug, Clone, Copy)]
pub enum Token {
    Number(f64),
    Operator(Op),
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Plus,
    Minus,
    Times,
    Slash,
    LeftParen,
    RightParen,
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, cursor: 0 }
    }

    fn current_token(&self) -> Option<Token> {
        self.tokens.get(self.cursor).map(|t| *t)
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.cursor).map(|t| *t);
        self.cursor += 1;
        t
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn expression(&mut self) -> MathParseResult {
        let mut x = self.mult()?;
        // while (self.current_token() == Operator(Op::LeftParen)) || (self.current_token() == Operator(Op::RightParen)){
        loop {
            match self.current_token() {
                Some(Token::Operator(Op::Plus)) => {
                    self.advance();
                    x = x + self.mult()?;
                }
                Some(Token::Operator(Op::Minus)) => {
                    self.advance();
                    x = x - self.mult()?;
                }
                _ => break,
            }
        }
        Ok(x)
    }

    fn mult(&mut self) -> MathParseResult {
        let mut x = self.factor()?;
        loop {
            match self.current_token() {
                Some(Token::Operator(Op::Times)) => {
                    self.advance();
                    x = x * self.factor()?;
                }
                Some(Token::Operator(Op::Slash)) => {
                    self.advance();
                    x = x / self.factor()?;
                }
                _ => break,
            }
        }
        Ok(x)
    }

    fn factor(&mut self) -> MathParseResult {
        match self.current_token() {
            Some(Token::Operator(Op::LeftParen)) => {
                self.advance();
                let expr = self.expression();
                match self.current_token() {
                    Some(Token::Operator(Op::RightParen)) => {
                        self.advance();
                        expr
                    }
                    Some(t) => panic_because_expected("(", t),
                    None => unexpected_end(),
                }
            }
            Some(Token::Number(x)) => {
                self.advance();
                Ok(x)
            }
            // Some(t) => panic_because_expected("'(' or number literal", t),
            Some(t) => Err(MathParseError::ExpectedButGot(String::from("'(' or number literal"), t)),
            None => Err(MathParseError::UnexpectedEOF),
        }
    }
}

fn panic_because_expected(expected: &str, got: Token) -> ! {
    panic!("Expected '{}' but got '{:?}'", expected, got)
}

fn unexpected_end() -> ! {
    panic!("Unexpected end of input")
}

#[derive(Debug)]
pub enum MathParseError {
    ScanError,
    ExpectedButGot(String, Token),
    UnexpectedEOF,
}

impl Display for MathParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MathParseError::ExpectedButGot(ref e, g) => {
                write!(f, "Expected '{}' but got '{:?}'", e, g)
            }
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl Error for MathParseError {
    fn description(&self) -> &str {
        match self {
            &MathParseError::ScanError => "Could not scan input.",
            &MathParseError::ExpectedButGot(ref e, g) => "Unexpected token",
            &MathParseError::UnexpectedEOF => "Unexpected end of input",
        }
    }
}

/*
Grammar:

expression = mult {add_op mult}
mult = factor {mult_op factor}
factor = "(" expression ")" | NUMBER
add_op = "+" | "-"
mult_op = "*" | "/"

*/

// 3 + 28 / 23 * ( 9 + 8 / 7 ) / 23 * 2 / ( 3 / ( 3 + 2 ) )
