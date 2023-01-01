#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate syn;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::visit::{self, Visit};
use syn::{DataStruct, File, GenericArgument, ItemFn, ItemStruct, Token};

struct StructVisitor {
    pub struct_defs: Vec<StructDef>,
}

impl Default for StructVisitor {
    fn default() -> Self {
        StructVisitor {
            struct_defs: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for StructVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let struct_name = node.ident.to_owned();
        let mut struct_fields: Vec<StructField> = Vec::new();

        let fields = node.fields.clone();
        match fields {
            syn::Fields::Named(names) => {
                let names = names.named;
                for field in names.iter() {
                    let field_name = field.ident.to_owned().unwrap().to_string();
                    match &field.ty {
                        syn::Type::Path(path) => {
                            let path = path.path.clone();
                            let mut segs = path.segments.into_iter();
                            let field_ty = segs.next().unwrap();
                            let field_ty_ident = field_ty.ident.to_owned().to_string();

                            if field_ty_ident == "Vec" {
                                let inner_ty = field_ty.arguments.to_owned();
                                match inner_ty {
                                    syn::PathArguments::AngleBracketed(args) => {
                                        let generic = args.args.last().unwrap();
                                        if let GenericArgument::Type(ty) = generic {
                                            match ty {
                                                syn::Type::Path(path) => {
                                                    let path = path.path.to_owned();
                                                    let mut segs = path.segments.into_iter();
                                                    let inner_ty = segs.next().unwrap();
                                                    let inner_ty_ident =
                                                        inner_ty.ident.to_owned().to_string();

                                                    struct_fields.push(StructField {
                                                        name: field_name,
                                                        ty: Type::Vec(Box::new(Type::Atom(
                                                            inner_ty_ident,
                                                        ))),
                                                    })
                                                }
                                                _ => panic!("The generic type is illegal!"),
                                            }
                                        } else {
                                            panic!("The generic is not a type!")
                                        }
                                    }
                                    _ => panic!("Not providing the Vec generic!"),
                                }
                            } else {
                                struct_fields.push(StructField {
                                    name: field_name,
                                    ty: Type::Atom(field_ty_ident),
                                });
                            }
                        }
                        _ => panic!("The field type is illegal!"),
                    }
                }
            }
            _ => (),
        };

        self.struct_defs.push(StructDef {
            name: struct_name.to_string(),
            fields: struct_fields,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Vec(Box<Type>),
    Atom(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructField {
    name: String,
    ty: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructDef {
    name: String,
    fields: Vec<StructField>,
}

pub fn get_struct_defs_from_file(file: &File) -> Vec<StructDef> {
    let mut visitor = StructVisitor::default();
    visitor.visit_file(file);
    visitor.struct_defs
}

pub fn get_struct_defs(token_stream: TokenStream) -> Vec<StructDef> {
    let file: File = syn::parse2(token_stream).unwrap();
    get_struct_defs_from_file(&file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_struct_defs_one() {
        let code = quote! {
            struct Person {
                name: String,
                age: u8
            }
        };

        let struct_defs = get_struct_defs(code);
        let struct_def = struct_defs.first().unwrap();
        assert_eq!(
            *struct_def,
            StructDef {
                name: "Person".to_string(),
                fields: vec![
                    StructField {
                        name: "name".to_string(),
                        ty: Type::Atom("String".to_string()),
                    },
                    StructField {
                        name: "age".to_string(),
                        ty: Type::Atom("u8".to_string()),
                    }
                ]
            }
        )
    }

    #[test]
    fn get_struct_defs_multiple() {
        let code = quote! {
            struct Person {
                name: String,
                age: u8
            }

            struct Animal {
                alive: bool
            }
        };

        let struct_defs = get_struct_defs(code);

        assert_eq!(
            struct_defs,
            vec![
                StructDef {
                    name: "Person".to_string(),
                    fields: vec![
                        StructField {
                            name: "name".to_string(),
                            ty: Type::Atom("String".to_string()),
                        },
                        StructField {
                            name: "age".to_string(),
                            ty: Type::Atom("u8".to_string()),
                        }
                    ]
                },
                StructDef {
                    name: "Animal".to_string(),
                    fields: vec![StructField {
                        name: "alive".to_string(),
                        ty: Type::Atom("bool".to_string()),
                    }]
                }
            ]
        )
    }

    #[test]
    fn get_struct_def_vec() {
        let code = quote! {
            struct Student {
                marks: Vec<u32>
            }
        };

        let struct_defs = get_struct_defs(code);

        assert_eq!(
            struct_defs,
            vec![StructDef {
                name: "Student".to_string(),
                fields: vec![StructField {
                    name: "marks".to_string(),
                    ty: Type::Vec(Box::new(Type::Atom("u32".to_string())))
                }]
            }]
        )
    }
}
