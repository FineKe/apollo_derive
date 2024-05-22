mod field_type;

use proc_macro::TokenStream;
use std::fmt::Pointer;

use quote::quote;
use syn::{Data, DeriveInput, GenericArgument, Type};
use syn::__private::TokenStream2;
use syn::PathArguments;
use crate::field_type::{field_type, FieldType};

#[proc_macro_derive(Apollo)]
pub fn apollo_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let id = ast.ident;

    let Data::Struct(s) = ast.data else {
        panic!("Apollo derive macro must be used in struct")
    };

    let mut apply_function_ast = quote!();
    let mut collect_keys_ast = quote!();

    for (idx, f) in s.fields.iter().enumerate() {
        let (field_id, filed_ty) = (f.ident.as_ref().unwrap(), &f.ty);
        match field_type(filed_ty) {
            FieldType::PRIMITIVE => {
                expand_primitive_type(field_id, filed_ty, &mut apply_function_ast, &mut collect_keys_ast)
            }
            FieldType::OPTION => {
                expand_option_type(field_id, filed_ty, &mut apply_function_ast, &mut collect_keys_ast)
            }
            FieldType::STRUCT => {
                expand_struct_type(field_id, filed_ty, &mut apply_function_ast, &mut collect_keys_ast)
            }
            FieldType::COLLECTION => {
                expand_collection_type(field_id, filed_ty, &mut apply_function_ast, &mut collect_keys_ast)
            }
        }
        // if is_primitive_type(filed_ty) {
        //     apply_funtion_ast.extend(quote! {
        //         println!("apollo key: {}",&(prefix.to_string()+stringify!(#field_id)));
        //         if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))) {
        //             self.#field_id = v.parse().unwrap();
        //         }
        //     });
        //     collect_keys_ast.extend(quote!{
        //         keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
        //     });
        // } else if is_option_of_primitive_type(filed_ty) {
        //
        //     apply_funtion_ast.extend(quote!{
        //         println!("apollo key: {}",&(prefix.to_string()+stringify!(#field_id)));
        //         if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))) {
        //             self.#field_id = Some(v.parse().unwrap());
        //         }
        //     });
        //     collect_keys_ast.extend(quote!{
        //         keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
        //     });
        // } else {
        //     if is_option_type(filed_ty) {
        //         let ty = get_option_inner_type(filed_ty).unwrap();
        //         if is_vec_type(ty) {
        //             apply_funtion_ast.extend(quote!{
        //             self.#field_id = serde_json::from_str(config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap()).unwrap();});
        //             collect_keys_ast.extend(quote!{
        //                 keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
        //             });
        //         }else {
        //             apply_funtion_ast.extend(quote!{
        //                 if self.#field_id.is_none() {
        //                     self.#field_id = Some(#ty::default());
        //                 }
        //             // println!("key {}",&(prefix.to_string()+ stringify!(#field_id)));
        //                 self.#field_id.as_mut().unwrap().apply(&(prefix.to_string()+ stringify!(#field_id)),config);
        //         });
        //             collect_keys_ast.extend(quote!{
        //             self.#field_id.as_mut().unwrap().collect_keys(&(prefix.to_string()+ stringify!(#field_id)),keys);
        //         })
        //         }
        //     }else if is_hash_map_type(filed_ty) {
        //         apply_funtion_ast.extend(quote!{
        //             self.#field_id = serde_json::from_str(config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap()).unwrap()
        //         });
        //         collect_keys_ast.extend(quote!{
        //             keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
        //         });
        //     }
        //     else  {
        //         apply_funtion_ast.extend(quote!{
        //             self.#field_id.apply(&(prefix.to_string()+ stringify!(#field_id)),config);
        //         });
        //         collect_keys_ast.extend(quote!{
        //             self.#field_id.collect_keys(&(prefix.to_string()+ stringify!(#field_id)),keys);
        //         })
        //     }
        // }
    }

    quote!{
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
    }.into()
}


fn expand_primitive_type(field_id: &syn::Ident, ty: &syn::Type, apply_function_ast: &mut TokenStream2, collect_keys_ast: &mut TokenStream2) {
    apply_function_ast.extend(
        quote! {
                //println!("apollo key: {}",&(prefix.to_string()+stringify!(#field_id)));
                if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))) {
                    self.#field_id = v.parse().unwrap();
                }
            }
    );

    collect_keys_ast.extend(
        quote!{
                keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
            }
    );
}

fn expand_option_type(field_id: &syn::Ident, ty: &syn::Type, apply_function_ast: &mut TokenStream2, collect_keys_ast: &mut TokenStream2) {
    let ty = get_option_inner_type(ty).unwrap();
    match field_type(ty) {
        FieldType::PRIMITIVE => {
            apply_function_ast.extend(
                quote!{
                        println!("apollo key: {}",&(prefix.to_string()+stringify!(#field_id)));
                        if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))) {
                            self.#field_id = Some(v.parse().unwrap());
                        }
                }
            );
            collect_keys_ast.extend(
                quote!{
                keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
            }
            );
        }
        FieldType::OPTION => {
            panic!("nested option type is not supported")
        }
        FieldType::STRUCT => {
            apply_function_ast.extend(quote!{
                        if self.#field_id.is_none() {
                            self.#field_id = Some(#ty::default());
                        }
                        println!("key {}",&(prefix.to_string()+ stringify!(#field_id)));
                        self.#field_id.as_mut().unwrap().apply(&(prefix.to_string()+ stringify!(#field_id)),config);}
            );

            collect_keys_ast.extend(
                quote!{
                    self.#field_id.as_mut().unwrap().collect_keys(&(prefix.to_string()+ stringify!(#field_id)),keys);
                    }
            )
        }

        FieldType::COLLECTION => {
            apply_function_ast.extend(
                quote!{
                    self.#field_id = serde_json::from_str(config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap()).unwrap();
                }
            );
            collect_keys_ast.extend(
                quote!{
                        keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
                    }
            );
        }
    };
    // apply_function_ast.extend(
    //     quote!{
    //             println!("apollo key: {}",&(prefix.to_string()+stringify!(#field_id)));
    //             if let Some(v) = config.get(&(prefix.to_string()+stringify!(#field_id))) {
    //                 self.#field_id = Some(v.parse().unwrap());
    //             }
    //         }
    // );
    // collect_keys_ast.extend(
    //     quote!{
    //             keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
    //         }
    // );
}

fn expand_struct_type(field_id: &syn::Ident, ty: &syn::Type, apply_function_ast: &mut TokenStream2, collect_keys_ast: &mut TokenStream2) {
    apply_function_ast.extend(
        quote!{
                    self.#field_id.apply(&(prefix.to_string()+ stringify!(#field_id)),config);
                }
    );
    collect_keys_ast.extend(
        quote!{
                    self.#field_id.collect_keys(&(prefix.to_string()+ stringify!(#field_id)),keys);
                }
    )
}

fn expand_collection_type(field_id: &syn::Ident, ty: &syn::Type, apply_function_ast: &mut TokenStream2, collect_keys_ast: &mut TokenStream2){
    apply_function_ast.extend(
        quote!{
                    self.#field_id = serde_json::from_str(config.get(&(prefix.to_string()+stringify!(#field_id))).unwrap()).unwrap()
                }
    );
    collect_keys_ast.extend(
        quote!{
                    keys.push((prefix.to_string()+stringify!(#field_id)).to_string());
                }
    );
}


// fn is_option_of_primitive_type(ty: &syn::Type) -> bool {
//     if let syn::Type::Path(type_path) = ty {
//         if let Some(segment) = type_path.path.segments.last() {
//             let ident = &segment.ident.to_string();
//
//             // 检查是否是 Option 类型
//             if ident == "Option" {
//                 if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
//                     if args.args.len() == 1 {
//                         if let syn::GenericArgument::Type(type_arg) = &args.args[0] {
//                             // 检查 Option 包装的类型是否是基本类型
//                             return is_primitive_type(type_arg);
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     false
// }




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