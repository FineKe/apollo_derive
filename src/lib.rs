use proc_macro::TokenStream;

use quote::quote;
use syn::{Ident, PathArguments};
use syn::__private::TokenStream2;
use syn::{Data, DeriveInput, Type};

use crate::field_type::{field_type, get_field_type, is_serde_with_json, FieldType, get_deserialize_type};

mod field_type;

#[proc_macro_derive(Apollo, attributes(apollo))]
pub fn apollo_derive(input: TokenStream) -> TokenStream {
    let mut apply_function_ast = quote!();
    let mut collect_keys_ast = quote!();

    let ast: DeriveInput = syn::parse(input).unwrap();
    let id = ast.ident;

    match ast.data {
        Data::Struct(s) => {
            for (_, f) in s.fields.iter().enumerate() {
                let field_id = f.ident.as_ref().unwrap();
                match field_type(f) {
                    FieldType::PRIMITIVE => expand_primitive_type(
                        field_id,
                        f,
                        &mut apply_function_ast,
                        &mut collect_keys_ast,
                    ),
                    FieldType::OPTION => expand_option_type(
                        field_id,
                        f,
                        &mut apply_function_ast,
                        &mut collect_keys_ast,
                    ),
                    FieldType::STRUCT => expand_struct_type(
                        field_id,
                        f,
                        &mut apply_function_ast,
                        &mut collect_keys_ast,
                    ),
                    FieldType::COLLECTION => expand_with_json(
                        field_id,
                        f,
                        &mut apply_function_ast,
                        &mut collect_keys_ast,
                    ),
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => {
            panic!("Apollo derive macro must be used in struct")
        }
    }
    // let Data::Struct(s) = ast.data else {
    //     panic!("Apollo derive macro must be used in struct")
    // };

    quote! {
        impl ApolloConfigure for #id {
            fn apply(&mut self,prefix: &String, config: &HashMap<String, String>){
                let prefix = if prefix.len() != 0 {
                    prefix.to_string() + "."
                } else {
                    prefix.to_string()
                };

                #apply_function_ast
            }

            fn collect_keys(&mut self, prefix: &String, keys: &mut Vec<String>) {
                let prefix = if prefix.len() != 0 {
                    prefix.to_string() + "."
                } else {
                    prefix.to_string()
                };
                #collect_keys_ast
            }
        }
    }
    .into()
}

fn expand_primitive_type(
    field_id: &syn::Ident,
    _ty: &syn::Field,
    apply_function_ast: &mut TokenStream2,
    collect_keys_ast: &mut TokenStream2,
) {
    apply_function_ast.extend(quote! {
        let v = config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap();
        self.#field_id = v.parse().unwrap();
    });

    collect_keys_ast.extend(quote! {
        keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
    });
}

fn expand_option_type(
    field_id: &syn::Ident,
    f: &syn::Field,
    apply_function_ast: &mut TokenStream2,
    collect_keys_ast: &mut TokenStream2,
) {
    let ty = get_option_inner_type(&f.ty).unwrap();
    match get_field_type(ty) {
        FieldType::PRIMITIVE => {
            apply_function_ast.extend(quote! {
                    println!("apollo key: {}",&(prefix.to_string()+stringify!(#field_id)));
                    if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))) {
                        self.#field_id = Some(v.parse().unwrap());
                    }
            });
            collect_keys_ast.extend(quote! {
                keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
            });
        }
        FieldType::OPTION => {
            panic!("nested option type is not supported")
        }
        FieldType::STRUCT => {
            if is_serde_with_json(f) {
                apply_function_ast.extend(quote! {
                      if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))){
                      self.#field_id = Some(serde_json::from_str(v).unwrap());
                        }
                });
                collect_keys_ast.extend(quote! {
                    keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
                });

                return;
            }
            apply_function_ast.extend(quote!{
                        if self.#field_id.is_none() {
                            self.#field_id = Some(#ty::default());
                        }
                        self.#field_id.as_mut().unwrap().apply(&(prefix.to_string()+ stringify!(#field_id)),config);}
            );

            collect_keys_ast.extend(
                quote!{
                    self.#field_id.as_mut().unwrap().collect_keys(&(prefix.to_string()+ stringify!(#field_id)),keys);
                    }
            )
        }

        FieldType::COLLECTION => {
            apply_function_ast.extend(quote! {
               if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))){
                  self.#field_id = Some(serde_json::from_str(v).unwrap());
                }
            });
            collect_keys_ast.extend(quote! {
                keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
            });
        }
    };
}

fn expand_struct_type(
    field_id: &syn::Ident,
    field: &syn::Field,
    apply_function_ast: &mut TokenStream2,
    collect_keys_ast: &mut TokenStream2,
) {

    let deserialize_type =  get_deserialize_type(field);
    match deserialize_type {
        None => {
            apply_function_ast.extend(quote! {
            self.#field_id.apply(&(prefix.to_string()+ stringify!(#field_id)),config);
            });
            collect_keys_ast.extend(quote! {
            self.#field_id.collect_keys(&(prefix.to_string()+ stringify!(#field_id)),keys);
            })
        }
        Some(ref func) => {
            if func.to_string() == "json" {
                apply_function_ast.extend(quote! {
                 let v = config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap();
                self.#field_id = serde_json::from_str(v).unwrap();
                });
                collect_keys_ast.extend(quote! {
                keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
                });
            }else {
                apply_function_ast.extend(quote! {
                let v = config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap();
                self.#field_id = #func(v);
                });
                collect_keys_ast.extend(quote! {
                keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
                });
            }
        }
    }
}

fn expand_with_json(
    field_id: &syn::Ident,
    _ty: &syn::Field,
    apply_function_ast: &mut TokenStream2,
    collect_keys_ast: &mut TokenStream2,
) {
    apply_function_ast.extend(quote! {
        let v = config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap();
        self.#field_id = serde_json::from_str(v).unwrap();
    });
    collect_keys_ast.extend(quote! {
        keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
    });
}

fn get_option_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident.to_string() == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(inner_type) = args.args.first() {
                        if let syn::GenericArgument::Type(inner_type) = inner_type {
                            return Some(&inner_type);
                        }
                    }
                }
            }
        }
    }
    None
}
