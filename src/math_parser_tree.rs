use std::error::Error;
use lexer::*;
use std::num::ParseFloatError;
use std::fmt::{self, Display};
use std::str::FromStr;
use ast::*;

pub fn parse_repl(input: &str) -> Result<ReplTree, MathParseError> {
    let tokens = scan(String::from(input));
    let mut parser = Parser::new(tokens);
    let exp = parser.parse_expression_or_eof()?;
    if parser.is_at_end() {
        Ok(exp)
    } else {
        Err(MathParseError::ExpectedButGot(
            String::from("end of file"),
            parser.current_token().unwrap(),
        ))
    }
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
    }

    fn parse_expression_or_eof(&mut self) -> Result<ReplTree, MathParseError> {
        match self.current_token() {
            None
            | Some(Token {
                token_value: TokenValue::EOF,
                ..
            }) => Ok(ReplTree::Empty),
            Some(_) => Ok(ReplTree::Expr(self.parse_expression()?)),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, MathParseError> {
        let first_term = self.parse_mult()?;
        let mut terms: Vec<(BinOp, Expr)> = Vec::new();
        loop {
            match self.current_token() {
                Some(Token {
                    token_value: TokenValue::Plus,
                    ..
                }) => {
                    self.advance();
                    terms.push((BinOp::Plus, self.parse_mult()?));
                }
                Some(Token {
                    token_value: TokenValue::Minus,
                    ..
                }) => {
                    self.advance();
                    terms.push((BinOp::Minus, self.parse_mult()?));
                }
                _ => break,
            }
        }
        // println!("Ter: {:?}", terms);
        match terms.len() {
            0 => Ok(first_term),
            1 => {
                let (op, expr) = unsafe { terms.get_unchecked(0).clone() };
                Ok(Expr {
                    line: first_term.line,
                    expr_type: ExprType::Binary(op, Box::new(first_term), Box::new(expr)),
                })
            }
            _ => {
                let mut current_expr: Option<Expr> = None;
                for (op, expr) in terms {
                    current_expr = Some(match current_expr {
                        None => Expr {
                            line: first_term.line,
                            expr_type: ExprType::Binary(
                                op,
                                Box::new(first_term.clone()),
                                Box::new(expr),
                            ),
                        },
                        Some(e) => Expr {
                            line: first_term.line,
                            expr_type: ExprType::Binary(op, Box::new(e), Box::new(expr)),
                        },
                    });
                }
                Ok(current_expr.unwrap())
            } // 2 => Ok(Expr {
              //     line: terms.get_unchecked(0).line,
              //     expr_type: ExprType::Binary(BinOp::P)
              // })
        }
        // Ok(Expr {line: 0, expr_type: ExprType::NumLit(3.0)})
    }

    fn parse_mult(&mut self) -> Result<Expr, MathParseError> {
        let first_term = self.parse_factor()?;
        let mut terms: Vec<(BinOp, Expr)> = Vec::new();
        loop {
            match self.current_token() {
                Some(Token {
                    token_value: TokenValue::Times,
                    ..
                }) => {
                    self.advance();
                    terms.push((BinOp::Times, self.parse_factor()?));
                }
                Some(Token {
                    token_value: TokenValue::Slash,
                    ..
                }) => {
                    self.advance();
                    terms.push((BinOp::Slash, self.parse_factor()?));
                }
                _ => break,
            }
        }
        // println!("Ter: {:?}", terms);
        match terms.len() {
            0 => Ok(first_term),
            1 => {
                let (op, expr) = unsafe { terms.get_unchecked(0).clone() };
                Ok(Expr {
                    line: first_term.line,
                    expr_type: ExprType::Binary(op, Box::new(first_term), Box::new(expr)),
                })
            }
            _ => {
                let mut current_expr: Option<Expr> = None;
                for (op, expr) in terms {
                    current_expr = Some(match current_expr {
                        None => Expr {
                            line: first_term.line,
                            expr_type: ExprType::Binary(
                                op,
                                Box::new(first_term.clone()),
                                Box::new(expr),
                            ),
                        },
                        Some(e) => Expr {
                            line: first_term.line,
                            expr_type: ExprType::Binary(op, Box::new(e), Box::new(expr)),
                        },
                    });
                }
                Ok(current_expr.unwrap())
            } // 2 => Ok(Expr {
              //     line: terms.get_unchecked(0).line,
              //     expr_type: ExprType::Binary(BinOp::P)
              // })
        }
    }

    fn parse_factor(&mut self) -> Result<Expr, MathParseError> {
        match self.current_token() {
            Some(Token {
                token_value: TokenValue::LeftParen,
                ..
            }) => {
                self.advance();
                let expr = self.parse_expression();
                match self.current_token() {
                    Some(Token {
                        token_value: TokenValue::RightParen,
                        ..
                    }) => {
                        self.advance();
                        expr
                    }
                    Some(t) => Err(MathParseError::ExpectedButGot(String::from(")"), t)),
                    None => Err(MathParseError::UnexpectedEOF),
                }
            }
            Some(Token {
                token_value: TokenValue::NumberLiteral(lit),
                line,
                ..
            }) => {
                self.advance();
                let num =
                    f64::from_str(&lit).map_err(|err| MathParseError::CouldNotParseFloat(err))?;
                Ok(Expr {
                    line,
                    expr_type: ExprType::NumLit(num),
                })
            }
            Some(Token {
                token_value: TokenValue::Identifier(name),
                ..
            }) => unimplemented!(),
            // Some(t) => panic_because_expected("'(' or number literal", t),
            Some(t) => Err(MathParseError::ExpectedButGot(
                String::from("'(' or number literal"),
                t,
            )),
            None => Err(MathParseError::UnexpectedEOF),
        }
    }
}

#[derive(Debug)]
pub enum MathParseError {
    ScanError,
    ExpectedButGot(String, Token),
    UnexpectedEOF,
    CouldNotParseFloat(ParseFloatError),
    UnknownIdentifier(String),
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
            &MathParseError::UnknownIdentifier(_) => "Unknown identifier",
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

expression = mult {add_op mult} | assignment
mult = factor {mult_op factor}
factor = "(" expression ")" | NUMBER | IDENTIFIER
add_op = "+" | "-"
mult_op = "*" | "/"
assignment = IDENTIFIER "=" expression

*/
