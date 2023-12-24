use crate::{
    info::{Group, GroupCollection, GroupType},
    symbol::Symbol,
};
use proc_macro_error::emit_error;
use std::str::FromStr;

pub struct GroupParser<'a> {
    groups: &'a mut GroupCollection,
}

impl<'a> GroupParser<'a> {
    pub fn new(groups: &'a mut GroupCollection) -> Self {
        Self { groups }
    }

    pub fn parse(self, attr: &syn::Attribute) {
        attr.parse_nested_meta(|meta| {
            let group_name = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(
                        &meta.path , "Group name is not specified correctly";
                        help = "Try to define it like `#[{}(foo = {}(1))]`", Symbol::Group, Symbol::AtLeast;
                        note = err
                    );
                    return Ok(());
                }
            };

            let group_type = match meta.value()?.parse()? {
                syn::Expr::Call(expr) => self.handle_group_call(&expr),
                syn::Expr::Path(expr) => self.handle_group_path(&expr),
                _ => {
                    emit_error!(
                        &attr.meta, "Can't parse group type";
                        hint = "Try to define it like `#[groups({} = {}(1))]`", group_name, Symbol::AtLeast
                    );
                    return Ok(());
                }
            };

            if let Some(group_type) = group_type {
                if let Some(earlier_definition) = self.groups.insert(group_name.to_string(), Group::new(group_name.clone(), group_type)) {
                    let earlier_span = earlier_definition.ident().span();
                    emit_error!(
                        &group_name, "Group defined multiple times";
                        help = earlier_span => "Also defined here"
                    );
                }
            }

            Ok(())
        })
        .unwrap_or_else(|err| emit_error!(
            &attr.meta, "Error occured while parsing group";
            note = err
        ));
    }

    fn handle_group_call(&self, expr: &syn::ExprCall) -> Option<GroupType> {
        let syn::ExprCall { func, args, .. } = expr;

        let type_ident = match func.as_ref() {
            syn::Expr::Path(syn::ExprPath { path, .. }) => match path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(
                        &expr , "Group type is not specified correctly";
                        help = "Try to define it like `#[{}(foo = {}(1))]`", Symbol::Group, Symbol::AtLeast;
                        note = err
                    );
                    return None;
                }
            },
            _ => {
                emit_error!(
                    &expr, "No group type specified";
                    help = "Try to define it like `#[{}(foo = {}(1))]`", Symbol::Group, Symbol::AtLeast
                );
                return None;
            }
        };

        if args.len() != 1 {
            emit_error!(func, "Group needs exactly one integer literal as argument");
            return None;
        }

        let group_argument = match args.first().unwrap() {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(val),
                ..
            }) => val.base10_parse::<usize>().ok(),
            _ => None,
        };

        let group_argument = match group_argument {
            Some(lit) => lit,
            None => {
                emit_error!(args, "Can't parse argument");
                return None;
            }
        };

        match Symbol::from_str(&type_ident.to_string()) {
            Ok(symbol) => match symbol {
                Symbol::AtLeast => Some(GroupType::AtLeast(group_argument)),
                Symbol::AtMost => Some(GroupType::AtMost(group_argument)),
                Symbol::Exact => Some(GroupType::Exact(group_argument)),
                Symbol::Single => {
                    emit_error!(
                        args,
                        "`{}` is the only group type that doesn't take any arguments", Symbol::Single;
                        help = "`{}` is shorthand for {}(1)", Symbol::Single, Symbol::Exact
                    );
                    None
                }
                symbol => {
                    emit_error!(
                        type_ident, format!("{symbol} is an unknown group type");
                        help = "Known group types are {}, {} and {}", Symbol::Single, Symbol::AtLeast, Symbol::AtMost
                    );
                    None
                }
            },
            Err(err) => {
                emit_error!(
                    &type_ident, "Unknown specifier";
                    note = err
                );
                None
            }
        }
    }

    fn handle_group_path(&self, expr: &syn::ExprPath) -> Option<GroupType> {
        let syn::ExprPath { path, .. } = expr;
        let type_ident = match path.require_ident() {
            Ok(ident) => ident,
            Err(err) => {
                emit_error!(
                    &expr , "Group type is not specified correctly";
                    help = "Try to define it like `#[{}(foo = {}(1))]`", Symbol::Group, Symbol::AtLeast;
                    note = err
                );
                return None;
            }
        };
        match Symbol::from_str(&type_ident.to_string()) {
            Ok(symbol) => match symbol {
                Symbol::Single => Some(GroupType::Exact(1)),
                Symbol::Exact | Symbol::AtLeast | Symbol::AtMost => {
                    emit_error!(
                        &expr,
                        "Missing arguments for group type";
                        help = "Try `{}(1)`, or any other usize", &type_ident
                    );
                    None
                }
                symbol => {
                    emit_error!(
                        type_ident, format!("{symbol} is an unknown group type");
                        help = "Known group types are {}, {} and {}", Symbol::Single, Symbol::AtLeast, Symbol::AtMost
                    );
                    None
                }
            },
            Err(err) => {
                emit_error!(
                    &type_ident, "Unknown specifier";
                    note = err
                );
                None
            }
        }
    }
}
