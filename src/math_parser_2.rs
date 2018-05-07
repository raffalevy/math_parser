use std::collections::HashMap;
use std::num::ParseFloatError;
use std::fmt::{self, Display};
use std::error::Error;
use std::str::FromStr;

use lexer::{self, Token, TokenValue};

pub type MathParseResult = Result<f64, MathParseError>;
// pub type MathScanResult = Result<Vec<Token>, MathParseError>;

pub struct EvalContext {
    vars: HashMap<String, f64>,
}

impl EvalContext {
    pub fn new() -> Self {
        EvalContext {
            vars: HashMap::new(),
        }
    }

    pub fn eval(&mut self, expr: &str) -> MathParseResult {
        // let tokens = scan(expr)?;
        let tokens = lexer::scan(String::from(expr));
        let mut parser = Parser::new(tokens, self);
        let res = parser.expression()?;
        if parser.is_at_end() {
            Ok(res)
        } else {
            Err(MathParseError::ExpectedButGot(
                String::from("end of file"),
                parser.current_token().unwrap(),
            ))
        }
        // Ok(res)
    }
}

// pub fn scan(expr: &str) -> MathScanResult {
//     let mut tokens: Vec<Token> = Vec::new();
//     for subs in expr.split_whitespace() {
//         tokens.push(match subs {
//             "+" => Token::Operator(Plus),
//             "-" => Token::Operator(Minus),
//             "*" => Token::Operator(Times),
//             "/" => Token::Operator(Slash),
//             "(" => Token::Operator(LeftParen),
//             ")" => Token::Operator(RightParen),
//             c => match f64::from_str(c) {
//                 Ok(n) => Token::Number(n),
//                 Err(_) => return Err(MathParseError::ScanError)
//             },
//         });
//     }
//     Ok(tokens)
// }

// pub fn is_digit(c: u8) -> bool {
//     c >= b'0' && c <= b'9'
// }

// #[derive(Debug, Clone, Copy)]
// pub enum Token {
//     Number(f64),
//     Operator(Op),
// }

// #[derive(Debug, Clone, Copy)]
// pub enum Op {
//     Plus,
//     Minus,
//     Times,
//     Slash,
//     LeftParen,
//     RightParen,
// }

struct Parser<'a> {
    tokens: Vec<Token>,
    cursor: usize,
    context: &'a mut EvalContext,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, context: &'a mut EvalContext) -> Self {
        Parser {
            tokens,
            cursor: 0,
            context,
        }
    }

    fn current_token(&self) -> Option<Token> {
        self.tokens.get(self.cursor).map(|t| t.clone())
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.cursor).map(|t| t.clone());
        self.cursor += 1;
        t
    }

    fn look_ahead(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.cursor + offset).map(|t| t.clone())
    }

    fn is_at_end(&self) -> bool {
        match self.current_token() {
            None
            | Some(Token {
                token_value: TokenValue::EOF,
                ..
            }) => true,
            Some(_) => false,
        }
        // self.cursor >= self.tokens.len()
    }

    fn assign_variable(&mut self, name: String, value: f64) {
        println!("Set variable '{}' to {}.", &name, value);
        self.context.vars.insert(name, value);
    }

    fn get_variable(&self, name: &String) -> Option<f64> {
        self.context.vars.get(name).map(|x| *x)
    }
    
    fn expression(&mut self) -> MathParseResult {
        match self.look_ahead(1) {
            Some(Token {
                token_value: TokenValue::Equal,
                ..
            }) => self.assignment(),
            _ => self.sum_expression(),
        }
    }

    fn sum_expression(&mut self) -> MathParseResult {
        let mut x = self.mult()?;
        loop {
            match self.current_token() {
                Some(Token {
                    token_value: TokenValue::Plus,
                    ..
                }) => {
                    self.advance();
                    x = x + self.mult()?;
                }
                Some(Token {
                    token_value: TokenValue::Minus,
                    ..
                }) => {
                    self.advance();
                    x = x - self.mult()?;
                }
                _ => break,
            }
        }
        Ok(x)
    }

    fn assignment(&mut self) -> MathParseResult {
        match self.current_token() {
            Some(Token {
                token_value: TokenValue::Identifier(name),
                ..
            }) => {
                self.advance();
                match self.current_token() {
                    Some(Token {
                        token_value: TokenValue::Equal,
                        ..
                    }) => {
                        self.advance();
                        // match self.expression() {
                        //     Ok(expr_value) => {
                        //         self.assign_value(name, expr_value);
                        //         Ok(expr_value)
                        //     }
                        //     Err(err) => Err(err)
                        // }
                        let expr_value = self.expression()?;
                        self.assign_variable(name, expr_value);
                        Ok(expr_value)
                    },
                    Some(t) => {
                        Err(MathParseError::ExpectedButGot(
                            String::from("[identifier]"),
                            t,
                        ))
                    }
                    None => Err(MathParseError::UnexpectedEOF),
                }
            }
            Some(t) => Err(MathParseError::ExpectedButGot(
                String::from("[identifier]"),
                t,
            )),
            None => Err(MathParseError::UnexpectedEOF),
        }
    }

    fn mult(&mut self) -> MathParseResult {
        let mut x = self.factor()?;
        loop {
            match self.current_token() {
                Some(Token {
                    token_value: TokenValue::Times,
                    ..
                }) => {
                    self.advance();
                    x = x * self.factor()?;
                }
                Some(Token {
                    token_value: TokenValue::Slash,
                    ..
                }) => {
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
            Some(Token {
                token_value: TokenValue::LeftParen,
                ..
            }) => {
                self.advance();
                let expr = self.expression();
                match self.current_token() {
                    Some(Token {
                        token_value: TokenValue::RightParen,
                        ..
                    }) => {
                        self.advance();
                        expr
                    }
                    Some(t) => panic_because_expected(")", t),
                    None => unexpected_end(),
                }
            }
            Some(Token {
                token_value: TokenValue::NumberLiteral(lit),
                ..
            }) => {
                self.advance();
                f64::from_str(&lit).map_err(|err| MathParseError::CouldNotParseFloat(err))
            },
            Some(Token {
                token_value: TokenValue::Identifier(name),
                ..
            }) => {
                self.advance();
                self.get_variable(&name).ok_or(MathParseError::UnknownIdentifier(name))
            }
            // Some(t) => panic_because_expected("'(' or number literal", t),
            Some(t) => Err(MathParseError::ExpectedButGot(
                String::from("'(' or number literal"),
                t,
            )),
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
    CouldNotParseFloat(ParseFloatError),
    UnknownIdentifier(String)
}

impl Display for MathParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MathParseError::ExpectedButGot(ref e, ref g) => {
                write!(f, "Expected '{}' but got '{:?}'", e, g)
            }
            &MathParseError::UnknownIdentifier(ref name) => {
                 write!(f, "Unknown identifier '{}'", name)
            }
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl Error for MathParseError {
    fn description(&self) -> &str {
        match self {
            &MathParseError::ScanError => "Could not scan input.",
            &MathParseError::ExpectedButGot(..) => "Unexpected token",
            &MathParseError::UnexpectedEOF => "Unexpected end of input",
            &MathParseError::CouldNotParseFloat(_) => "Could not parse float",
            &MathParseError::UnknownIdentifier(_) => "Unknown identifier"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &MathParseError::CouldNotParseFloat(ref pferr) => Some(pferr),
            _ => None,
        }
    }
}

/*
Grammar:

replinput = expression | EOF;
expression = mult {add_op mult} | assignment
mult = factor {mult_op factor}
factor = "(" expression ")" | NUMBER | IDENTIFIER
add_op = "+" | "-"
mult_op = "*" | "/"
assignment = IDENTIFIER "=" expression

*/

// 3 + 28 / 23 * ( 9 + 8 / 7 ) / 23 * 2 / ( 3 / ( 3 + 2 ) )
