use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::{DeriveInput, Ident};

use crate::common::{FieldInfo, extract_type_ident, filter_fields, get_enum_derive, get_meta_list};

struct NestedFieldInfo {
    /// The field ident in the parent struct
    field_ident: Ident,
    /// The `__typename_field_type_variants` helper macro of the nested type
    helper_macro: Ident,
}

pub struct DeriveFieldType {
    vis: syn::Visibility,
    ident: Ident,
    enum_ident: Ident,
    generics: syn::Generics,
    derive_attr: TokenStream2,
    extra_attrs: Vec<TokenStream2>,
    /// Non-nested fields only
    regular_fields: Vec<FieldInfo>,
    /// Nested fields
    nested_fields: Vec<NestedFieldInfo>,
    type_snake: String,
}

impl DeriveFieldType {
    pub fn new(input: DeriveInput) -> syn::Result<Self> {
        let vis = input.vis;
        let ident = input.ident;
        let generics = input.generics;
        let enum_ident = Ident::new(&(ident.to_string() + "FieldType"), Span::call_site());

        // TODO: replace with get_meta_list
        let derive_attr = get_enum_derive(
            &input.attrs,
            &["stem_type_derive", "ste_type_derive"],
            quote! {},
        );
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

        let regular_fields: Vec<FieldInfo> =
            fields.iter().filter(|f| !f.is_nested).cloned().collect();
        let nested_raw: Vec<&FieldInfo> = fields.iter().filter(|f| f.is_nested).collect();

        let type_snake = ident.to_string().to_snake_case();

        let mut nested_fields = Vec::new();
        for f in &nested_raw {
            let inner_type_ident = extract_type_ident(&f.field_ty)?;
            let inner_snake = inner_type_ident.to_string().to_snake_case();
            nested_fields.push(NestedFieldInfo {
                field_ident: Ident::new(&f.field_ident.to_string(), Span::call_site()),
                helper_macro: Ident::new(
                    &format!("__{}_field_type_variants", inner_snake),
                    Span::call_site(),
                ),
            });
        }

        Ok(Self {
            vis,
            ident,
            enum_ident,
            generics,
            derive_attr,
            extra_attrs,
            regular_fields,
            nested_fields,
            type_snake,
        })
    }

    pub fn expand(&self) -> syn::Result<TokenStream2> {
        if self.nested_fields.is_empty() {
            self.expand_simple()
        } else {
            self.expand_nested()
        }
    }

    fn expand_simple(&self) -> syn::Result<TokenStream2> {
        let enum_def = self.simple_enum_definition();
        let converter = self.simple_converter_impl();
        let helper_macro = self.generate_own_helper_simple();

        Ok(quote! {
            #enum_def
            #converter
            #helper_macro
        })
    }

    fn simple_enum_definition(&self) -> TokenStream2 {
        let Self {
            vis,
            enum_ident,
            generics,
            derive_attr,
            extra_attrs,
            regular_fields,
            ..
        } = self;

        let (_, _, where_clause) = generics.split_for_impl();

        let variants = regular_fields.iter().map(|f| {
            let variant = &f.variant_ident;
            let ty = &f.field_ty;
            quote! { #variant(#ty) }
        });

        quote! {
            #derive_attr
            #(#[#extra_attrs])*
            #vis enum #enum_ident #generics
                #where_clause
            {
                #(#variants),*
            }
        }
    }

    fn simple_converter_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            enum_ident,
            generics,
            regular_fields,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let fields_count = regular_fields.len();

        let constructs = regular_fields.iter().map(|f| {
            let field = &f.field_ident;
            let variant = &f.variant_ident;
            quote! { #enum_ident::#variant(#field) }
        });

        let field_idents = regular_fields.iter().map(|f| {
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

    /// Emit the `__typename_field_type_variants!` helper macro for a leaf struct.
    ///
    /// # Token format (shared by all helpers and the builder)
    ///
    /// ```text
    /// callback!{ [ path_tokens ] ; Variant1(Type1) { expr1 } Variant2(Type2) { expr2 } ... }
    /// ```
    ///
    /// - `[ path_tokens ]` — the access path so far as a bracket-grouped `tt*` (e.g. `[source.field]`).
    ///   Using brackets makes it an unambiguous single `tt` at the call site.
    /// - Each `Variant(Type) { expr }` — one accumulated entry.
    ///
    /// # Helper invocation
    ///
    /// ```text
    /// helper!{ callback ; [ path_tokens ] ; existing_entries }
    /// ```
    ///
    /// The helper appends its own entries (formed by extending the current path with
    /// `.field_name`) and calls the callback in the unified format.
    fn generate_own_helper_simple(&self) -> TokenStream2 {
        let type_snake = &self.type_snake;
        let helper_macro_name = Ident::new(
            &format!("__{}_field_type_variants", type_snake),
            Span::call_site(),
        );

        // Each entry: `Variant(Type) [rel_path_tokens]`
        // rel_path_tokens is the relative path (without `source`) — e.g. `middle . deep . z`.
        // The builder macro prepends `source .` when constructing field access expressions.
        let variant_entries: Vec<TokenStream2> = self
            .regular_fields
            .iter()
            .map(|f| {
                let variant = &f.variant_ident;
                let ty = &f.field_ty;
                let field = Ident::new(&f.field_ident.to_string(), Span::call_site());
                quote! { #variant(#ty) [$($path)* . #field] }
            })
            .collect();

        quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #helper_macro_name {
                // `[ $($path:tt)* ]` — bracket-grouped relative path (e.g. `[middle . deep]`),
                //   unambiguous as a single tt.  Does NOT include `source`.
                // `; $($acc:tt)*`    — separator then accumulated variant entries.
                ($callback:path ; [ $($path:tt)* ] ; $($acc:tt)*) => {
                    $callback!{
                        [ $($path)* ] ;
                        $($acc)*
                        #(#variant_entries)*
                    }
                };
            }
        }
    }

    // ── Nested ───────────────────────────────────────────────────────────────

    fn expand_nested(&self) -> syn::Result<TokenStream2> {
        let type_snake = &self.type_snake;

        let builder_macro_name = Ident::new(
            &format!("__{}_field_type_build", type_snake),
            Span::call_site(),
        );
        let builder_macro = self.generate_builder_macro(&builder_macro_name);

        let chain_macros = self.generate_chain_macros(&builder_macro_name);

        let helper_macro_name = Ident::new(
            &format!("__{}_field_type_variants", type_snake),
            Span::call_site(),
        );
        let own_helper = self.generate_own_helper_nested(&helper_macro_name);

        let invocation = self.generate_invocation(&builder_macro_name);

        Ok(quote! {
            #builder_macro
            #(#chain_macros)*
            #own_helper
            #invocation
        })
    }

    /// The builder macro receives: `[ $($path:tt)* ] ; $($variant:ident($ty:ty) { $expr:expr })*`
    ///
    /// The path is ignored (it was only needed during chain traversal).
    /// Emits the enum definition and `From`/`Into` impl.
    fn generate_builder_macro(&self, macro_name: &Ident) -> TokenStream2 {
        let vis = &self.vis;
        let enum_ty = &self.enum_ident;
        let derive_attr = &self.derive_attr;
        let extra_attrs = &self.extra_attrs;
        let struct_ty = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let own_variant_defs: Vec<TokenStream2> = self
            .regular_fields
            .iter()
            .map(|f| {
                let variant = &f.variant_ident;
                let ty = &f.field_ty;
                quote! { #variant(#ty) }
            })
            .collect();

        // Own fields are accessed via `source . field_name` in the From/Into impl.
        let own_constructs: Vec<TokenStream2> = self
            .regular_fields
            .iter()
            .map(|f| {
                let variant = &f.variant_ident;
                let field = Ident::new(&f.field_ident.to_string(), Span::call_site());
                quote! { #enum_ty::#variant(source . #field) }
            })
            .collect();

        let mut ty_generics_tokens = TokenStream2::new();
        ty_generics.to_tokens(&mut ty_generics_tokens);

        if self.generics.params.is_empty() {
            quote! {
                #[doc(hidden)]
                macro_rules! #macro_name {
                    // Entry format: Variant(Type) [rel_path_tokens]
                    // rel_path_tokens: field access path WITHOUT `source` prefix
                    // We emit `source . $($rel)*` in the function body.
                    ([ $($_ignored:tt)* ] ; $($variant:ident($ty:ty) [$($rel:tt)*])*) => {
                        #derive_attr
                        #(#[#extra_attrs])*
                        #vis enum #enum_ty {
                            #(#own_variant_defs,)*
                            $($variant($ty)),*
                        }

                        impl From<#struct_ty> for [#enum_ty; {
                            let mut _n = 0usize;
                            #( { let _ = stringify!(#own_variant_defs); _n += 1; } )*
                            $({ let _ = stringify!($variant); _n += 1; })*
                            _n
                        }] {
                            fn from(source: #struct_ty) -> Self {
                                [#(#own_constructs,)* $(#enum_ty::$variant(source . $($rel)*)),*]
                            }
                        }
                    };
                }
            }
        } else {
            quote! {
                #[doc(hidden)]
                macro_rules! #macro_name {
                    ([ $($_ignored:tt)* ] ; $($variant:ident($ty:ty) [$($rel:tt)*])*) => {
                        #derive_attr
                        #(#[#extra_attrs])*
                        #vis enum #enum_ty #ty_generics_tokens {
                            #(#own_variant_defs,)*
                            $($variant($ty)),*
                        }

                        impl #impl_generics Into<[#enum_ty #ty_generics_tokens; {
                            let mut _n = 0usize;
                            #( { let _ = stringify!(#own_variant_defs); _n += 1; } )*
                            $({ let _ = stringify!($variant); _n += 1; })*
                            _n
                        }]> for #struct_ty #ty_generics_tokens
                            #where_clause
                        {
                            fn into(self) -> [#enum_ty #ty_generics_tokens; {
                                let mut _n = 0usize;
                                #( { let _ = stringify!(#own_variant_defs); _n += 1; } )*
                                $({ let _ = stringify!($variant); _n += 1; })*
                                _n
                            }] {
                                let source = self;
                                [#(#own_constructs,)* $(#enum_ty::$variant(source . $($rel)*)),*]
                            }
                        }
                    };
                }
            }
        }
    }

    /// Generate chain macros for multiple nested fields on the outer struct.
    ///
    /// Chain macro `i` receives `[ path ] ; acc` and calls nested helper `i` with
    /// `[ path . nested_field_i ] ; acc`.
    fn generate_chain_macros(&self, builder_macro_name: &Ident) -> Vec<TokenStream2> {
        let type_snake = &self.type_snake;
        let nested_helpers: Vec<&Ident> =
            self.nested_fields.iter().map(|f| &f.helper_macro).collect();
        let nested_fields: Vec<&Ident> =
            self.nested_fields.iter().map(|f| &f.field_ident).collect();
        let num_nested = nested_helpers.len();

        let mut chain_macros = Vec::new();

        for i in 1..num_nested {
            let chain_name = Ident::new(
                &format!("__{}_field_type_chain_{}", type_snake, i),
                Span::call_site(),
            );
            let nested_mac = nested_helpers[i];
            let nested_field = nested_fields[i];
            let next_target = if i == num_nested - 1 {
                builder_macro_name.clone()
            } else {
                Ident::new(
                    &format!("__{}_field_type_chain_{}", type_snake, i + 1),
                    Span::call_site(),
                )
            };

            // The chain macro ignores the previous path (from the preceding nested field)
            // and starts a fresh path `[ nested_field_name ]` for the next nested field.
            // All nested fields on the outer struct are direct children of that struct,
            // so their paths are independent.
            chain_macros.push(quote! {
                #[doc(hidden)]
                macro_rules! #chain_name {
                    ([ $($_prev_path:tt)* ] ; $($acc:tt)*) => {
                        #nested_mac!{
                            #next_target ;
                            [ #nested_field ] ;
                            $($acc)*
                        }
                    };
                }
            });
        }

        chain_macros
    }

    /// The initial invocation that kicks off the macro chain.
    fn generate_invocation(&self, builder_macro_name: &Ident) -> TokenStream2 {
        let type_snake = &self.type_snake;
        let nested_helpers: Vec<&Ident> =
            self.nested_fields.iter().map(|f| &f.helper_macro).collect();
        let nested_fields: Vec<&Ident> =
            self.nested_fields.iter().map(|f| &f.field_ident).collect();
        let num_nested = nested_helpers.len();

        let first_helper = nested_helpers[0];
        let first_field = nested_fields[0];

        let first_target = if num_nested == 1 {
            builder_macro_name.clone()
        } else {
            Ident::new(
                &format!("__{}_field_type_chain_1", type_snake),
                Span::call_site(),
            )
        };

        // Pass `[first_field]` as the initial relative path (no `source` prefix —
        // the builder prepends `source .` when constructing field access expressions).
        quote! {
            #first_helper!{ #first_target ; [ #first_field ] ; }
        }
    }

    /// Own helper macro for when THIS struct (which has nested fields) is used as a nested
    /// field in another struct.
    ///
    /// Called as: `helper!{ callback ; [ $($path:tt)* ] ; $($acc:tt)* }`
    ///
    /// Appends own regular-field entries, then chains into its own nested helper(s)
    /// to accumulate their entries, then calls the callback.
    fn generate_own_helper_nested(&self, helper_macro_name: &Ident) -> TokenStream2 {
        let type_snake = &self.type_snake;

        let own_variant_entries: Vec<TokenStream2> = self
            .regular_fields
            .iter()
            .map(|f| {
                let variant = &f.variant_ident;
                let ty = &f.field_ty;
                let field = Ident::new(&f.field_ident.to_string(), Span::call_site());
                quote! { #variant(#ty) [$($path)* . #field] }
            })
            .collect();

        let nested_helpers: Vec<&Ident> =
            self.nested_fields.iter().map(|f| &f.helper_macro).collect();
        let nested_fields: Vec<&Ident> =
            self.nested_fields.iter().map(|f| &f.field_ident).collect();
        let num_nested = nested_helpers.len();

        if num_nested == 0 {
            // Leaf case (shouldn't happen since we're in expand_nested, but for safety)
            quote! {
                #[doc(hidden)]
                #[macro_export]
                macro_rules! #helper_macro_name {
                    ($callback:path ; [ $($path:tt)* ] ; $($acc:tt)*) => {
                        $callback!{
                            [ $($path)* ] ;
                            $($acc)* #(#own_variant_entries)*
                        }
                    };
                }
            }
        } else if num_nested == 1 {
            let inner_mac = nested_helpers[0];
            let inner_field = nested_fields[0];
            quote! {
                #[doc(hidden)]
                #[macro_export]
                macro_rules! #helper_macro_name {
                    ($callback:path ; [ $($path:tt)* ] ; $($acc:tt)*) => {
                        #inner_mac!{
                            $callback ;
                            [ $($path)* . #inner_field ] ;
                            $($acc)* #(#own_variant_entries)*
                        }
                    };
                }
            }
        } else {
            // Multiple inner nested fields: use forwarding macros
            let mut fwd_macros = Vec::new();
            for i in 1..num_nested {
                let fwd_name = Ident::new(
                    &format!("__{}_field_type_helper_fwd_{}", type_snake, i),
                    Span::call_site(),
                );
                let nested_mac_i = nested_helpers[i];
                let field_i = nested_fields[i];

                if i == num_nested - 1 {
                    fwd_macros.push(quote! {
                        #[doc(hidden)]
                        macro_rules! #fwd_name {
                            ($callback:path ; [ $($base:tt)* ] ; $($acc:tt)*) => {
                                #nested_mac_i!{
                                    $callback ;
                                    [ $($base)* . #field_i ] ;
                                    $($acc)*
                                }
                            };
                        }
                    });
                } else {
                    let next_fwd = Ident::new(
                        &format!("__{}_field_type_helper_fwd_{}", type_snake, i + 1),
                        Span::call_site(),
                    );
                    fwd_macros.push(quote! {
                        #[doc(hidden)]
                        macro_rules! #fwd_name {
                            ($callback:path ; [ $($base:tt)* ] ; $($acc:tt)*) => {
                                #nested_mac_i!{
                                    #next_fwd ;
                                    [ $($base)* . #field_i ] ;
                                    $($acc)*
                                }
                            };
                        }
                    });
                }
            }

            let first_fwd = Ident::new(
                &format!("__{}_field_type_helper_fwd_1", type_snake),
                Span::call_site(),
            );
            let first_inner_mac = nested_helpers[0];
            let first_inner_field = nested_fields[0];

            quote! {
                #(#fwd_macros)*

                #[doc(hidden)]
                #[macro_export]
                macro_rules! #helper_macro_name {
                    ($callback:path ; [ $($path:tt)* ] ; $($acc:tt)*) => {
                        #first_inner_mac!{
                            #first_fwd ;
                            [ $($path)* . #first_inner_field ] ;
                            $($acc)* #(#own_variant_entries)*
                        }
                    };
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
            regular_fields,
            ..
        } = self;
        // Use From/Into instead

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let fields_count = regular_fields.len();

        let constructs = regular_fields.iter().map(|f| {
            let field = &f.field_ident;
            let variant = &f.variant_ident;
            quote! { #enum_ident::#variant(#field) }
        });

        let field_idents = regular_fields.iter().map(|f| {
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
