use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Ident};

use crate::common::{FieldInfo, filter_fields, get_meta_list};

pub struct DeriveFieldType {
    vis: syn::Visibility,
    ident: Ident,
    enum_ident: Ident,
    generics: syn::Generics,
    derive_attr: Vec<TokenStream2>,
    extra_attrs: Vec<TokenStream2>,
    fields: Vec<FieldInfo>,
}

impl DeriveFieldType {
    pub fn new(input: DeriveInput) -> syn::Result<Self> {
        let vis = input.vis;
        let ident = input.ident;
        let generics = input.generics;
        let enum_ident = Ident::new(&(ident.to_string() + "FieldType"), Span::call_site());

        let derive_attr = get_meta_list(&input.attrs, &["stem_type_derive", "ste_type_derive"])?;
        let extra_attrs = get_meta_list(&input.attrs, &["stem_type_attr", "ste_type_attr"])?;

        let struct_fields = match input.data {
            syn::Data::Struct(s) => s.fields,
            _ => {
                return Err(syn::Error::new_spanned(
                    &ident,
                    "FieldType can only be derived for structs",
                ));
            }
        };

        let fields = filter_fields(&struct_fields, &["stem_type", "ste_type"])?;

        if fields.is_empty() {
            return Err(syn::Error::new_spanned(
                &ident,
                "FieldType can only be derived for non-empty structs",
            ));
        }

        Ok(Self {
            vis,
            ident,
            enum_ident,
            generics,
            derive_attr,
            extra_attrs,
            fields,
        })
    }

    pub fn expand(&self) -> syn::Result<TokenStream2> {
        let enum_def = self.enum_definition();
        let converter = self.converter_impl();
        Ok(quote! {
            #enum_def
            #converter
        })
    }

    fn enum_definition(&self) -> TokenStream2 {
        let Self {
            vis,
            enum_ident,
            generics,
            derive_attr,
            extra_attrs,
            fields,
            ..
        } = self;

        let (_, _, where_clause) = generics.split_for_impl();

        let variants = fields.iter().map(|f| {
            let variant = &f.variant_ident;
            let ty = &f.field_ty;
            quote! { #variant(#ty) }
        });

        quote! {
            #[derive(#(#derive_attr),*)]
            #(#[#extra_attrs])*
            #vis enum #enum_ident #generics
                #where_clause
            {
                #(#variants),*
            }
        }
    }

    fn converter_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            enum_ident,
            generics,
            fields,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let fields_count = fields.len();

        let constructs = fields.iter().map(|f| {
            let field = &f.field_ident;
            let variant = &f.variant_ident;
            quote! { #enum_ident::#variant(#field) }
        });

        let field_idents = fields.iter().map(|f| {
            let field = &f.field_ident;
            quote! { #field }
        });
        let destructuring = quote! { #ident { #(#field_idents,)* .. } };

        if generics.params.is_empty() {
            quote! {
                impl From<#ident> for [#enum_ident; #fields_count] {
                    fn from(source: #ident) -> Self {
                        let #destructuring = source;
                        [#(#constructs),*]
                    }
                }
            }
        } else {
            quote! {
                impl #impl_generics Into<[#enum_ident #ty_generics; #fields_count]> for #ident #ty_generics
                    #where_clause
                {
                    fn into(self) -> [#enum_ident #ty_generics; #fields_count] {
                        let #destructuring = self;
                        [#(#constructs),*]
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    fn field_type_array_impl(&self) -> TokenStream2 {
        let Self {
            vis,
            ident,
            enum_ident,
            generics,
            fields,
            ..
        } = self;
        // Use From/Into instead

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let fields_count = fields.len();

        let constructs = fields.iter().map(|f| {
            let field = &f.field_ident;
            let variant = &f.variant_ident;
            quote! { #enum_ident::#variant(#field) }
        });

        let field_idents = fields.iter().map(|f| {
            let field = &f.field_ident;
            quote! { #field }
        });
        let destructuring = quote! { #ident { #(#field_idents,)* .. } };

        quote! {
            impl #impl_generics #ident #ty_generics
                #where_clause
            {
                #vis fn into_field_type_array(self) -> [#enum_ident #ty_generics; #fields_count] {
                    let #destructuring = self;
                    [#(#constructs),*]
                }
            }
        }
    }
}
