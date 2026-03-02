use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use std::{collections::HashSet, iter::FromIterator};
use syn::{DeriveInput, Ident};

const DEFAULT_DERIVES: &[&str] = &["Debug", "PartialEq", "Eq", "Clone", "Copy"];

use crate::common::{FieldInfo, extract_type_ident, filter_fields, get_meta_list};

struct NestedFieldInfo {
    helper_macro: Ident,
}

pub struct DeriveFieldName {
    vis: syn::Visibility,
    ident: Ident,
    enum_ident: Ident,
    generics: syn::Generics,
    derive_attrs: Vec<TokenStream2>,
    extra_attrs: Vec<TokenStream2>,
    regular_variant_pairs: Vec<(Ident, String)>,
    nested_fields: Vec<NestedFieldInfo>,
    type_snake: String,
    impl_generics_tokens: TokenStream2,
    from_lifetime: TokenStream2,
}

impl DeriveFieldName {
    pub fn new(input: DeriveInput) -> syn::Result<Self> {
        let vis = input.vis;
        let ident = input.ident;
        let generics = input.generics;

        let struct_fields = match input.data {
            syn::Data::Struct(s) => s.fields,
            _ => {
                return Err(syn::Error::new_spanned(
                    &ident,
                    "FieldName can only be derived for structs",
                ));
            }
        };
        let enum_ident = Ident::new(&(ident.to_string() + "FieldName"), Span::call_site());

        let mut derive_attrs =
            get_meta_list(&input.attrs, &["stem_name_derive", "ste_name_derive"])?;

        let merge_defaults = true; //TODO: make configurable later

        if merge_defaults {
            let present_derives: HashSet<String> = derive_attrs
                .iter()
                .flat_map(|ts| {
                    ts.clone().into_iter().filter_map(|tt| match tt {
                        proc_macro2::TokenTree::Ident(id) => Some(id.to_string()),
                        _ => None,
                    })
                })
                .collect();

            for d_derive in DEFAULT_DERIVES {
                if !present_derives.contains(*d_derive) {
                    derive_attrs.push(
                        d_derive
                            .parse::<TokenStream2>()
                            .expect("Invalid default derive"),
                    );
                }
            }
        }

        let extra_attrs = get_meta_list(&input.attrs, &["stem_name_attr", "ste_name_attr"])?;

        let fields = filter_fields(&struct_fields, &["stem_name", "ste_name"])?;

        if fields.is_empty() {
            return Err(syn::Error::new_spanned(
                &ident,
                "FieldName can only be derived for non-empty structures",
            ));
        }

        let regular_fields: Vec<&FieldInfo> = fields.iter().filter(|f| !f.is_nested).collect();
        let nested_raw: Vec<&FieldInfo> = fields.iter().filter(|f| f.is_nested).collect();

        let regular_variant_pairs: Vec<(Ident, String)> = regular_fields
            .iter()
            .map(|f| (f.variant_ident.clone(), f.field_ident.to_string()))
            .collect();

        let type_snake = ident.to_string().to_snake_case();

        let mut nested_fields = Vec::new();
        for f in &nested_raw {
            let inner_type_ident = extract_type_ident(&f.field_ty)?;
            let inner_snake = inner_type_ident.to_string().to_snake_case();
            nested_fields.push(NestedFieldInfo {
                helper_macro: Ident::new(
                    &format!("__{}_field_name_variants", inner_snake),
                    Span::call_site(),
                ),
            });
        }

        let (impl_generics, _, _) = generics.split_for_impl();
        let from_lifetime = quote! { 'field_name_from_lifetime__ };

        let mut impl_generics_tokens = TokenStream2::new();
        impl_generics.to_tokens(&mut impl_generics_tokens);
        if impl_generics_tokens.is_empty() {
            impl_generics_tokens = quote! { <#from_lifetime> };
        } else {
            let mut tokens: Vec<_> = quote! { #from_lifetime, }.into_iter().collect();
            let mut gen_iter = impl_generics_tokens.into_iter();
            if let Some(token) = gen_iter.next() {
                tokens.insert(0, token);
            }
            tokens.extend(gen_iter);
            impl_generics_tokens = TokenStream2::from_iter(tokens);
        }

        Ok(Self {
            vis,
            ident,
            enum_ident,
            generics,
            derive_attrs,
            extra_attrs,
            regular_variant_pairs,
            nested_fields,
            type_snake,
            impl_generics_tokens,
            from_lifetime,
        })
    }

    pub fn expand(&self) -> syn::Result<TokenStream2> {
        if self.nested_fields.is_empty() {
            self.expand_simple()
        } else {
            self.expand_nested()
        }
    }

    fn helper_variant_entries(&self) -> Vec<TokenStream2> {
        self.regular_variant_pairs
            .iter()
            .map(|(variant_ident, field_name)| {
                quote! { #variant_ident => #field_name }
            })
            .collect()
    }

    fn expand_simple(&self) -> syn::Result<TokenStream2> {
        let Self {
            vis,
            ident,
            enum_ident,
            generics,
            derive_attrs,
            extra_attrs,
            regular_variant_pairs,
            impl_generics_tokens,
            from_lifetime,
            type_snake,
            ..
        } = self;

        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

        let helper_variant_entries = self.helper_variant_entries();

        let field_name_variants = regular_variant_pairs
            .iter()
            .map(|(variant_ident, _)| quote! { #variant_ident });

        let field_name_constructs = regular_variant_pairs
            .iter()
            .map(|(variant_ident, _)| quote! { #enum_ident::#variant_ident });

        let from_field_name_constructs = field_name_constructs.clone();

        let fields_count = regular_variant_pairs.len();

        let helper_macro_name = Ident::new(
            &format!("__{}_field_name_variants", type_snake),
            Span::call_site(),
        );

        Ok(quote! {
            #[derive(#(#derive_attrs),*)]
            #(#[#extra_attrs])*
            #vis enum #enum_ident {
                #(#field_name_variants),*
            }

            impl #impl_generics_tokens From<& #from_lifetime #ident #ty_generics> for [#enum_ident; #fields_count] {
                fn from(_source: & #from_lifetime #ident #ty_generics) -> Self {
                    [#(#from_field_name_constructs),*]
                }
            }

            #[doc(hidden)]
            #[macro_export]
            macro_rules! #helper_macro_name {
                ($callback:path; $($acc:tt)*) => {
                    $callback!{$($acc)* #(#helper_variant_entries,)*}
                };
            }
        })
    }

    fn expand_nested(&self) -> syn::Result<TokenStream2> {
        let (_impl_generics, ty_generics, _where_clause) = self.generics.split_for_impl();

        let helper_variant_entries = self.helper_variant_entries();

        let type_snake = &self.type_snake;
        let nested_helper_macros: Vec<&Ident> =
            self.nested_fields.iter().map(|n| &n.helper_macro).collect();
        let num_nested = nested_helper_macros.len();

        let builder_macro_name = Ident::new(
            &format!("__{}_field_name_build", type_snake),
            Span::call_site(),
        );

        let builder_macro =
            self.generate_field_name_builder_macro(&builder_macro_name, &ty_generics);

        let mut chain_macros = Vec::new();
        for (i, nested_mac) in nested_helper_macros.iter().enumerate().skip(1) {
            let chain_name = Ident::new(
                &format!("__{}_field_name_chain_{}", type_snake, i),
                Span::call_site(),
            );
            let next_target = if i == num_nested - 1 {
                builder_macro_name.clone()
            } else {
                Ident::new(
                    &format!("__{}_field_name_chain_{}", type_snake, i + 1),
                    Span::call_site(),
                )
            };
            chain_macros.push(Self::generate_chain_macro(
                &chain_name,
                nested_mac,
                &next_target,
            ));
        }

        let first_target = if num_nested == 1 {
            builder_macro_name.clone()
        } else {
            Ident::new(
                &format!("__{}_field_name_chain_1", type_snake),
                Span::call_site(),
            )
        };

        let first_nested_mac = nested_helper_macros[0];

        let invocation = quote! {
            #first_nested_mac!{#first_target; #(#helper_variant_entries,)*}
        };

        let helper_macro_name = Ident::new(
            &format!("__{}_field_name_variants", type_snake),
            Span::call_site(),
        );

        let own_helper = self.generate_own_helper_with_nested(
            &helper_macro_name,
            &helper_variant_entries,
            &nested_helper_macros,
        );

        Ok(quote! {
            #builder_macro
            #(#chain_macros)*
            #own_helper
            #invocation
        })
    }

    /// Emit a `macro_rules!` that receives all `Variant => "field_name"` pairs (accumulated
    /// through the callback chain)
    fn generate_field_name_builder_macro(
        &self,
        macro_name: &Ident,
        ty_generics: &syn::TypeGenerics,
    ) -> TokenStream2 {
        let vis = &self.vis;
        let enum_ty = &self.enum_ident;
        let derive = &self.derive_attrs;
        let attrs = &self.extra_attrs;
        let ty = &self.ident;
        let impl_generics_tokens = &self.impl_generics_tokens;
        let from_lifetime = &self.from_lifetime;
        quote! {
            #[doc(hidden)]
            macro_rules! #macro_name {
                ($($variant:ident => $name_str:expr,)*) => {
                    #[derive(#(#derive),*)]
                    #(#[#attrs])*
                    #vis enum #enum_ty {
                        $($variant),*
                    }

                    impl #impl_generics_tokens From<& #from_lifetime #ty #ty_generics> for [#enum_ty; { let mut _n = 0usize; $({ let _ = stringify!($variant); _n += 1; })* _n }] {
                        fn from(_source: & #from_lifetime #ty #ty_generics) -> Self {
                            [$(#enum_ty::$variant),*]
                        }
                    }
                };
            }
        }
    }

    /// Emit an intermediary macro that forwards accumulated variants to the next nested type's helper.
    fn generate_chain_macro(
        chain_name: &Ident,
        nested_mac: &Ident,
        next_target: &Ident,
    ) -> TokenStream2 {
        quote! {
            #[doc(hidden)]
            macro_rules! #chain_name {
                ($($acc:tt)*) => {
                    #nested_mac!{#next_target; $($acc)*}
                };
            }
        }
    }

    /// Emit this type's exported helper macro.
    fn generate_own_helper_with_nested(
        &self,
        helper_macro_name: &Ident,
        helper_variant_entries: &[TokenStream2],
        nested_helper_macros: &[&Ident],
    ) -> TokenStream2 {
        let num_nested = nested_helper_macros.len();
        let type_snake = &self.type_snake;

        if num_nested == 0 {
            quote! {
                #[doc(hidden)]
                #[macro_export]
                macro_rules! #helper_macro_name {
                    ($callback:path; $($acc:tt)*) => {
                        $callback!{$($acc)* #(#helper_variant_entries,)*}
                    };
                }
            }
        } else if num_nested == 1 {
            let nested_mac = nested_helper_macros[0];
            quote! {
                #[doc(hidden)]
                #[macro_export]
                macro_rules! #helper_macro_name {
                    ($callback:path; $($acc:tt)*) => {
                        #nested_mac!{$callback; $($acc)* #(#helper_variant_entries,)*}
                    };
                }
            }
        } else {
            let mut fwd_macros = Vec::new();
            for (i, nested_mac_i) in nested_helper_macros.iter().enumerate().skip(1) {
                let fwd_name = Ident::new(
                    &format!("__{}_field_name_helper_fwd_{}", type_snake, i),
                    Span::call_site(),
                );
                if i == num_nested - 1 {
                    fwd_macros.push(quote! {
                        #[doc(hidden)]
                        macro_rules! #fwd_name {
                            ($callback:path; $($acc:tt)*) => {
                                #nested_mac_i!{$callback; $($acc)*}
                            };
                        }
                    });
                } else {
                    let next_fwd = Ident::new(
                        &format!("__{}_field_name_helper_fwd_{}", type_snake, i + 1),
                        Span::call_site(),
                    );
                    fwd_macros.push(quote! {
                        #[doc(hidden)]
                        macro_rules! #fwd_name {
                            ($callback:path; $($acc:tt)*) => {
                                #nested_mac_i!{#next_fwd; $callback; $($acc)*}
                            };
                        }
                    });
                }
            }

            let first_fwd = Ident::new(
                &format!("__{}_field_name_helper_fwd_1", type_snake),
                Span::call_site(),
            );
            let first_nested = nested_helper_macros[0];

            quote! {
                #(#fwd_macros)*

                #[doc(hidden)]
                #[macro_export]
                macro_rules! #helper_macro_name {
                    ($callback:path; $($acc:tt)*) => {
                        #first_nested!{#first_fwd; $callback; $($acc)* #(#helper_variant_entries,)*}
                    };
                }
            }
        }
    }
}
