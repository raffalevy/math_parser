use std::error::Error;
use lexer::*;
use std::num::ParseFloatError;
use std::fmt::{self, Display};
use std::str::FromStr;
use ast::{self, *};

macro_rules! some_token {
    ($token_value : pat) => {
        Some(Token {
            token_value: $token_value,
            ..
        })
    };
    ($token_value : pat, $l: ident) => {
        Some(Token {
            token_value: $token_value,
            line: $l,
            ..
        })
    };
}

// macro_rules! handle_some_t {
//     ($expected: expr) => {
//         Some(t) => return Err(MathParseError::ExpectedButGot($expected, t))
//     };
// }

// macro_rules! handle_none_eof {
//     () => {
//         None => return Err(MathParseError::UnexpectedEOF)
//     }
// }

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

pub fn parse_file(contents: &str) -> Result<Block, MathParseError> {
    let tokens = scan(String::from(contents));
    let mut parser = Parser::new(tokens);
    parser.parse_file()
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

    fn parse_file(&mut self) -> Result<Block, MathParseError> {
        let mut exprs: Vec<Expr> = Vec::new();
        loop {
            match self.current_token() {
                None
                | Some(Token {
                    token_value: TokenValue::EOF,
                    ..
                }) => {
                    break;
                }
                Some(_) => exprs.push(self.parse_expression()?),
            }
        }
        if exprs.len() > 0 {
            Ok(ast::Block::Exprs(exprs))
        } else {
            Ok(ast::Block::Empty)
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

    fn parse_block(&mut self) -> Result<Block, MathParseError> {
        match self.current_token() {
            some_token!(TokenValue::LeftBracket) => (),
            Some(t) => return Err(MathParseError::ExpectedButGot(String::from("'{'"), t)),
            None => return Err(MathParseError::UnexpectedEOF),
        }

        self.advance();

        let mut exprs: Vec<Expr> = Vec::new();
        loop {
            match self.current_token() {
                some_token!(TokenValue::RightBracket) => {
                    self.advance();
                    break;
                }
                _ => exprs.push(self.parse_expression()?),
            }
        }
        if exprs.len() > 0 {
            Ok(ast::Block::Exprs(exprs))
        } else {
            Ok(ast::Block::Empty)
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, MathParseError> {
        match self.current_token() {
            some_token!(TokenValue::Keyword(KeywordValue::DEF)) => self.parse_funcdef(),
            _ => match self.look_ahead(1) {
                Some(Token {
                    token_value: TokenValue::Equal,
                    ..
                }) => self.parse_assign(),
                _ => self.parse_add(),
            },
        }
    }

    fn parse_add(&mut self) -> Result<Expr, MathParseError> {
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
            }
        }
    }

    fn parse_assign(&mut self) -> Result<Expr, MathParseError> {
        match self.current_token() {
            Some(Token {
                token_value: TokenValue::Identifier(name),
                line,
                ..
            }) => {
                self.advance();
                match self.current_token() {
                    Some(Token {
                        token_value: TokenValue::Equal,
                        ..
                    }) => {
                        self.advance();
                        let expr_value = self.parse_expression()?;
                        Ok(Expr {
                            line,
                            expr_type: ExprType::Assign(String::from(name), Box::new(expr_value)),
                        })
                    }
                    Some(t) => Err(MathParseError::ExpectedButGot(
                        String::from("[identifier]"),
                        t,
                    )),
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

    #[allow(non_shorthand_field_patterns)]
    fn parse_funcdef(&mut self) -> Result<Expr, MathParseError> {
        let line = match self.current_token() {
            some_token!(TokenValue::Keyword(KeywordValue::DEF), line) => line,
            Some(t) => return Err(MathParseError::ExpectedButGot(String::from("'def'"), t)),
            None => return Err(MathParseError::UnexpectedEOF),
        };
        self.advance();
        let name = match self.current_token() {
            some_token!(TokenValue::Identifier(name)) => name,
            Some(t) => {
                return Err(MathParseError::ExpectedButGot(
                    String::from("[identifier]"),
                    t,
                ))
            }
            None => return Err(MathParseError::UnexpectedEOF),
        };
        self.advance();
        match self.current_token() {
            some_token!(TokenValue::LeftParen) => (),
            Some(t) => return Err(MathParseError::ExpectedButGot(String::from("'('"), t)),
            None => return Err(MathParseError::UnexpectedEOF),
        }
        self.advance();

        let mut params: Vec<String> = Vec::new();

        loop {
            match self.current_token() {
                some_token!(TokenValue::Identifier(param)) => {
                    params.push(param);
                    self.advance();
                    match self.current_token() {
                        some_token!(TokenValue::Comma) => {
                            self.advance();
                        },
                        some_token!(TokenValue::RightParen) => break,
                        Some(t) => {
                            return Err(MathParseError::ExpectedButGot(String::from("',' or ')'"), t))
                        }
                        None => return Err(MathParseError::UnexpectedEOF),
                    }
                }
                some_token!(TokenValue::RightParen) => break,
                Some(t) => {
                    return Err(MathParseError::ExpectedButGot(
                        String::from("[identifier]"),
                        t,
                    ))
                }
                None => return Err(MathParseError::UnexpectedEOF),
            }
        }
        self.advance();
        Ok(Expr {
            line,
            expr_type: ExprType::FuncDef(name, params, self.parse_block()?)
        })
    }

    fn parse_mult(&mut self) -> Result<Expr, MathParseError> {
        let first_term = self.parse_exp()?;
        let mut terms: Vec<(BinOp, Expr)> = Vec::new();
        loop {
            match self.current_token() {
                Some(Token {
                    token_value: TokenValue::Times,
                    ..
                }) => {
                    self.advance();
                    terms.push((BinOp::Times, self.parse_exp()?));
                }
                Some(Token {
                    token_value: TokenValue::Slash,
                    ..
                }) => {
                    self.advance();
                    terms.push((BinOp::Slash, self.parse_exp()?));
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

    fn parse_exp(&mut self) -> Result<Expr, MathParseError> {
        let first_term = self.parse_factor()?;
        let mut terms: Vec<(BinOp, Expr)> = Vec::new();
        loop {
            match self.current_token() {
                Some(Token {
                    token_value: TokenValue::Caret,
                    ..
                }) => {
                    self.advance();
                    terms.push((BinOp::Exp, self.parse_factor()?));
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
            }
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
                    Some(t) => Err(MathParseError::ExpectedButGot(String::from("')'"), t)),
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
                line,
                ..
            }) => {
                self.advance();
                match self.current_token() {
                    Some(Token {
                        token_value: TokenValue::LeftParen,
                        ..
                    }) => self.parse_function_starting_at_argument_list(&name, line),
                    _ => Ok(Expr {
                        line,
                        expr_type: ExprType::Var(name),
                    }),
                }
            }
            // Some(t) => panic_because_expected("'(' or number literal", t),
            Some(t) => Err(MathParseError::ExpectedButGot(
                String::from("'(', number literal, or variable"),
                t,
            )),
            None => Err(MathParseError::UnexpectedEOF),
        }
    }

    fn parse_function_starting_at_argument_list(
        &mut self,
        name: &str,
        line: usize,
    ) -> Result<Expr, MathParseError> {
        match self.advance() {
            some_token!(TokenValue::LeftParen) => (),
            Some(t) => return Err(MathParseError::ExpectedButGot(String::from("'('"), t)),
            None => return Err(MathParseError::UnexpectedEOF),
        };
        let mut args: Vec<Expr> = Vec::new();
        loop {
            match self.current_token() {
                some_token!(TokenValue::RightParen) => break,
                _ => {
                    args.push(self.parse_expression()?);
                    match self.current_token() {
                        some_token!(TokenValue::RightParen) => break,
                        some_token!(TokenValue::Comma) => {
                            self.advance();
                        }
                        Some(t) => {
                            return Err(MathParseError::ExpectedButGot(
                                String::from("',' or ')'"),
                                t,
                            ))
                        }
                        None => return Err(MathParseError::UnexpectedEOF),
                    }
                }
            }
        }
        self.advance();
        Ok(Expr {
            line,
            expr_type: ExprType::FuncCall(String::from(name), args),
        })
    }
}

#[derive(Debug)]
pub enum MathParseError {
    ScanError,
    ExpectedButGot(String, Token),
    UnexpectedEOF,
    CouldNotParseFloat(ParseFloatError),
    UnknownIdentifier(String),
    WrongNumberOfArguments(usize, usize)
}

impl Display for MathParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MathParseError::ExpectedButGot(ref e, ref g) => {
                write!(f, "Expected {} but got {:?}", e, g)
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
            &MathParseError::WrongNumberOfArguments(..) => "Wrong number of arguments"
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

block = {expression}
expression = mult {add_op mult} | assignment | fundef
mult = factor {mult_op factor}
factor = "(" expression ")" | NUMBER | IDENTIFIER
add_op = "+" | "-"
mult_op = "*" | "/"
assignment = IDENTIFIER "=" expression

*/
