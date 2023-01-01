#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate syn;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::visit::{self, Visit};
use syn::{DataStruct, File, ItemFn, ItemStruct, Token};

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
                            let field_ty = segs.next().unwrap().ident.to_owned().to_string();
                            if field_ty == "Vec" {
                                let inner_ty = segs.next();
                                match inner_ty {
                                    Some(field_ty) => struct_fields.push(StructField {
                                        name: field_name,
                                        ty: Type::Vec(Box::new(Type::Atom(
                                            field_ty.ident.to_owned().to_string(),
                                        ))),
                                    }),
                                    None => panic!("The Type inside the Vec not found!"),
                                }
                            } else {
                                struct_fields.push(StructField {
                                    name: field_name,
                                    ty: Type::Atom(field_ty),
                                });
                            }
                        }
                        _ => (),
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
                fields: Vec::new()
            }]
        )
    }
}
