use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{parse_macro_input, token::Comma, DeriveInput, GenericParam, Type, TypeArray, TypePath};

use crate::{
    new_utils::{
        get_base_type_witness_fn_name_by_ident, get_empty_path_field_initialization_of_type,
        get_equivalent_type, get_type_params_from_generics, get_type_path_of_field,
        get_witness_ident, has_engine_generic_param,
    },
    new_witness::derive_witness_struct,
};

pub(crate) fn is_primitive(path_ty: &TypePath) -> bool {
    for p in ["u8", "u16", "u32", "u64", "u128"].iter() {
        if *path_ty == syn::parse_str::<TypePath>(p).unwrap() {
            return true;
        }
    }
    false
}

pub(crate) fn derive_get_witness(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derived_input = parse_macro_input!(input as DeriveInput);

    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = derived_input.clone();

    let mut array_initializations = TokenStream::new();
    let mut witness_initializations = TokenStream::new();

    match data {
        syn::Data::Struct(ref struct_data) => match struct_data.fields {
            syn::Fields::Named(ref named_fields) => {
                for field in named_fields.named.iter() {
                    let field_ident = field.ident.clone().expect("a field ident");

                    let expanded_init_line = match field.ty {
                        Type::Path(ref path_ty) => {
                            derive_get_witness_by_path(&field_ident, path_ty)
                        }
                        Type::Array(ref arr_ty) => {
                            let (expanded_field_init, expaned_array_init) =
                                derive_get_witness_by_array(&field_ident, arr_ty);
                            array_initializations.extend(expaned_array_init);
                            expanded_field_init
                        }
                        _ => abort_call_site!("only array and path types are allowed"),
                    };

                    witness_initializations.extend(expanded_init_line);
                }
            }
            _ => abort_call_site!("only named fields are allowed"),
        },
        _ => abort_call_site!("only data structs are allowed"),
    }

    let witness_ident = get_witness_ident(&ident);
    let witness_struct = derive_witness_struct(derived_input);

    let comma = Comma(Span::call_site());

    let type_params_of_allocated_struct = get_type_params_from_generics(&generics, &comma, false);
    let type_params_of_witness_struct =
        get_type_params_from_generics(&witness_struct.generics, &comma, false);

    let engine_generic_param = syn::parse_str::<GenericParam>(&"E: Engine").unwrap();
    let additional_engine_generic =
        if has_engine_generic_param(&generics.params, &engine_generic_param) {
            quote! {}
        } else {
            quote! {<E:Engine>}
        };

    let expanded = quote! {
        impl#generics #ident<#type_params_of_allocated_struct>{
            pub fn get_witness#additional_engine_generic(&self) -> Option<#witness_ident<#type_params_of_witness_struct>>{
                use num_traits::Zero;
                use std::convert::TryInto;
                #array_initializations

                let witness = #witness_ident{
                    #witness_initializations

                    _marker: std::marker::PhantomData,
                };

                Some(witness)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn derive_get_witness_by_array(ident: &Ident, ty: &TypeArray) -> (TokenStream, TokenStream) {
    match *ty.elem {
        Type::Path(ref _p) => {}
        _ => abort_call_site!("only array of elements is allowed here"),
    };

    let len = &ty.len;
    let ty_arr = Type::Array(ty.clone());
    let ty_path = get_type_path_of_field(&ty_arr);
    let eq_ty = get_equivalent_type(&ty_arr);
    let ty_ident = get_type_path_of_field(&eq_ty);
    let fn_ident =
        if let Some(base_type_fn_ident) = get_base_type_witness_fn_name_by_ident(&ty_path) {
            base_type_fn_ident
        } else {
            syn::parse_str("get_witness").unwrap()
        };

    let field_initialization = get_empty_path_field_initialization_of_type(&ty_ident);
    // handle "SmallFixedWidthInteger<E, U32>
    let small_fixed_int = syn::parse_str::<TypePath>("SmallFixedWidthInteger<E, U32>").unwrap();
    let expaned_array_init = if ty_path == small_fixed_int {
        quote! {
            let mut #ident = [#field_initialization; #len];
            for (a, b) in #ident.iter_mut().zip(self.#ident.iter()){
                *a = b.#fn_ident()? as #ty_ident;
            }
        }
    } else {
        quote! {
            let mut #ident: #eq_ty = vec![#field_initialization; #len].try_into().unwrap();
            for (a, b) in #ident.iter_mut().zip(self.#ident.iter()){
                *a = b.#fn_ident()?;
            }
        }
    };

    let expanded_field = quote! {
        #ident,
    };

    (expanded_field, expaned_array_init)
}

fn derive_get_witness_by_path(ident: &Ident, ty: &TypePath) -> TokenStream {
    let fn_ident = if let Some(base_type_fn_ident) = get_base_type_witness_fn_name_by_ident(ty) {
        base_type_fn_ident
    } else {
        syn::parse_str("get_witness").unwrap()
    };
    let eq_ty = get_equivalent_type(&Type::Path(ty.clone()));
    let ty_ident = get_type_path_of_field(&eq_ty);
    // handle "SmallFixedWidthInteger<E, U32>"
    let small_fixed_int = syn::parse_str::<TypePath>("SmallFixedWidthInteger<E, U32>").unwrap();
    if *ty == small_fixed_int {
        quote! {
            #ident: self.#ident.#fn_ident()? as #ty_ident,
        }
    } else {
        quote! {
            #ident: self.#ident.#fn_ident()?,
        }
    }
}
