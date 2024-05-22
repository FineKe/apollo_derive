use std::any::Any;
use syn::{Field, Type};


pub enum FieldType {
    PRIMITIVE,
    OPTION,
    STRUCT,
    COLLECTION
}

pub fn field_type(ty: &syn::Type) -> FieldType {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = &segment.ident.to_string();
            return match ident.as_str() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "usize" | "isize" | "f32" | "f64" | "bool" | "char" | "str" | "String" => return FieldType::PRIMITIVE,
                "Option" => return FieldType::OPTION,
                "HashMap" | "Vec" => return FieldType::COLLECTION,
                _ => return FieldType::STRUCT,

            }
        }
    }

    panic!("Unsupported field type: ");
}

fn is_collection_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        let segments = &type_path.path.segments;
        if segments.len() == 1  {
            let ident = segments[0].ident.to_string();
            if ident == "Vec" || ident == "HashMap" {
                return true;
            }
        }
    }
    false
}

fn is_primitive_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = &segment.ident.to_string();
            // 检查是否是 Rust 的基本类型
            match ident.as_str() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "usize" | "isize" | "f32" | "f64" | "bool" | "char" | "str" | "String" => return true,
                _ => return false,
            }
        }
    }
    false
}
