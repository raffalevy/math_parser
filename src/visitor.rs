use ast;

pub trait Visitor<T> {
    fn visit_block(&mut self, f: &ast::Block) -> T;

    fn visit_repltree(&mut self, t: &ast::ReplTree) -> T;

    fn visit_expr(&mut self, e: &ast::Expr) -> T;
}

// fn walk_block(v: &mut Visitor<()>, f: &ast::Block) {
//     match *f {
//         ast::Block
//     }
// }