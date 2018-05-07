#[derive(Debug, Clone)]
pub enum ReplTree {
    Expr(Expr),
    Empty
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub line: usize,
    pub expr_type: ExprType,
}

#[derive(Debug, Clone)]
pub enum ExprType {
    Binary(BinOp, Box<Expr>, Box<Expr>),
    NumLit(f64),
    Var(String),
    Assign(String, Box<Expr>)
}

#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Slash,
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
