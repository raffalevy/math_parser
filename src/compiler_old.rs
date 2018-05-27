use std::collections::HashMap;
use std::collections::HashSet;
use ast;
use visitor::Visitor;
use vm::*;
use std::mem::{size_of, transmute};
use std::slice;

pub fn push_bytes_to_vec<T: Copy>(v: &mut Vec<u8>, val: T) {
    let oldlen = v.len();
    v.reserve(size_of::<T>());
    unsafe {
        v.set_len(oldlen + size_of::<T>());
        *transmute::<_, *mut T>(&mut v[oldlen]) = val;
    }
}

fn push_bytes_to_chunk<T: Copy>(v: &mut Chunk, val: T) {
    unsafe {
        let bytes: Vec<ByteCodeUnit> =
            slice::from_raw_parts(transmute::<_, *const u8>(&val), size_of::<T>())
                .iter()
                .map(|b| ByteCodeUnit::Byte(*b))
                .collect();
        v.extend_from_slice(&bytes);
    }
}

pub fn compile(block: &ast::Block) -> Vec<u8> {
    let mut compiler = Compiler::new(block);
    compiler.compile();
    compiler.link()
}

#[derive(Clone)]
enum ByteCodeUnit {
    Byte(u8),
    Const(String),
}

type Chunk = Vec<ByteCodeUnit>;
type Scope = HashMap<String, usize>;

struct Compiler<'t> {
    tree: &'t ast::Block,
    chunks: HashMap<String, Chunk>,
    main_chunk: Option<Chunk>,
    global_scope: Scope,
}

impl<'t> Compiler<'t> {
    fn new(tree: &'t ast::Block) -> Self {
        let mut global_decl_vis = DeclVisitor::new();
        global_decl_vis.visit_block(tree);
        println!("{:?}", global_decl_vis.scope);
        Compiler {
            tree,
            chunks: HashMap::new(),
            main_chunk: None,
            global_scope: global_decl_vis.scope,
        }
    }

    fn compile(&mut self) {
        // let mut main_chunk = Chunk::new();
        let mut cv = CompileVisitor::new(self);
        cv.visit_block(self.tree);
        self.main_chunk = Some(cv.chunk);
    }

    fn link(&self) -> Vec<u8> {
        let mut pgrm = Vec::<u8>::new();
        let mc = self.main_chunk.clone().unwrap();
        let mut a = mc.iter().map(|bcu| match bcu.clone() {
            ByteCodeUnit::Byte(b) => b,
            ByteCodeUnit::Const(_) => unimplemented!()
        }).collect::<Vec<_>>();
        pgrm.append(&mut a);
        pgrm
    }
}

struct DeclVisitor {
    scope: Scope,
    i: usize,
}

impl DeclVisitor {
    fn new() -> Self {
        DeclVisitor {
            scope: Scope::new(),
            i: 0,
        }
    }
}

impl Visitor<()> for DeclVisitor {
    fn visit_block(&mut self, f: &ast::Block) {
        match *f {
            ast::Block::Empty => (),
            ast::Block::Exprs(ref exprs) => for expr in exprs.iter() {
                self.visit_expr(expr);
            },
        }
    }

    fn visit_repltree(&mut self, t: &ast::ReplTree) {
        unimplemented!()
    }

    fn visit_expr(&mut self, e: &ast::Expr) {
        use ast::ExprType;
        match e.expr_type {
            ExprType::NumLit(_) => {}
            ExprType::Var(_) => (),
            ExprType::Binary(_, ref a, ref b) => {
                self.visit_expr(a);
                self.visit_expr(b);
            }
            ExprType::Assign(ref name, ref expr) => {
                self.scope.insert(name.clone(), self.i);
                self.i += 1;
            }
            ExprType::FuncCall(_, ref args) => {
                for arg in args.iter() {
                    self.visit_expr(arg);
                }
            }
            _ => unimplemented!(),
        }
    }
}

struct CompileVisitor {
    chunk: Chunk,
}

impl CompileVisitor {
    fn new(compiler: &mut Compiler) -> Self {
        CompileVisitor {
            chunk: Chunk::new(),
        }
    }
}

impl Visitor<()> for CompileVisitor {
    fn visit_block(&mut self, b: &ast::Block) {
        match *b {
            ast::Block::Empty => (),
            ast::Block::Exprs(ref exprs) => for expr in exprs.iter() {
                self.visit_expr(expr);
                self.chunk.push(ByteCodeUnit::Byte(POP_F64))
            },
        }
    }

    fn visit_repltree(&mut self, t: &ast::ReplTree) {
        unimplemented!()
    }

    fn visit_expr(&mut self, e: &ast::Expr) {
        use ast::ExprType;
        match e.expr_type {
            ExprType::NumLit(num) => {
                self.chunk.push(ByteCodeUnit::Byte(CONST_F64));
                push_bytes_to_chunk(&mut self.chunk, num);
            }
            ExprType::Var(_) => (),
            ExprType::Binary(_, ref a, ref b) => {
                self.visit_expr(a);
                self.visit_expr(b);
                self.chunk.push(ByteCodeUnit::Byte(ADD_F64));
            }
            ExprType::FuncCall(ref name, ref exprs) if (name == "print") && (exprs.len() == 1) => {
                self.visit_expr(unsafe { exprs.get_unchecked(0) });
                self.chunk.push(ByteCodeUnit::Byte(PRINT_F64));
            }
            ExprType::Assign(ref name, ref expr) => {}
            _ => unimplemented!(),
        }
    }
}

// pub fn compile(block: &ast::Block) -> Vec<u8> {
//     let mut program: Vec<u8> = Vec::new();
//     {
//         let mut visitor = CompileVisitor {
//             program: &mut program,
//             var_stack: Vec::new(),
//             const_pool: ConstPool::new(),
//             funcs_stack: Vec::new()
//         };
//         visitor.visit_block(block);
//     }
//     program
// }

// #[derive(Debug)]
// struct ConstPool {
//     func_indexes: HashMap<String, usize>,
//     func_count: usize,
// }

// impl ConstPool {
//     fn new() -> Self {
//         ConstPool {
//             func_indexes: HashMap::new(),
//             func_count: 0,
//         }
//     }

//     fn add_func(&mut self, name: &str) -> usize {
//         let ind = self.func_count;
//         self.func_count += 1;
//         self.func_indexes.insert(String::from(name), ind);
//         ind
//     }

//     fn link(&self, program: &mut Vec<u8>, funcs: &mut [Vec<u8>]) {
//         let mut indices: Vec<usize> = Vec::with_capacity(self.func_count);
//         let mut i = 0;
//         for func in funcs {
//             let ind = program.len();
//             program.append(func);
//             indices[i] = ind;
//             i += 1;
//         }
//         let mut j: usize = 0;
//         loop {
//             if program[j] == CALL {
//                 unsafe {
//                     let cons: usize = *transmute::<_, *const usize>(&program[j + 1]);
//                     *transmute::<_, *mut usize>(&mut program[j + 1]) = indices[cons];
//                 }
//             }
//             j += 1;
//         }
//     }
// }

// pub fn push_bytes_to_vec<T: Copy>(v: &mut Vec<u8>, val: T) {
//     let oldlen = v.len();
//     v.reserve(size_of::<T>());
//     unsafe {
//         v.set_len(oldlen + size_of::<T>());
//         *transmute::<_, *mut T>(&mut v[oldlen]) = val;
//     }
// }

// struct CompileVisitor<'p> {
//     program: &'p mut Vec<u8>,
//     var_stack: Vec<HashMap<String, usize>>,
//     const_pool: ConstPool,
//     funcs_stack: Vec<Vec<Vec<u8>>>,
// }

// impl<'p> Visitor<()> for CompileVisitor<'p> {
//     fn visit_block(&mut self, f: &ast::Block) {
//         match *f {
//             ast::Block::Exprs(ref exprs) => {
//                 // let mut first = true;
//                 let mut alloc_visitor = AllocVisitor {
//                     vars: HashSet::new(),
//                 };
//                 alloc_visitor.visit_block(f);
//                 let mut vars: HashMap<String, usize> = HashMap::new();
//                 let mut i: usize = 0;
//                 for var in alloc_visitor.vars.iter() {
//                     vars.insert(var.clone(), i);
//                     i += 1;
//                 }
//                 let varlen = vars.len();
//                 self.var_stack.push(vars);
//                 self.program.push(SET_CTX);
//                 for i in 0..varlen {
//                     self.program.push(CONST_0_64);
//                 }
//                 {
//                     let mut funcdef_visitor = FuncDefVisitor::new(&mut self.const_pool);
//                     funcdef_visitor.visit_block(f);
//                 }
//                 for expr in exprs.iter() {
//                     // if first {
//                     //     first = false;
//                     // } else {
//                     //     self.program.push(POP_F64);
//                     // }
//                     self.visit_expr(expr);
//                     self.program.push(POP_F64);
//                 }
//                 for i in 0..varlen {
//                     self.program.push(POP_F64);
//                 }
//                 self.program.push(RET_CTX);
//                 self.var_stack.pop();
//                 self.program.push(EXIT);
//                 // self.const_pool.link(self.program, funcs);
//             }
//             ast::Block::Empty => self.program.push(NOP),
//         }
//     }

//     fn visit_repltree(&mut self, t: &ast::ReplTree) {
//         unimplemented!()
//     }

//     #[allow(unconditional_recursion)]
//     fn visit_expr(&mut self, e: &ast::Expr) {
//         match e.expr_type {
//             ast::ExprType::Binary(ast::BinOp::Plus, ref a, ref b) => {
//                 self.visit_expr(a);
//                 self.visit_expr(b);
//                 self.program.push(ADD_F64);
//             }
//             ast::ExprType::Binary(ast::BinOp::Minus, ref a, ref b) => {
//                 self.visit_expr(a);
//                 self.visit_expr(b);
//                 self.program.push(SUB_F64);
//             }
//             ast::ExprType::Binary(ast::BinOp::Times, ref a, ref b) => {
//                 self.visit_expr(a);
//                 self.visit_expr(b);
//                 self.program.push(MUL_F64);
//             }
//             ast::ExprType::Binary(ast::BinOp::Slash, ref a, ref b) => {
//                 self.visit_expr(a);
//                 self.visit_expr(b);
//                 self.program.push(DIV_F64);
//             }
//             ast::ExprType::NumLit(n) => {
//                 self.program.push(CONST_F64);
//                 let bytes = &n as *const f64 as *const [u8; 8];
//                 self.program
//                     .extend_from_slice(unsafe { bytes.as_ref() }.unwrap());
//             }
//             ast::ExprType::FuncDef(ref name, ref params, ref block) => {

//             }
//             ast::ExprType::FuncCall(ref name, ref args) if (name == "print" && args.len() == 1) => {
//                 self.visit_expr(unsafe { args.get_unchecked(0) });
//                 self.program.push(PRINT_F64);
//             }
//             ast::ExprType::FuncCall(ref name, ref args) => {
//                 match self.const_pool.func_indexes.get(name) {
//                     Some(ind) => {
//                         self.program.push(CALL);
//                         push_bytes_to_vec(self.program, *ind);
//                     }
//                     None => panic!("Unknown function")
//                 };
//             }
//             ast::ExprType::Assign(ref name, ref expr) => {
//                 let oind = self.var_stack.last().unwrap().get(name).map(|x| *x);
//                 match oind {
//                     Some(vind) => {
//                         self.visit_expr(expr);
//                         self.program.push(STORE_F64_U8);
//                         self.program.push(vind as u8);
//                     }
//                     None => panic!("unknown variable"),
//                 };
//             }
//             ast::ExprType::Var(ref name) => {
//                 let oind = self.var_stack.last().unwrap().get(name).map(|x| *x);
//                 match oind {
//                     Some(vind) => {
//                         self.program.push(LOAD_F64_U8);
//                         self.program.push(vind as u8);
//                     }
//                     None => panic!("unknown variable"),
//                 };
//             }
//             _ => unimplemented!(),
//         }
//     }
// }

// struct AllocVisitor {
//     vars: HashSet<String>,
// }
// impl Visitor<()> for AllocVisitor {
//     fn visit_block(&mut self, f: &ast::Block) {
//         match *f {
//             ast::Block::Exprs(ref exprs) => for expr in exprs.iter() {
//                 self.visit_expr(expr);
//             },
//             ast::Block::Empty => (),
//         }
//     }

//     fn visit_repltree(&mut self, t: &ast::ReplTree) {
//         unimplemented!()
//     }

//     fn visit_expr(&mut self, e: &ast::Expr) {
//         use ast::ExprType::*;
//         match e.expr_type {
//             NumLit(_) => (),
//             Var(_) => (),
//             FuncCall(_, ref exprs) => for expr in exprs.iter() {
//                 self.visit_expr(expr);
//             },
//             FuncDef(..) => (),
//             Binary(_, ref a, ref b) => {
//                 self.visit_expr(a);
//                 self.visit_expr(b);
//             }
//             Assign(ref name, ref expr) => {
//                 self.vars.insert(name.clone());
//                 self.visit_expr(expr);
//             }
//             // _ => unimplemented!(),
//         }
//     }
// }

// struct FuncDefVisitor<'p> {
//     const_pool: &'p mut ConstPool,
// }

// impl<'p> FuncDefVisitor<'p> {
//     fn new(const_pool: &'p mut ConstPool) -> Self {
//         FuncDefVisitor {
//             const_pool,
//         }
//     }
// }

// impl<'p> Visitor<()> for FuncDefVisitor<'p> {
//     fn visit_block(&mut self, f: &ast::Block) {
//         match *f {
//             ast::Block::Exprs(ref exprs) => for expr in exprs.iter() {
//                 self.visit_expr(expr);
//             },
//             ast::Block::Empty => (),
//         }
//     }

//     fn visit_repltree(&mut self, t: &ast::ReplTree) {
//         unimplemented!()
//     }

//     fn visit_expr(&mut self, e: &ast::Expr) {
//         use ast::ExprType::*;
//         match e.expr_type {
//             NumLit(_) => (),
//             Var(_) => (),
//             FuncCall(_, ref exprs) => for expr in exprs.iter() {
//                 self.visit_expr(expr);
//             },
//             Binary(_, ref a, ref b) => {
//                 self.visit_expr(a);
//                 self.visit_expr(b);
//             }
//             Assign(_, ref expr) => self.visit_expr(expr),
//             FuncDef(ref name, ref params, ref block) => {
//                 self.const_pool.add_func(name);
//             }
//             // _ => unimplemented!(),
//         }
//     }
// }
