use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Ident, Type};

use crate::common::{FieldInfo, extract_type_ident, filter_fields, get_meta_list};

#[inline]
fn get_helper_macro_name(type_snake: &str) -> Ident {
    Ident::new(
        &format!("__{}_field_type_variants", type_snake),
        Span::call_site(),
    )
}

struct NormalField {
    variant_ident: Ident,
    field_ident: Ident,
    field_ty: Type,
}

/// A single field slot in declaration order
enum FieldSlot {
    /// One or more consecutive regular fields: (variant_ident, field_ident, field_type)
    Regular(Vec<NormalField>),
    /// A nested field - calls to the inner type's helper macro
    Nested {
        helper_macro: Ident,
        field_ident: Ident,
    },
}

pub struct DeriveFieldType {
    vis: syn::Visibility,
    ident: Ident,
    enum_ident: Ident,
    generics: syn::Generics,
    derive_attr: Vec<TokenStream2>,
    extra_attrs: Vec<TokenStream2>,
    ///
    slots: Vec<FieldSlot>,
    // fields: Vec<FieldInfo>,
    type_snake: String,
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

        let type_snake = ident.to_string().to_snake_case();
        // PERF: merge filte_fields and slot conversion.

        // Build declaration-order slots
        let mut slots: Vec<FieldSlot> = Vec::new();
        for f in fields {
            if f.is_nested {
                let inner_type_ident = extract_type_ident(&f.field_ty)?;
                let inner_snake = inner_type_ident.to_string().to_snake_case();
                slots.push(FieldSlot::Nested {
                    helper_macro: get_helper_macro_name(&inner_snake),
                    field_ident: f.field_ident.clone(),
                });
            } else {
                // Form current field
                let field = NormalField {
                    variant_ident: f.variant_ident.clone(),
                    field_ident: f.field_ident.clone(),
                    field_ty: f.field_ty.clone(),
                };

                // Append to last slot if or create a new one if It's Nested
                if let Some(FieldSlot::Regular(triples)) = slots.last_mut() {
                    triples.push(field);
                } else {
                    slots.push(FieldSlot::Regular(vec![field]));
                }
            }
        }

        Ok(Self {
            vis,
            ident,
            enum_ident,
            generics,
            derive_attr,
            extra_attrs,
            slots,
            type_snake,
        })
    }

    pub fn expand(self) -> syn::Result<TokenStream2> {
        let has_nested = self
            .slots
            .iter()
            .any(|s| matches!(s, FieldSlot::Nested { .. }));
        if has_nested {
            self.expand_nested()
        } else {
            self.expand_simple()
        }
    }

    fn expand_simple(self) -> syn::Result<TokenStream2> {
        let enum_def = self.enum_definition();
        let converter = self.converter_impl();

        let type_snake = &self.type_snake;
        let helper_macro_name = get_helper_macro_name(type_snake);

        let a = self.get_fields_for_simple();

        // Emitting here for simplicity
        let entries = a.into_iter().map(|f| {
            let NormalField {
                variant_ident,
                field_ty,
                field_ident,
            } = f;
            quote! { #variant_ident(#field_ty) { $($pfx)* . #field_ident }, }
        });

        let own_helper = quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #helper_macro_name {
                ($callback:tt; { $($pfx:tt)* }; $($acc:tt)*) => {
                    $callback!{$($acc)* #(#entries)*}
                };
            }
        };

        Ok(quote! {
            #enum_def
            #converter
            #own_helper
        })
    }

    fn get_fields_for_simple(&self) -> &Vec<NormalField> {
        //PERF: this function is called several times instead of once
        debug_assert_eq!(self.slots.len(), 1);
        match &self.slots[..] {
            [FieldSlot::Regular(p)] => p,
            _ => unreachable!("expand_simple called with nested slots"),
        }
    }

    fn expand_nested(&self) -> syn::Result<TokenStream2> {
        let (_, ty_generics, _) = self.generics.split_for_impl();
        let type_snake = &self.type_snake;

        let builder_macro_name = Ident::new(
            &format!("__{}_field_type_build", type_snake),
            Span::call_site(),
        );

        let builder_macro =
            self.generate_field_type_builder_macro(&builder_macro_name, &ty_generics);

        // Generate step macros
        let num_slots = self.slots.len();
        let step_name = |i: usize| {
            Ident::new(
                &format!("__{}_field_type_step_{}", type_snake, i),
                Span::call_site(),
            )
        };

        let mut step_macros: Vec<TokenStream2> = Vec::new();
        for (i, slot) in self.slots.iter().enumerate() {
            let this_step = step_name(i);
            let is_last = i == num_slots - 1;

            match slot {
                FieldSlot::Regular(triples) => {
                    let entries = triples.iter().map(
                        |NormalField {
                             variant_ident,
                             field_ident,
                             field_ty,
                         }| {
                            quote! { #variant_ident(#field_ty) { $($pfx)* . #field_ident }, }
                        },
                    );

                    if is_last {
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; { $($pfx:tt)* }; $($acc:tt)*) => {
                                    $callback!{$($acc)* #(#entries)*}
                                };
                            }
                        });
                    } else {
                        let next = step_name(i + 1);
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; { $($pfx:tt)* }; $($acc:tt)*) => {
                                    #next!{$callback; { $($pfx)* }; $($acc)* #(#entries)*}
                                };
                            }
                        });
                    }
                }
                FieldSlot::Nested {
                    helper_macro,
                    field_ident,
                } => {
                    if is_last {
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; { $($pfx:tt)* }; $($acc:tt)*) => {
                                    #helper_macro!{$callback; { $($pfx)* . #field_ident }; $($acc)*}
                                };
                            }
                        });
                    } else {
                        let next = step_name(i + 1);
                        step_macros.push(quote! {
                            #[doc(hidden)]
                            macro_rules! #this_step {
                                ($callback:tt; { $($pfx:tt)* }; $($acc:tt)*) => {
                                    #helper_macro!{#next; { $($pfx)* . #field_ident }; $callback; { $($pfx)* }; $($acc)*}
                                };
                            }
                        });
                    }
                }
            }
        }

        // Kick off: step 0 with builder as callback, empty prefix
        let step_0 = step_name(0);
        let invocation = quote! { #step_0!{#builder_macro_name; {}; } };

        // Own helper for when this type is nested in a grandparent
        let helper_macro_name = get_helper_macro_name(type_snake);
        let own_helper = quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #helper_macro_name {
                ($callback:tt; { $($pfx:tt)* }; $($acc:tt)*) => {
                    #step_0!{$callback; { $($pfx)* }; $($acc)*}
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

    /// Terminal builder macro: receives all `Variant(Type) { .path },` entries
    /// and generates the enum and From impls.
    /// The path entries are like `.a` or `.inner.x` - the builder prepends `source`.
    fn generate_field_type_builder_macro(
        &self,
        macro_name: &Ident,
        ty_generics: &syn::TypeGenerics,
    ) -> TokenStream2 {
        let vis = &self.vis;
        let enum_ty = &self.enum_ident;
        let derive = &self.derive_attr;
        let attrs = &self.extra_attrs;
        let ty = &self.ident;
        let (impl_generics, _, where_clause) = self.generics.split_for_impl();

        if self.generics.params.is_empty() {
            quote! {
                #[doc(hidden)]
                macro_rules! #macro_name {
                    ($($variant:ident ( $vty:ty ) { $($path:tt)* },)*) => {
                        #[derive(#(#derive),*)]
                        #(#[#attrs])*
                        #vis enum #enum_ty {
                            $($variant($vty)),*
                        }

                        impl From<#ty> for [#enum_ty; { let mut _n = 0usize; $({ let _ = stringify!($variant); _n += 1; })* _n }] {
                            fn from(source: #ty) -> Self {
                                [$(#enum_ty::$variant(source $($path)*)),*]
                            }
                        }
                    };
                }
            }
        } else {
            quote! {
                #[doc(hidden)]
                macro_rules! #macro_name {
                    ($($variant:ident ( $vty:ty ) { $($path:tt)* },)*) => {
                        #[derive(#(#derive),*)]
                        #(#[#attrs])*
                        #vis enum #enum_ty #ty_generics {
                            $($variant($vty)),*
                        }

                        impl #impl_generics Into<[#enum_ty #ty_generics; { let mut _n = 0usize; $({ let _ = stringify!($variant); _n += 1; })* _n }]> for #ty #ty_generics
                            #where_clause
                        {
                            fn into(self) -> [#enum_ty #ty_generics; { let mut _n = 0usize; $({ let _ = stringify!($variant); _n += 1; })* _n }] {
                                let source = self;
                                [$(#enum_ty::$variant(source $($path)*)),*]
                            }
                        }
                    };
                }
            }
        }
    }

    fn enum_definition(&self) -> TokenStream2 {
        let Self {
            vis,
            enum_ident,
            generics,
            derive_attr,
            extra_attrs,
            ..
        } = self;

        let (_, _, where_clause) = generics.split_for_impl();

        let fields = self.get_fields_for_simple();

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
            ..
        } = self;
        let fields = self.get_fields_for_simple();

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
            ..
        } = self;

        let fields = self.get_fields_for_simple();

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
