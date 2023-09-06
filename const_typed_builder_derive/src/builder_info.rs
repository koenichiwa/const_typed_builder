use quote::format_ident;

pub struct BuilderInfo {
    pub name: syn::Ident,
}

impl BuilderInfo {
    pub fn new(ast: &syn::DeriveInput) -> BuilderInfo {
        BuilderInfo {
            name: format_ident!("{}Builder", &ast.ident),
        }
    }
}
