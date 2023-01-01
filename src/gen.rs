#![allow(unused_variables)]

use crate::def::*;
use codegen::{Formatter, Function, Scope};
use proc_macro2::TokenStream;

pub fn generate_type_from_field_ty(ty: Type) -> String {
    match ty {
        Type::Atom(atom) => format!("Type::Atom(\"{}\".to_string())", atom),
        Type::Vec(inner) => format!("Type::Vec({})", generate_type_from_field_ty(*inner)),
    }
}

pub fn generate_string_from_struct_def(struct_def: &StructDef) -> String {
    format!(
        "StructDef {{
        name: \"{}\".to_string(),
        fields: vec![{}],
    }}",
        struct_def.name,
        struct_def.fields.iter().fold("\n".to_string(), |v, field| {
            let code = format!(
                "StructField {{
            name: \"{}\".to_string(),
            ty: {}
        }}",
                field.name,
                generate_type_from_field_ty(field.ty.clone())
            );
            if v.len() <= 1 {
                code
            } else {
                format!("{v}, {code}")
            }
        })
    )
}

pub fn generate_scope_from_struct_defs(struct_defs: Vec<StructDef>) -> Scope {
    let mut scope = Scope::new();

    scope.import("rusto::def", "*");

    for struct_def in struct_defs.iter() {
        let mut func = Function::new("get_structs");
        func.line(generate_string_from_struct_def(struct_def));
        scope.push_fn(func);
    }

    scope
}

pub fn generate_scope(token_stream: TokenStream) -> Scope {
    let struct_defs = get_struct_defs(token_stream);
    generate_scope_from_struct_defs(struct_defs)
}

pub fn generate(token_stream: TokenStream) -> String {
    let scope = generate_scope(token_stream);
    scope.to_string()
}

pub fn generate_format(token_stream: TokenStream) -> String {
    let mut code = generate(token_stream);
    let mut formatter = Formatter::new(&mut code);
    formatter.indent(move |&mut _| 1);
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn get_code() -> TokenStream {
        return quote! {
            struct Person {
                name: String,
                age: u8
            }
        };
    }

    #[test]
    fn generate_test() {
        let code = generate_format(get_code());
        assert_eq!(code, "use rusto::def::*;

fn get_structs() {
    StructDef {
            name: \"Person\".to_string(),
            fields: vec![StructField {
                name: \"name\".to_string(),
                ty: Type::Atom(\"String\".to_string())
            }, StructField {
                name: \"age\".to_string(),
                ty: Type::Atom(\"u8\".to_string())
            }],
        }
}".to_string());
    }
}
