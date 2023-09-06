use quote::format_ident;

pub struct DataInfo {
    pub name: syn::Ident,
    pub minimize_option: bool,
}

impl DataInfo {
    pub fn new(ast: &syn::DeriveInput) -> DataInfo {
        DataInfo {
            name: format_ident!("{}Data", &ast.ident),
            minimize_option: false,
        }
    }
}
