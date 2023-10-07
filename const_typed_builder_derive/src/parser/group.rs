use proc_macro_error::emit_error;

use crate::{info, symbol};

pub struct Group;

impl Group {
    pub fn parse(attr: &syn::Attribute) -> Option<info::Group> {
        let mut group = None;
        attr.parse_nested_meta(|meta| {
            let group_name = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(
                        &meta.path , "Group name is not specified correctly";
                        help = "Try to define it like `#[{}(foo = {}(1))]`", symbol::GROUP, symbol::AT_LEAST;
                        note = err
                    );
                    return Ok(());
                }
            };

            let group_type = match meta.value()?.parse()? {
                syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
                    let group_type = match func.as_ref() {
                        syn::Expr::Path(syn::ExprPath { path, .. }) => match path.require_ident() {
                            Ok(ident) => ident,
                            Err(err) => {
                                emit_error!(
                                    &meta.path , "Group type is not specified correctly";
                                    help = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST;
                                    note = err
                                );
                                return Ok(());
                            }
                        },
                        _ => {
                            emit_error!(
                                &attr.meta, "No group type specified";
                                help = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST
                            );
                            return Ok(());
                        }
                    };

                    match args.first() {
                        Some(syn::Expr::Lit(syn::ExprLit {
                            attrs: _,
                            lit: syn::Lit::Int(val),
                        })) => match val.base10_parse::<usize>() {
                            Ok(group_args) => match (&group_type.to_string()).into() {
                                symbol::EXACT => info::GroupType::Exact(group_args),
                                symbol::AT_LEAST => info::GroupType::AtLeast(group_args),
                                symbol::AT_MOST => info::GroupType::AtMost(group_args),
                                symbol::SINGLE => {
                                    emit_error!(
                                        args,
                                        "`{}` doesn't take any arguments", symbol::SINGLE;
                                        help = "`{}` is shorthand for {}(1)", symbol::SINGLE, symbol::EXACT
                                    );
                                    return Ok(());
                                }
                                _ => {
                                    emit_error!(
                                        group_type, "Unknown group type";
                                        help = "Known group types are {}, {} and {}", symbol::EXACT, symbol::AT_LEAST, symbol::AT_MOST
                                    );
                                    return Ok(());
                                }
                            },
                            Err(err) => {
                                emit_error!(
                                    val, "Couldn't parse group argument";
                                    note = err
                                );
                                return Ok(());
                            }
                        },

                        _ => {
                            emit_error!(func, "Can't parse group argument");
                            return Ok(());
                        }
                    }
                }
                syn::Expr::Path(syn::ExprPath { path, .. }) => {
                    let group_type = match path.require_ident() {
                        Ok(ident) => ident,
                        Err(err) => {
                            emit_error!(
                                &meta.path , "Group type is not specified correctly";
                                help = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST;
                                note = err
                            );
                            return Ok(());
                        }
                    };
                    match (&group_type.to_string()).into() {
                        symbol::EXACT | symbol::AT_LEAST | symbol::AT_MOST => {
                            emit_error!(
                                &attr.meta,
                                "Missing arguments for group type";
                                help = "Try `{}(1)`, or any other usize", &group_type
                            );
                            return Ok(());
                        }
                        symbol::SINGLE => info::GroupType::Exact(1),
                        _ => {
                            emit_error!(
                                group_type,
                                "Unknown group type";
                                help = "Known group types are {}, {} and {}", symbol::EXACT, symbol::AT_LEAST, symbol::AT_MOST
                            );
                            return Ok(());
                        }
                    }
                }
                _ => {
                    emit_error!(
                        &attr.meta, "No group type specified";
                        hint = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST
                    );
                    return Ok(());
                }
            };
            group = Some(info::Group::new(group_name.clone(), group_type));

            Ok(())
        })
        .unwrap_or_else(|err| emit_error!(
            &attr.meta, "Unknown error";
            note = err
        ));
        group
    }
}
