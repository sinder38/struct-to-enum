use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Ident, Meta};

use crate::common::{filter_fields, DeriveVariant};

/// Extracts the enum path from an attribute like `#[stem_reverse_name(MyEnum)]`
fn get_enum_path(attrs: &[syn::Attribute], attr_names: &[&str]) -> syn::Result<syn::Path> {
    for attr in attrs {
        let meta = &attr.meta;
        for attr_name in attr_names {
            if meta.path().is_ident(attr_name) {
                if let Meta::List(meta_list) = meta {
                    let path: syn::Path = syn::parse2(meta_list.tokens.clone())?;
                    return Ok(path);
                } else {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!("expected #[{attr_name}(EnumName)]"),
                    ));
                }
            }
        }
    }
    Err(syn::Error::new(
        Span::call_site(),
        format!(
            "missing required attribute: #[{}(EnumName)]",
            attr_names[0]
        ),
    ))
}

pub fn expand_verify_field_name(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;

    let enum_path = get_enum_path(
        &input.attrs,
        &["stem_reverse_name", "ste_reverse_name"],
    )?;

    let struct_fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "VerifyFieldName can only be derived for structs",
            ));
        }
    };

    let fields = filter_fields(
        struct_fields,
        &["stem_name", "ste_name"],
        DeriveVariant::Name,
    )?;

    let variant_count = fields.len();

    // Generate assertions that each expected variant exists on the enum
    // and that the enum is exhaustively covered
    let variant_idents: Vec<&Ident> = fields.iter().map(|f| &f.variant_ident).collect();
    let field_names: Vec<String> = fields.iter().map(|f| f.field_ident.to_string()).collect();

    let test_mod_name = Ident::new(
        &format!(
            "__verify_{}_field_name",
            ident.to_string().to_snake_case()
        ),
        Span::call_site(),
    );

    let test_fn_name = Ident::new(
        &format!(
            "verify_{}_matches_field_name",
            ident.to_string().to_snake_case()
        ),
        Span::call_site(),
    );

    // Build a match arm for each variant to ensure exhaustiveness
    let match_arms: Vec<TokenStream2> = variant_idents
        .iter()
        .zip(field_names.iter())
        .map(|(variant, name)| {
            quote! { #enum_path::#variant => #name }
        })
        .collect();

    // Build array of variant constructions to verify order and count
    let variant_constructs: Vec<TokenStream2> = variant_idents
        .iter()
        .map(|v| quote! { #enum_path::#v })
        .collect();

    Ok(quote! {
        #[cfg(test)]
        mod #test_mod_name {
            use super::*;

            #[test]
            fn #test_fn_name() {
                // Verify variant count matches
                let variants: [#enum_path; #variant_count] = [
                    #(#variant_constructs),*
                ];
                assert_eq!(variants.len(), #variant_count, "enum variant count mismatch");

                // Verify exhaustive match (will fail to compile if enum has extra variants)
                fn exhaustive_check(v: #enum_path) -> &'static str {
                    match v {
                        #(#match_arms),*
                    }
                }

                // Verify each variant maps to the correct field name
                #(
                    assert_eq!(
                        exhaustive_check(#enum_path::#variant_idents),
                        #field_names,
                        "variant name mismatch"
                    );
                )*
            }
        }
    })
}

pub fn expand_verify_field_type(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;

    let enum_path = get_enum_path(
        &input.attrs,
        &["stem_reverse_type", "ste_reverse_type"],
    )?;

    let struct_fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "VerifyFieldType can only be derived for structs",
            ));
        }
    };

    let fields = filter_fields(
        struct_fields,
        &["stem_type", "ste_type"],
        DeriveVariant::Type,
    )?;

    let variant_idents: Vec<&Ident> = fields.iter().map(|f| &f.variant_ident).collect();
    let field_tys: Vec<&syn::Type> = fields.iter().map(|f| &f.field_ty).collect();
    let field_idents: Vec<&Ident> = fields.iter().map(|f| &f.field_ident).collect();

    let test_mod_name = Ident::new(
        &format!(
            "__verify_{}_field_type",
            ident.to_string().to_snake_case()
        ),
        Span::call_site(),
    );

    let test_fn_name = Ident::new(
        &format!(
            "verify_{}_matches_field_type",
            ident.to_string().to_snake_case()
        ),
        Span::call_site(),
    );

    // Build match arms that destructure the tuple variant and check the type
    let match_arms: Vec<TokenStream2> = variant_idents
        .iter()
        .zip(field_idents.iter())
        .map(|(variant, field)| {
            let name = field.to_string();
            quote! { #enum_path::#variant(_) => #name }
        })
        .collect();

    // Build type assertion functions to verify each variant wraps the correct type
    let type_checks: Vec<TokenStream2> = variant_idents
        .iter()
        .zip(field_tys.iter())
        .enumerate()
        .map(|(i, (variant, ty))| {
            let check_fn = Ident::new(&format!("__check_type_{i}"), Span::call_site());
            quote! {
                fn #check_fn(v: #ty) -> #enum_path {
                    #enum_path::#variant(v)
                }
            }
        })
        .collect();

    Ok(quote! {
        #[cfg(test)]
        mod #test_mod_name {
            use super::*;

            // Type-checking functions: each verifies that the variant accepts the expected type.
            // If the enum variant wraps a different type, this will fail to compile.
            #(#type_checks)*

            #[test]
            fn #test_fn_name() {
                // Verify exhaustive match (will fail to compile if enum has extra/missing variants)
                fn exhaustive_check(v: #enum_path) -> &'static str {
                    match v {
                        #(#match_arms),*
                    }
                }

                let _ = exhaustive_check;
            }
        }
    })
}
