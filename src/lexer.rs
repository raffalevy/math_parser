use std::fmt::{self, Display};

pub fn scan(source: String) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens: Vec<Token> = vec![];
    loop {
        match lexer.scan_token() {
            Some(token) => tokens.push(token),
            None => break,
        }
    }
    tokens.push(Token {
        token_value: TokenValue::EOF,
        lexeme: String::from(""),
        line: lexer.current_line,
    });
    tokens
}

struct Lexer {
    chars: Vec<char>,
    current_line: usize,
    start: usize,
    current: usize,
}

impl Lexer {
    fn new(source: String) -> Self {
        Lexer {
            chars: source.chars().collect(),
            current_line: 1,
            start: 0,
            current: 0,
        }
    }

    fn scan_token(&mut self) -> Option<Token> {
        self.start = self.current;
        if !self.is_at_end() {
            match self.advance() {
                '(' => Some(self.make_token(TokenValue::LeftParen)),
                ')' => Some(self.make_token(TokenValue::RightParen)),
                ',' => Some(self.make_token(TokenValue::Comma)),
                '.' => Some(self.make_token(TokenValue::Period)),
                '+' => Some(self.make_token(TokenValue::Plus)),
                '-' => Some(self.make_token(TokenValue::Minus)),
                '*' => Some(self.make_token(TokenValue::Times)),
                '!' => {
                    let token_value = if self.match_next('=') {
                        TokenValue::BangEqual
                    } else {
                        TokenValue::Bang
                    };
                    Some(self.make_token(token_value))
                }
                '=' => {
                    let token_value = if self.match_next('=') {
                        TokenValue::EqualEqual
                    } else {
                        TokenValue::Equal
                    };
                    Some(self.make_token(token_value))
                }
                ' ' => self.scan_token(),
                '\n' => {
                    self.current_line += 1;
                    self.scan_token()
                }
                '/' => if self.match_next('/') {
                    loop {
                        match self.peek() {
                            Some('\n') | None => break,
                            Some(_) => {
                                self.advance();
                            }
                        }
                    }
                    self.scan_token()
                } else {
                    Some(self.make_token(TokenValue::Slash))
                },
                '"' => self.handle_string(),
                c if is_digit(c) => self.handle_number(),
                c if is_id_start(c) => self.handle_word(),
                c => {
                    eprintln!("Unexpected char '{}'", c);
                    None
                }
            }
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.chars[self.current - 1]
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if self.chars[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.chars[self.current])
        }
    }

    fn peek2(&self) -> Option<char> {
        if self.current + 1 >= self.chars.len() {
            None
        } else {
            Some(self.chars[self.current + 1])
        }
    }

    fn make_token(&self, token_value: TokenValue) -> Token {
        Token {
            token_value,
            lexeme: (&self.chars[self.start..self.current]).iter().collect(),
            line: self.current_line,
        }
    }

    fn handle_string(&mut self) -> Option<Token> {
        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    let string_value: String = self.chars[(self.start + 1)..(self.current - 1)]
                        .iter()
                        .collect();
                    return Some(self.make_token(TokenValue::StringLiteral(string_value)));
                }
                Some('\n') => {
                    self.current_line += 1;
                    self.advance();
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    println!("Unterminated string!");
                    return None;
                }
            }
        }
        #[allow(unreachable_code)]
        self.scan_token()
    }

    fn handle_number(&mut self) -> Option<Token> {
        loop {
            match self.peek() {
                Some(c) if is_digit(c) => {
                    self.advance();
                }
                Some('.') => match self.peek2() {
                    Some(c) if is_digit(c) => {
                        self.advance();
                    }
                    _ => {
                        break;
                    }
                },
                _ => {
                    break;
                } // None => unimplemented!()
            }
        }
        Some(self.make_token(TokenValue::NumberLiteral(
            self.chars[self.start..self.current].iter().collect(),
        )))
    }

    fn handle_word(&mut self) -> Option<Token> {
        loop {
            match self.peek() {
                Some(c) if is_id_char(c) => {self.advance();},
                _ => {break;}
            }
        }
        let text : String = self.chars[self.start..self.current].iter().collect();
        if let Some(keyword_value) = KeywordValue::from(&text) {
            Some(self.make_token(TokenValue::Keyword(keyword_value)))
        } else {
            Some(self.make_token(TokenValue::Identifier(text)))
        }
    }
}

pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub fn is_id_start(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

pub fn is_id_char(c: char) -> bool {
    is_digit(c) || is_id_start(c)
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_value: TokenValue,
    pub lexeme: String,
    pub line: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Token \"{}\", type {:?}", self.lexeme, self.token_value)
    }
}

#[derive(Debug, Clone)]
pub enum TokenValue {
    LeftParen,
    RightParen,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Comma,
    Period,
    Plus,
    Minus,
    Slash,
    Times,

    Identifier(String),
    Keyword(KeywordValue),
    StringLiteral(String),
    NumberLiteral(String),

    True,
    False,

    EOF,
}

#[derive(Debug, Clone)]
pub enum KeywordValue {
    IF,
    FUN,
}

impl KeywordValue {
    fn from(val: &str) -> Option<Self> {
        match val {
            "if" => Some(KeywordValue::IF),
            "fun" => Some(KeywordValue::FUN),
            _ => None,
        }
    }
}
