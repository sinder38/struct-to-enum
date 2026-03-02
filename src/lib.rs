mod common;
mod field_name;
mod field_type;

use field_name::DeriveFieldName;
use field_type::DeriveFieldType;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(
    FieldType,
    attributes(
        stem_type,
        ste_type,
        stem_type_derive,
        ste_type_derive,
        stem_type_attr,
        ste_type_attr,
    )
)]
pub fn field_type(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    DeriveFieldType::new(input)
        .and_then(|d| d.expand())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(
    FieldName,
    attributes(
        stem_name,
        ste_name,
        stem_name_derive,
        ste_name_derive,
        stem_name_attr,
        ste_name_attr,
    )
)]
pub fn field_name(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    DeriveFieldName::new(input)
        .and_then(|d| d.expand())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
