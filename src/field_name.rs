use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use std::{collections::HashSet, iter::FromIterator};
use syn::{DeriveInput, Ident};

const DEFAULT_DERIVES: &[&str] = &["Debug", "PartialEq", "Eq", "Clone", "Copy"];

use crate::common::{extract_type_ident, filter_fields, get_meta_list};

#[inline]
fn get_helper_macro_name(type_snake: &str) -> Ident {
    Ident::new(
        &format!("__{}_field_name_variants", type_snake),
        Span::call_site(),
    )
}

struct FieldNamePair {
    variant_ident: Ident,
    field_name: String,
}

/// A single field slot in declaration order
enum FieldSlot {
    /// One or more consecutive regular fields: (variant_ident, field_name)
    Regular(Vec<FieldNamePair>),
    /// A nested field - calls to the inner type's helper macro
    Nested(Ident),
}

impl FieldSlot {
    /// Render each (Variant, "name") as Variant => "name"
    fn entries(pairs: &[FieldNamePair]) -> Vec<TokenStream2> {
        pairs
            .iter()
            .map(
                |FieldNamePair {
                     variant_ident,
                     field_name,
                 }| quote! { #variant_ident => #field_name },
            )
            .collect()
    }
}

pub struct DeriveFieldName {
    vis: syn::Visibility,
    ident: Ident,
    enum_ident: Ident,
    generics: syn::Generics,
    derive_attrs: Vec<TokenStream2>,
    extra_attrs: Vec<TokenStream2>,
    /// Fields in declaration order, grouped into regular runs and nested slots.
    slots: Vec<FieldSlot>,
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

        let type_snake = ident.to_string().to_snake_case();

        // Build declaration-order slots: merge consecutive regular fields into one Regular slot.
        let mut slots: Vec<FieldSlot> = Vec::new();
        for f in &fields {
            if f.is_nested {
                let inner_type_ident = extract_type_ident(&f.field_ty)?;
                let inner_snake = inner_type_ident.to_string().to_snake_case();
                slots.push(FieldSlot::Nested(get_helper_macro_name(&inner_snake)));
            } else {
                let pair = FieldNamePair {
                    variant_ident: f.variant_ident.to_owned(),
                    field_name: f.field_ident.to_string(),
                };
                if let Some(FieldSlot::Regular(pairs)) = slots.last_mut() {
                    pairs.push(pair);
                } else {
                    slots.push(FieldSlot::Regular(vec![pair]));
                }
            }
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
            slots,
            type_snake,
            impl_generics_tokens,
            from_lifetime,
        })
    }

    pub fn expand(&self) -> syn::Result<TokenStream2> {
        let has_nested = self.slots.iter().any(|s| matches!(s, FieldSlot::Nested(_)));
        if has_nested {
            self.expand_nested()
        } else {
            self.expand_simple()
        }
    }

    fn get_fields_for_simple(&self) -> &Vec<FieldNamePair> {
        //PERF: this function is called several times instead of once
        debug_assert_eq!(self.slots.len(), 1);
        match &self.slots[..] {
            [FieldSlot::Regular(p)] => p,
            _ => unreachable!("expand_simple called with nested slots"),
        }
    }

    fn expand_simple(&self) -> syn::Result<TokenStream2> {
        // No nested fields-  exactly one Regular slot.
        let pairs = self.get_fields_for_simple();
        let entries = FieldSlot::entries(pairs);
        let variants: Vec<&Ident> = pairs
            .iter()
            .map(
                |FieldNamePair {
                     variant_ident,
                     field_name: _,
                 }| variant_ident,
            )
            .collect();
        let constructs: Vec<TokenStream2> = variants
            .iter()
            .map(|v| {
                let e = &self.enum_ident;
                quote! { #e::#v }
            })
            .collect();
        let fields_count = pairs.len();

        let vis = &self.vis;
        let ident = &self.ident;
        let enum_ident = &self.enum_ident;
        let derive_attrs = &self.derive_attrs;
        let extra_attrs = &self.extra_attrs;
        let impl_generics_tokens = &self.impl_generics_tokens;
        let from_lifetime = &self.from_lifetime;
        let (_impl_generics, ty_generics, _where_clause) = self.generics.split_for_impl();
        let helper_macro_name = get_helper_macro_name(&self.type_snake);

        let own_helper = quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #helper_macro_name {
                ($callback:tt; $($acc:tt)*) => {
                    $callback!{$($acc)* #(#entries,)*}
                };
            }
        };

        Ok(quote! {
            #[derive(#(#derive_attrs),*)]
            #(#[#extra_attrs])*
            #vis enum #enum_ident {
                #(#variants),*
            }

            impl #impl_generics_tokens From<& #ident #ty_generics> for [#enum_ident; #fields_count] {
                fn from(_source: &  #ident #ty_generics) -> Self {
                    [#(#constructs),*]
                }
            }

            #own_helper

        })
    }

    fn expand_nested(&self) -> syn::Result<TokenStream2> {
        let (_impl_generics, ty_generics, _where_clause) = self.generics.split_for_impl();
        let type_snake = &self.type_snake;

        let builder_macro_name = Ident::new(
            &format!("__{}_field_name_build", type_snake),
            Span::call_site(),
        );

        let builder_macro =
            self.generate_field_name_builder_macro(&builder_macro_name, &ty_generics);

        // Generate one step macro per slot in declaration order.
        // Step i receives ($callback:tt; $($acc:tt)*) and then passes the updated
        // accumulator to step i+1, or directly to $callback if it's on the last step
        let num_slots = self.slots.len();
        let step_name = |i: usize| {
            Ident::new(
                &format!("__{}_field_name_step_{}", type_snake, i),
                Span::call_site(),
            )
        };

        let mut step_macros: Vec<TokenStream2> = Vec::new();
        for (i, slot) in self.slots.iter().enumerate() {
            let this_step = step_name(i);
            let is_last = i == num_slots - 1;

            match slot {
                FieldSlot::Regular(pairs) => {
                    let entries = FieldSlot::entries(pairs);
                    if is_last {
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; $($acc:tt)*) => {
                                    $callback!{$($acc)* #(#entries,)*}
                                };
                            }
                        });
                    } else {
                        let next = step_name(i + 1);
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; $($acc:tt)*) => {
                                    #next!{$callback; $($acc)* #(#entries,)*}
                                };
                            }
                        });
                    }
                }
                FieldSlot::Nested(helper_mac) => {
                    if is_last {
                        // Last slot: call nested helper with $callback
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; $($acc:tt)*) => {
                                    #helper_mac!{$callback; $($acc)*}
                                };
                            }
                        });
                    } else {
                        // Non-last: nested helper's callback is the next step,
                        // which receives $callback as its first arg
                        let next = step_name(i + 1);
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; $($acc:tt)*) => {
                                    #helper_mac!{#next; $callback; $($acc)*}
                                };
                            }
                        });
                    }
                }
            }
        }

        // Build: step 0 with the builder as callback
        let step_0 = step_name(0);
        let invocation = quote! { #step_0!{#builder_macro_name;} };

        // Finnaly add own helper: when this type is nested in a grandparent,
        // just start the same step chain but with the grandparent's callback forwarded as tt.
        let helper_macro_name = get_helper_macro_name(type_snake);
        let own_helper = quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #helper_macro_name {
                ($callback:tt; $($acc:tt)*) => {
                    #step_0!{$callback; $($acc)*}
                };
            }
        };

        Ok(quote! {
            #builder_macro
            #(#step_macros)*
            #own_helper
            #invocation
        })
    }

    /// Emit the terminal macro that, once it has the full flat list of
    /// `Variant => "field_name"` pairs, generates the enum and `From` impl.
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
}
