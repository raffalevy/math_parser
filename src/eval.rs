use std::collections::HashMap;
use parser::MathParseError;
use visitor::Visitor;
use ast;

use std::f64;

pub type EvalResult = Result<Option<f64>, MathParseError>;

pub struct EvalContext {
    stack: Vec<StackFrame>,
}

#[derive(Debug)]
pub struct StackFrame {
    vars: HashMap<String, f64>,
    funcs: HashMap<String, (Vec<String>, ast::Block)>,
}

impl EvalContext {
    pub fn new() -> Self {
        EvalContext {
            stack: vec![StackFrame::new()],
        }
    }

    pub fn eval_file(&mut self, f: &ast::Block) -> EvalResult {
        let mut visitor = EvalVisitor { context: self };
        visitor.visit_block(f)
    }

    pub fn eval_repltree(&mut self, t: &ast::ReplTree) -> EvalResult {
        let mut visitor = EvalVisitor { context: self };
        visitor.visit_repltree(t)
    }

    fn current_stack_frame(&self) -> &StackFrame {
        self.stack.last().unwrap()
    }

    fn current_stack_frame_mut(&mut self) -> &mut StackFrame {
        self.stack.last_mut().unwrap()
    }

    fn assign_var(&mut self, name: &str, value: f64) {
        self.current_stack_frame_mut()
            .vars
            .insert(String::from(name), value);
    }

    fn get_var(&self, name: &str) -> Option<f64> {
        self.current_stack_frame().vars.get(name).map(|x| *x)
    }
}

impl StackFrame {
    pub fn new() -> Self {
        let mut c = StackFrame {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        };
        c.vars.insert(String::from("pi"), f64::consts::PI);
        c.vars.insert(String::from("e"), f64::consts::E);
        c
    }
}

struct EvalVisitor<'a> {
    context: &'a mut EvalContext,
}

impl<'a> Visitor<EvalResult> for EvalVisitor<'a> {
    fn visit_block(&mut self, f: &ast::Block) -> EvalResult {
        match *f {
            ast::Block::Exprs(ref exprs) => {
                let mut lastres: Option<f64> = None;
                for expr in exprs.iter() {
                    lastres = self.visit_expr(expr)?;
                }
                Ok(lastres)
            }
            ast::Block::Empty => Ok(None),
        }
    }

    fn visit_repltree(&mut self, t: &ast::ReplTree) -> EvalResult {
        match *t {
            ast::ReplTree::Expr(ref expr) => self.visit_expr(expr),
            ast::ReplTree::Empty => Ok(None),
        }
    }

    fn visit_expr(&mut self, e: &ast::Expr) -> EvalResult {
        match e.expr_type {
            ast::ExprType::Binary(ast::BinOp::Plus, ref expr1, ref expr2) => {
                match (self.visit_expr(expr1)?, self.visit_expr(expr2)?) {
                    (Some(a), Some(b)) => Ok(Some(a + b)),
                    _ => Ok(None),
                }
            }
            ast::ExprType::Binary(ast::BinOp::Minus, ref expr1, ref expr2) => {
                match (self.visit_expr(expr1)?, self.visit_expr(expr2)?) {
                    (Some(a), Some(b)) => Ok(Some(a - b)),
                    _ => Ok(None),
                }
            }
            ast::ExprType::Binary(ast::BinOp::Slash, ref expr1, ref expr2) => {
                match (self.visit_expr(expr1)?, self.visit_expr(expr2)?) {
                    (Some(a), Some(b)) => Ok(Some(a / b)),
                    _ => Ok(None),
                }
            }
            ast::ExprType::Binary(ast::BinOp::Times, ref expr1, ref expr2) => {
                match (self.visit_expr(expr1)?, self.visit_expr(expr2)?) {
                    (Some(a), Some(b)) => Ok(Some(a * b)),
                    _ => Ok(None),
                }
            }
            ast::ExprType::Binary(ast::BinOp::Exp, ref expr1, ref expr2) => {
                match (self.visit_expr(expr1)?, self.visit_expr(expr2)?) {
                    (Some(a), Some(b)) => Ok(Some(a.powf(b))),
                    _ => Ok(None),
                }
            }
            ast::ExprType::NumLit(n) => Ok(Some(n)),
            ast::ExprType::Assign(ref name, ref expr) => {
                let val = self.visit_expr(expr);
                if let Ok(Some(val)) = val {
                    self.context.assign_var(name, val)
                }
                val
            }
            ast::ExprType::Var(ref name) => match self.context.get_var(name) {
                Some(val) => Ok(Some(val)),
                None => Err(MathParseError::UnknownIdentifier(name.clone())),
            },
            ast::ExprType::FuncDef(ref name, ref params, ref block) => {
                self.eval_funcdef(name, params, block)
            }
            ast::ExprType::FuncCall(ref name, ref args) => self.eval_function(name, args),
            // _ => unimplemented!(),
        }
    }
}

impl<'a> EvalVisitor<'a> {
    fn eval_funcdef(&mut self, name: &str, params: &Vec<String>, block: &ast::Block) -> EvalResult {
        self.context
            .current_stack_frame_mut()
            .funcs
            .insert(name.to_string(), (params.clone(), block.clone()));
        Ok(Some(0.0))
    }

    fn eval_function(&mut self, name: &str, args: &Vec<ast::Expr>) -> EvalResult {
        match name {
            "sin" => {
                if args.len() == 1 {
                    match self.visit_expr(unsafe { args.get_unchecked(0) }) {
                        Ok(Some(x)) => Ok(Some(x.sin())),
                        _ => unimplemented!(),
                    }
                } else {
                    unimplemented!()
                }
            }
            "cos" => {
                if args.len() == 1 {
                    match self.visit_expr(unsafe { args.get_unchecked(0) }) {
                        Ok(Some(x)) => Ok(Some(x.cos())),
                        _ => unimplemented!(),
                    }
                } else {
                    unimplemented!()
                }
            }
            "sqrt" => {
                if args.len() == 1 {
                    match self.visit_expr(unsafe { args.get_unchecked(0) }) {
                        Ok(Some(x)) => Ok(Some(x.sqrt())),
                        _ => unimplemented!(),
                    }
                } else {
                    unimplemented!()
                }
            }
            _ => {
                let pb = self.context.current_stack_frame().funcs.get(name);
                match pb {
                    Some(&(ref params, ref block)) => {
                        let _self = unsafe {
                            let __self: *const Self = self;
                            let ___self = __self as *mut Self;
                            ___self.as_mut().unwrap()
                        };
                        let plen = params.len();
                        let alen = args.len();
                        if plen == alen {
                            let mut sf = StackFrame::new();
                            for (i, arg) in args.iter().enumerate() {
                                match _self.visit_expr(arg)? {
                                    Some(val) => sf.vars.insert(params[i].to_string(), val),
                                    None => unimplemented!(),
                                };
                            }
                            _self.context.stack.push(sf);
                            let res = _self.visit_block(block);
                            // println!("{:#?}", _self.context.stack);
                            _self.context.stack.pop();
                            res
                        } else {
                            Err(MathParseError::WrongNumberOfArguments(plen, alen))
                        }
                    }
                    None => Err(MathParseError::UnknownIdentifier(String::from(name))),
                }
            }
        }
    }
}
