use rnix::{
    ast::{Expr, Ident},
    SyntaxKind, SyntaxToken,
};

pub fn str_to_js(nix_expr: &str) -> Result<String, ()> {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    ast_to_js(&root_expr)
}

pub fn ast_to_js(nix_ast: &Expr) -> Result<String, ()> {
    match nix_ast {
        Expr::Ident(ident) => ident_to_js(&ident),
        _ => panic!("Not implemented: {:?}", nix_ast),
    }
}

fn ident_to_js(ident: &Ident) -> Result<String, ()> {
    let token = ident.ident_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_IDENT => ident_token_to_js(&token),
        // Expr::UnaryOp(unary_op) => eval_unary_op(unary_op),
        _ => todo!(),
    }
}

fn ident_token_to_js(token: &SyntaxToken) -> Result<String, ()> {
    Ok(match token.text() {
        "true" => "true".to_owned(),
        "false" => "false".to_owned(),
        _ => todo!(),
    })
}
