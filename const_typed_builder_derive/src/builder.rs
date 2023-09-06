use proc_macro2::TokenStream;
use quote::quote;
use syn::{AngleBracketedGenericArguments, FieldsNamed, Ident, PathArguments, Type};

pub(crate) fn builder(config: &Config, fields: &FieldsNamed) -> TokenStream {
    let struct_ = builder_struct(config);
    let impl_ = builder_impl(config, fields);

    quote!(
        #struct_
        #impl_
    )
}

fn builder_struct(config: &Config) -> TokenStream {
    let builder_name = config.builder_name;
    let target_name = config.target_name;
    quote!(
        pub struct #builder_name {
            data: #target_name
        }
    )
}

fn builder_impl(config: &Config, fields: &FieldsNamed) -> TokenStream {
    // let builder_name = config.builder_name;
    // let target_name = config.target_name;

    // let setters = fields.named.iter().map(|field| {
    //     let name = field.ident.as_ref().unwrap();
    //     let ty = &field.ty;

    //     if let Type::Path(ty_path) = ty {
    //         let last_seg = ty_path.path.segments.last().unwrap();
    //         if last_seg.ident.to_string() ==  "Option" {
    //             if let PathArguments::AngleBracketed(args) = last_seg.arguments {
    //                 args.
    //             }
    //         }

    //         let outer = ty_path.path.segments.iter().fold(String::new(), |mut acc, v| {
    //             acc.push_str(&v.ident.to_string());
    //             acc.push(':');
    //             acc
    //         });
    //         let inner_ty = ty_path.qself.as_ref().expect("no inner").ty.as_ref();
    //         if let Type::Path(inner_path) = inner_ty {
    //             let inner = inner_path.path.segments.iter().fold(String::new(), |mut acc, v| {
    //                 acc.push_str(&v.ident.to_string());
    //                 acc.push(':');
    //                 acc
    //             });
    //             panic!("inner {inner}")
    //         }

    //         panic!("outer {outer}", );
    //     } else {
    //         panic!("Not a path");
    //     }

    //     let function_signature = quote!(pub fn #name (self, input: #ty) -> Self);
    //     quote!(
    //         #function_signature {
    //             let mut data = self.data;
    //             data.#name = input;
    //             Self {
    //                 data
    //             }
    //         }
    //     )
    // });
    // quote! {
        // impl #builder_name {
        //     fn new() -> #builder_name {
        //         #builder_name {
        //             data: Default::default()
        //         }
        //     }

    //         #(#setters)*

    //         pub fn build(self) -> #target_name {
    //             self.data
    //         }
    //     }
    // }
    quote!()
}
