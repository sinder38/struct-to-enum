use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Ident, Path};

const DEFAULT_DERIVES: &[&str] = &["Debug", "PartialEq", "Eq", "Clone", "Copy"];

use crate::common::{
    extract_type_ident, filter_fields, get_meta_list, macro_rules_field_counter, path_to_string,
};

/// Returns the hidden helper macro identifier used to forward field-name entries
/// from a nested type's expansion up to its parent's accumulator
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
    /// Visibility of the generated enum
    vis: syn::Visibility,
    /// Origin struct Ident
    ident: Ident,
    /// Generated Enum Ident
    enum_ident: Ident,
    generics: syn::Generics,
    enum_derives: Vec<syn::Path>,
    extra_attrs: Vec<TokenStream2>,
    /// Fields in declaration order, grouped into regular runs and nested slots.
    slots: Vec<FieldSlot>,
    type_snake: String,
}

impl DeriveFieldName {
    /// Parses DeriveInput and collects data into [`DeriveFieldName`]
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

        let derive_attrs_ts =
            get_meta_list(&input.attrs, &["stem_name_derive", "ste_name_derive"])?;
        //PERF: Could pass code below as closure to avoid collecting into a vector inside get_meta_list

        let enum_derives = extract_enum_derives(derive_attrs_ts)?;

        let extra_attrs = get_meta_list(&input.attrs, &["stem_name_attr", "ste_name_attr"])?;

        let fields = filter_fields(&struct_fields, &["stem_name", "ste_name"])?;

        //TODO: allow empty structs by deriving from an empty enum later
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

        Ok(Self {
            vis,
            ident,
            enum_ident,
            generics,
            enum_derives,
            extra_attrs,
            slots,
            type_snake,
        })
    }

    /// Generates the derived `FieldName` enum and its implmentations
    /// uses either simple or nested expansion path.
    pub fn expand(&self) -> syn::Result<TokenStream2> {
        let has_nested = self.slots.iter().any(|s| matches!(s, FieldSlot::Nested(_)));
        if has_nested {
            self.expand_nested()
        } else {
            self.expand_simple()
        }
    }

    /// Returns the flat list of `FieldNamePair`s
    /// Only valid when there are no nested slots (expant_simple)
    fn get_fields_for_simple(&self) -> &Vec<FieldNamePair> {
        //PERF: this function is called several times instead of once
        debug_assert_eq!(self.slots.len(), 1);
        match &self.slots[..] {
            [FieldSlot::Regular(p)] => p,
            _ => unreachable!("expand_simple called with nested slots"),
        }
    }

    /// Expands with no nested fields
    fn expand_simple(&self) -> syn::Result<TokenStream2> {
        // No nested fields-  exactly one Regular slot.
        let pairs = self.get_fields_for_simple();
        let entries = FieldSlot::entries(pairs);
        let variants: Vec<TokenStream2> = pairs
            .iter()
            .map(
                |FieldNamePair {
                     variant_ident,
                     field_name: _,
                 }| quote! {#variant_ident},
            )
            .collect();
        let variant_count = variants.len();
        let constructs: Vec<TokenStream2> = variants
            .iter()
            .map(|v| {
                let e = &self.enum_ident;
                quote! { #e::#v }
            })
            .collect();

        let derive_attrs = &self.enum_derives;
        let helper_macro_name = get_helper_macro_name(&self.type_snake);

        // Parts
        let own_helper = quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #helper_macro_name {
                ($callback:tt; $($acc:tt)*) => {
                    $callback!{$($acc)* #(#entries,)*}
                };
            }
        };
        let variant_counter = quote! { #variant_count };
        let enum_def = self.gen_enum_def(derive_attrs, &variants);
        let field_names_impl = self.gen_field_names_impl(&constructs, variant_counter);

        Ok(quote! {
            #enum_def
            #field_names_impl
            #own_helper
        })
    }

    /// Expands the struct with one or more nested fields using a chain of step macros
    /// that accumulate `Variant => "name"` pairs
    fn expand_nested(&self) -> syn::Result<TokenStream2> {
        let type_snake = &self.type_snake;

        let builder_macro_name = Ident::new(
            &format!("__{}_field_name_build", type_snake),
            Span::call_site(),
        );

        let builder_macro = self.generate_field_name_builder_macro(&builder_macro_name);

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

            let next = (!is_last).then(|| step_name(i + 1));

            let step_macro_body = match slot {
                FieldSlot::Regular(pairs) => {
                    let entries = FieldSlot::entries(pairs);
                    match next {
                        None => quote! { $callback!{$($acc)* #(#entries,)*} },
                        Some(next) => quote! { #next!{$callback; $($acc)* #(#entries,)*} },
                    }
                }
                FieldSlot::Nested(helper_mac) => match next {
                    // Last slot: call nested helper with $callback
                    None => quote! { #helper_mac!{$callback; $($acc)*} },
                    // Non-last: nested helper's callback is the next step,
                    // which receives $callback as its first arg
                    Some(next) => quote! { #helper_mac!{#next; $callback; $($acc)*} },
                },
            };

            step_macros.push(make_step_macro(&this_step, step_macro_body));
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

    /// Emit the builder macro that, once it has the full flat list of
    /// `Variant => "field_name"` pairs, generates the enum and `From` impl.
    fn generate_field_name_builder_macro(&self, macro_name: &Ident) -> TokenStream2 {
        let enum_ty = &self.enum_ident;
        let enum_derives = &self.enum_derives;

        // Parts
        let variant_counter = macro_rules_field_counter();
        let enum_def = self.gen_enum_def(enum_derives, &[quote!($($variant),*)]);
        let field_names_impl =
            self.gen_field_names_impl(&[quote!($(#enum_ty::$variant),*)], variant_counter);

        quote! {
            #[doc(hidden)]
            macro_rules! #macro_name {
                ($($variant:ident => $name_str:expr,)*) => {
                    #enum_def
                    #field_names_impl
                };
            }
        }
    }

    /// Generates the enum definition
    fn gen_enum_def(&self, derive_attrs: &Vec<Path>, variants: &[TokenStream2]) -> TokenStream2 {
        let vis = &self.vis;
        let enum_ident = &self.enum_ident;
        let extra_attrs = &self.extra_attrs;
        quote! {
            #[derive(#(#derive_attrs),*)]
            #(#[#extra_attrs])*
            #vis enum #enum_ident {
                #(#variants),*
            }
        }
    }

    /// Generates the `impl FieldNames<N> for OriginalStruct` block
    /// variant_counter has to be `Tokenstream2` because nested enum field names aren't known at derive macro level
    fn gen_field_names_impl(
        &self,
        constructs: &[TokenStream2],
        variant_counter: TokenStream2,
    ) -> TokenStream2 {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let enum_ident = &self.enum_ident;
        quote! {
            #[automatically_derived]
            impl #impl_generics ::struct_to_enum::FieldNames<#variant_counter>
                for #ident #ty_generics
                #where_clause
            {
                type FieldName = #enum_ident;
                fn field_names() -> [Self::FieldName; #variant_counter] {
                    [#(#constructs),*]
                }
            }
        }
    }
}

/// Parses the token streams from macro attributes into a list of derive paths, respecting the `no_defaults` flag and
/// merging with `DEFAULT_DERIVES`
fn extract_enum_derives(derive_attrs_ts: Vec<TokenStream2>) -> syn::Result<Vec<syn::Path>> {
    let mut merge_defaults = true;
    let mut enum_derives: Vec<syn::Path> = Vec::new();
    for ts in derive_attrs_ts {
        // For each row, collect derive attributes into `enum_derives` and look for `no_defaults` flag
        let mut iter = ts.into_iter().peekable();
        while let Some(tt) = iter.next() {
            match tt {
                proc_macro2::TokenTree::Ident(id) => {
                    if id == "no_defaults" {
                        if matches!(
                            iter.peek(),
                            Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '='
                        ) {
                            iter.next(); // consume `=`
                            merge_defaults = match iter.next() {
                                Some(proc_macro2::TokenTree::Ident(val)) if val == "true" => true,
                                Some(proc_macro2::TokenTree::Ident(val)) if val == "false" => false,
                                Some(unexpected) => {
                                    return Err(syn::Error::new_spanned(
                                        &unexpected,
                                        "expected `true` or `false` for `no_defaults` flag",
                                    ));
                                }
                                None => {
                                    return Err(syn::Error::new(
                                        proc_macro2::Span::call_site(),
                                        "unexpected end of input: expected `true` or `false` after `no_defaults =`",
                                    ));
                                }
                            };
                        } else {
                            merge_defaults = true; // bare flag
                        }
                    } else {
                        // Consume `::Ident` segments to build a full path
                        let mut segments =
                            syn::punctuated::Punctuated::<syn::PathSegment, syn::Token![::]>::new();
                        segments.push(syn::PathSegment::from(id));
                        while matches!(
                            iter.peek(),
                            Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ':'
                        ) {
                            iter.next(); // consume first `:`
                            iter.next(); // consume second `:`
                            match iter.next() {
                                Some(proc_macro2::TokenTree::Ident(next_id)) => {
                                    segments.push(syn::PathSegment::from(next_id));
                                }
                                Some(unexpected) => {
                                    return Err(syn::Error::new_spanned(
                                        &unexpected,
                                        "expected identifier after `::`",
                                    ));
                                }
                                None => {
                                    return Err(syn::Error::new(
                                        proc_macro2::Span::call_site(),
                                        "unexpected end of input: expected identifier after `::`",
                                    ));
                                }
                            }
                        }
                        enum_derives.push(syn::Path {
                            leading_colon: None,
                            segments,
                        });
                    }
                }
                proc_macro2::TokenTree::Punct(_) => {}
                unexpected => {
                    return Err(syn::Error::new_spanned(
                        &unexpected,
                        "unexpected token in derive attribute",
                    ));
                }
            }
        }
    }
    if merge_defaults {
        for &d_derive in DEFAULT_DERIVES {
            if !enum_derives.iter().any(|p| path_to_string(p) == d_derive) {
                let d_path: syn::Path =
                    syn::parse_str(d_derive).expect("invalid default derive path");
                enum_derives.push(d_path);
            }
        }
    }
    Ok(enum_derives)
}

/// Wraps body in a this_step macro_rules
fn make_step_macro(this_step: &Ident, body: TokenStream2) -> TokenStream2 {
    quote! {
        #[doc(hidden)]
        macro_rules! #this_step {
            ($callback:tt; $($acc:tt)*) => { #body };
        }
    }
}
