use syn::{Field, Ident, Meta, NestedMeta, Type};

pub enum FieldType {
    PRIMITIVE,
    OPTION,
    STRUCT,
    COLLECTION,
}

pub fn field_type(ty: &syn::Field) -> FieldType {
    get_field_type(&ty.ty)
}

pub fn get_field_type(ty: &Type) -> FieldType {
    if let syn::Type::Path(type_path) = &ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = &segment.ident.to_string();
            return match ident.as_str() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "usize" | "isize" | "f32" | "f64" | "bool" | "char" | "str" | "String" => {
                    FieldType::PRIMITIVE
                }
                "Option" => FieldType::OPTION,
                "HashMap" | "Vec" => FieldType::COLLECTION,
                _ => FieldType::STRUCT,
            };
        }
    }

    panic!("Unsupported field type: ");
}

pub fn is_serde_with_json(field: &Field) -> bool {
    for attr in &field.attrs {
        if !attr.path.is_ident("apollo") {
            continue;
        }

        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            for nested_meta in meta_list.nested {
                if let NestedMeta::Meta(Meta::NameValue(meta_name_value)) = nested_meta {
                    if meta_name_value.path.is_ident("with") {
                        if let syn::Lit::Str(lit_str) = meta_name_value.lit {
                            let renamed_value = lit_str.value();
                            if renamed_value == "json" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

pub fn get_deserialize_type(field: &Field) -> Option<Ident> {
    for attr in &field.attrs {
        if !attr.path.is_ident("apollo") {
            continue;
        }

        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            for nested_meta in meta_list.nested {
                if let NestedMeta::Meta(Meta::NameValue(meta_name_value)) = nested_meta {
                    if meta_name_value.path.is_ident("with") {
                        // if let syn::Lit::Str(lit_str) = meta_name_value.lit {
                        //     let renamed_value = lit_str.value();
                        //     return renamed_value;
                        // }

                        if let syn::Lit::Str(func_name) = &meta_name_value.lit {
                            let func_ident = syn::Ident::new(&func_name.value(), func_name.span());
                            return Some(func_ident);
                        }
                    }
                }
            }
        }
    }


    None
}