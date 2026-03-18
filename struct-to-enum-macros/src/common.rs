use proc_macro2::{Span, TokenStream as TokenStream2};
#[cfg(any(feature = "nested-type", feature = "nested-name"))]
use quote::quote;
use quote::ToTokens;
use syn::{Attribute, Fields, Ident, Meta, Type};

use heck::ToUpperCamelCase;

//TODO: maybe I could move derive name constants to this Enum, not sure
#[derive(Debug, Clone, Copy)]
pub enum DeriveVariant {
    Type,
    Name,
}

#[derive(Clone)]
pub struct FieldInfo {
    /// Original struct field ident
    pub field_ident: Ident,
    /// Original struct field type
    pub field_ty: Type,
    /// Related field ident for generated enum
    pub variant_ident: Ident,
    #[cfg(any(feature = "nested-type", feature = "nested-name"))]
    /// Has is_nested attribute
    pub is_nested: bool,
}

/// Extract user-specified attributes
/// Used for derive and derive-attributes inheritance for generated enums
pub fn get_meta_list(
    attrs: &[Attribute],
    attr_names: &[&'static str],
) -> syn::Result<Vec<TokenStream2>> {
    let mut result = Vec::new();
    for attr in attrs {
        let meta = &attr.meta;
        // check for each alias
        for attr_name in attr_names {
            if meta.path().is_ident(attr_name) {
                if let Meta::List(meta_list) = meta {
                    let tokens = &meta_list.tokens;
                    result.push(tokens.clone());
                } else {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!("expected at least 1 argument in parentheses: #[{attr_name}(...)]"),
                    ));
                }
            }
        }
    }
    Ok(result)
}

/// Collect struct fields, skipping marked `skip` record marked as `nested`
/// Returns `FieldInfo` for each included field.
pub fn filter_fields(
    fields: &Fields,
    attr_names: &[&'static str],
    derive_type: DeriveVariant,
) -> syn::Result<Vec<FieldInfo>> {
    let mut result = Vec::new();
    for field in fields {
        let is_skip = field
            .attrs
            .iter()
            .any(|attr| has_attr_with_value(attr, attr_names, "skip"));

        if is_skip || field.ident.is_none() {
            continue;
        }

        let is_nested = field
            .attrs
            .iter()
            .any(|attr| has_attr_with_value(attr, attr_names, "nested"));

        match derive_type {
            DeriveVariant::Type =>
            {
                #[cfg(not(feature = "nested-type"))]
                if is_nested {
                    return Err(syn::Error::new_spanned(
                        field,
                        "nested fields are not supported without the 'nested-type' feature",
                    ));
                }
            }
            DeriveVariant::Name =>
            {
                #[cfg(not(feature = "nested-name"))]
                if is_nested {
                    return Err(syn::Error::new_spanned(
                        field,
                        "nested fields are not supported without the 'nested-name' feature",
                    ));
                }
            }
        }

        let field_ident = field.ident.as_ref().unwrap().clone();

        // TODO: allow changing Enum varint ident generation
        let field_name = field_ident.to_string();
        let variant_ident = Ident::new(&field_name.to_upper_camel_case(), Span::call_site());

        result.push(FieldInfo {
            field_ident,
            field_ty: field.ty.clone(),
            variant_ident,
            #[cfg(any(feature = "nested-type", feature = "nested-name"))]
            is_nested,
        });
    }
    Ok(result)
}

/// Get a field attribute value as string
fn get_attr_value(attr: &Attribute, attr_names: &[&str]) -> syn::Result<Option<String>> {
    let meta = &attr.meta;
    let matches = attr_names.iter().any(|name| meta.path().is_ident(name));
    if !matches {
        return Ok(None);
    }

    let value = match meta {
        // #[...("skip")]
        Meta::List(list) => list.tokens.to_string(),

        // #[... = "skip"]  value is an expression
        Meta::NameValue(name_value) => name_value.value.to_token_stream().to_string(),

        Meta::Path(_) => {
            return Err(syn::Error::new_spanned(meta, "Unknown attribute format"));
        }
    };

    // Strip surrounding quotes from string literals
    let normalized = value.trim_matches('"').to_string();
    Ok(Some(normalized))
}

fn has_attr_with_value(attr: &Attribute, attr_names: &[&str], expected: &str) -> bool {
    get_attr_value(attr, attr_names)
        .ok()
        .flatten()
        .is_some_and(|v| v == expected)
}

/// Extract the type name from a path
#[cfg(any(feature = "nested-type", feature = "nested-name"))]
pub fn extract_type_ident(ty: &Type) -> syn::Result<&Ident> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| &seg.ident)
            .ok_or_else(|| syn::Error::new_spanned(ty, "Type path must have at least one segment")),
        _ => Err(syn::Error::new_spanned(
            ty,
            "nested attribute can only be used with named struct types",
        )),
    }
}

pub fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Returns a token stream that counts the number of variants in an enum using macro_rules!
/// Uses field with the name `variant` to count!
#[cfg(any(feature = "nested-type", feature = "nested-name"))]
pub fn macro_rules_field_counter() -> TokenStream2 {
    quote! {
        { [$(stringify!($variant),)*].len() }
    }
}
