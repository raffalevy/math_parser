use visitor::Visitor;
use std::rc::Rc;
use std::collections::HashMap;
use ast;
use std::mem::{size_of, transmute};
use vm;

type Chunk = Vec<BCUnit>;

pub fn compile(block: &ast::Block) -> Vec<u8> {
    let main = Func::compile(block, &Vec::new());
    unimplemented!()
}

#[derive(Copy, Clone, Debug)]
enum BCUnit {
    Byte(u8),
    Float64(f64),
    FuncConst(usize),
}

const USIZE_SIZE: usize = size_of::<usize>();

impl BCUnit {
    fn push_to_vec(self, v: &mut Vec<u8>) {
        use self::BCUnit::*;
        match self {
            Byte(b) => v.push(b),
            Float64(f) => unsafe {
                let oldlen = v.len();
                v.reserve(8);
                v.set_len(oldlen + 8);
                *transmute::<_, *mut f64>(&mut v[oldlen]) = f;
            },
            FuncConst(i) => unsafe {
                let oldlen = v.len();
                v.reserve(USIZE_SIZE);
                v.set_len(oldlen + USIZE_SIZE);
                *transmute::<_, *mut usize>(&mut v[oldlen]) = i;
            },
        }
    }
}

#[derive(Debug)]
struct Scope {
    vars: HashMap<String, usize>,
    varsc: usize,
    funcs: HashMap<String, usize>,
    funcsc: usize,
}

impl Scope {
    fn new() -> Self {
        Scope {
            vars: HashMap::new(),
            varsc: 0,
            funcs: HashMap::new(),
            funcsc: 0,
        }
    }

    fn add_var(&mut self, name: &str) {
        if let None = self.vars.insert(String::from(name), self.varsc) {
            self.varsc += 1;
        }
    }

    fn add_func(&mut self, name: &str) {
        if let None = self.vars.insert(String::from(name), self.funcsc) {
            self.funcsc += 1;
        }
    }
}

#[derive(Debug)]
struct Func {
    scope: Scope,
    chunk: Chunk,
}

impl Func {
    fn compile(funcs: Vec<Func>, b: &ast::Block, args: &Vec<String>) -> Self {
        let mut this = Func {
            scope: Scope::new(),
            chunk: Chunk::new(),
        };
        this.visit_block(b);
        this
    }
}

impl Visitor<bool> for Func {
    fn visit_block(&mut self, f: &ast::Block) -> bool {
        match *f {
            ast::Block::Exprs(ref exprs) if exprs.len() == 0 => false,
            ast::Block::Exprs(ref exprs) => {
                let mut should_pop = false;
                for expr in exprs.iter() {
                    if should_pop {
                        self.chunk.push(BCUnit::Byte(vm::POP_F64));
                    }
                    should_pop = self.visit_expr(expr);
                }
                should_pop
            }
            ast::Block::Empty => false,
        }
    }

    fn visit_repltree(&mut self, t: &ast::ReplTree) -> bool {
        unimplemented!()
    }

    fn visit_expr(&mut self, e: &ast::Expr) -> bool {
        use ast::ExprType;

        match e.expr_type {
            ExprType::NumLit(num) => {
                self.chunk.push(BCUnit::Byte(vm::CONST_F64));
                self.chunk.push(BCUnit::Float64(num));
                true
            }
            ExprType::Binary(ref op, ref a, ref b) => {
                self.visit_expr(a);
                self.visit_expr(b);
                self.chunk.push(BCUnit::Byte(vm::ADD_F64));
                true
            }
            ExprType::Var(_) => unimplemented!(),
            ExprType::FuncCall(..) | ExprType::FuncDef(..) | ExprType::Assign(..) => unimplemented!()
        }
    }
}