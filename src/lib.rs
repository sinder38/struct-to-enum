//! Derive macros that generate enums from struct fields.
//!
//! | Macro | Generated type | Conversion |
//! |-------|---------------|------------|
//! | [`FieldName`] | Unit enum — one variant per field name | `From<&Struct>` → `[FieldName; N]` |
//! | [`FieldType`] | Tuple enum — one variant per field value | `From<Struct>` → `[FieldType; N]` |

mod common;
mod field_name;
mod field_type;

use field_name::DeriveFieldName;
use field_type::DeriveFieldType;
use proc_macro::TokenStream;
use syn::DeriveInput;

/// Generates `{StructName}FieldType`, an enum whose variants wrap struct field values.
/// For enum variants without values use `FieldName` instead.
///
/// Each non-skipped field becomes a variant `VariantName(FieldType)` where the variant name is
/// the field name in `PascalCase`. The variants are ordered by field declaration order.
///
/// **No derives are added by default.** Add with `#[stem_type_derive(...)]`](#attributes).
/// ** Doesn't support nesting currently**
///
/// # Attributes
/// | Attribute | Target | Description |
/// |-----------|--------|-------------|
/// | `#[stem_type(skip)]` | field | Exclude this field from the generated enum. |
/// | `#[stem_type_derive(...)]` | struct | Derives for the generated enum. None are added by default. |
/// | `#[stem_type_attr(...)]` | struct | Extra attributes applied verbatim to the generated enum. |
///
/// All `stem_type*` attributes have short aliases: `ste_type`, `ste_type_derive`, `ste_type_attr`.
/// # Generated items
///
/// For a struct `Foo` with `N` non-skipped fields, this macro generates:
///
/// ```text
/// enum FooFieldType { Field1(T1), Field2(T2), ... }
///
/// impl From<Foo>          for [FooFieldType; N] { ... }
/// impl Into<[FooFieldType; N]> for Foo          { ... }
/// ```
///
/// # Example
///
/// ```rust
/// use struct_to_enum::FieldType;
///
/// #[derive(Clone)]
/// #[derive(FieldType)]
/// #[stem_type_derive(Debug, PartialEq, Clone)]
/// struct Config {
///     width: u32,
///     height: u32,
///     #[stem_type(skip)]
///     name: String,
/// }
///
/// // Generated: enum ConfigFieldType { Width(u32), Height(u32) }
///
/// let cfg = Config { width: 1920, height: 1080, name: "hd".into() };
///
/// // Three equivalent conversions:
/// let by_into:   [ConfigFieldType; 2] = cfg.clone().into();
/// let by_from:   [ConfigFieldType; 2] = <[ConfigFieldType; 2]>::from(cfg.clone());
///
/// assert_eq!(by_from[0], ConfigFieldType::Width(1920));
/// assert_eq!(by_from[1], ConfigFieldType::Height(1080));
/// ```
///
/// # Generics
///
/// Generic structs are supported. The generated enum carries the same type parameters:
///
/// ```rust
/// use struct_to_enum::FieldType;
///
/// #[derive(FieldType)]
/// #[stem_type_derive(Debug, PartialEq)]
/// struct Pair<A, B> {
///     first: A,
///     second: B,
/// }
///
/// // Generated: enum PairFieldType<A, B> { First(A), Second(B) }
///
/// let fields: [PairFieldType<i32, &str>; 2] = Pair { first: 42_i32, second: "hi" }.into();
/// assert_eq!(fields[0], PairFieldType::First(42));
/// ```
///
/// # Combining with other derives
///
/// Use `#[stem_type_derive]` and `#[stem_type_attr]` to pass anything to the generated enum.
/// This works with crates like [`strum`](https://docs.rs/strum):
///
/// ```rust
/// use struct_to_enum::FieldType;
/// use strum::VariantNames;
///
/// #[derive(FieldType)]
/// #[stem_type_derive(Debug, strum_macros::VariantNames)]
/// #[stem_type_attr(strum(serialize_all = "SCREAMING-KEBAB-CASE"))]
/// struct Request {
///     user_id: u64,
///     payload: Vec<u8>,
/// }
///
/// assert_eq!(RequestFieldType::VARIANTS, ["USER-ID", "PAYLOAD"]);
/// ```
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

/// Generates `{StructName}FieldName`, an enum whose variants represent struct field names.
/// For enum variants with values use `FieldType` instead.
///
/// Each non-skipped field becomes a unit variant in `PascalCase`. The variants are ordered by
/// field declaration order.
///
/// The generated enum derives `Debug`, `PartialEq`, `Eq`, `Clone`, and `Copy` by default.
/// Add more this with [`#[stem_name_derive(...)]`](#attributes).
///
/// # Attributes
/// | Attribute | Target | Description |
/// |-----------|--------|-------------|
/// | `#[stem_name(skip)]` | field | Exclude this field from the generated enum. |
/// | `#[stem_name(nested)]` | field | Flatten the nested struct's `FieldName` variants into this enum. |
/// | `#[stem_name_derive(...)]` | struct | Override derives on the generated enum (replaces the defaults). |
/// | `#[stem_name_attr(...)]` | struct | Extra attributes applied verbatim to the generated enum. |
///
/// All `stem_name*` attributes have short aliases: `ste_name`, `ste_name_derive`, `ste_name_attr`.
///
/// Additinaly generated enum implements a conversion into from it's struct into all its enum varints in an array.
/// ```rust,ignore
/// impl From<&Foo> for [FooFieldName; N] { ... }
/// ```
///
/// # Example
///
/// ```rust
/// use struct_to_enum::FieldName;
///
/// #[derive(FieldName)]
/// struct User {
///     id: u64,
///     user_name: String,
///     #[stem_name(skip)]
///     internal_token: String,
/// }
///
/// // Generated: enum UserFieldName { Id, UserName }  (Debug, PartialEq, Eq, Clone, Copy)
///
/// let user = User { id: 1, user_name: "alice".into(), internal_token: "x".into() };
/// let names: [UserFieldName; 2] = (&user).into();
/// assert_eq!(names, [UserFieldName::Id, UserFieldName::UserName]);
/// ```
///
/// # Flattening nested structs
///
/// Mark a field with `#[stem_name(nested)]` to inline the variants of a nested struct
/// (which must also derive `FieldName`) directly into the parent enum. Nesting can be
/// arbitrarily deep.
///
/// ```rust
/// use struct_to_enum::FieldName;
///
/// #[derive(FieldName)]
/// pub struct Address {
///     pub street: String,
///     pub city: String,
/// }
///
/// #[derive(FieldName)]
/// struct Person {
///     name: String,
///     #[stem_name(nested)]
///     address: Address,
/// }
///
/// // PersonFieldName: Name, Street, City  (Address's variants are inlined)
///
/// let p = Person {
///     name: "Bob".into(),
///     address: Address { street: "1 Main St".into(), city: "NY".into() },
/// };
/// let fields: [PersonFieldName; 3] = (&p).into();
/// assert_eq!(fields, [PersonFieldName::Name, PersonFieldName::Street, PersonFieldName::City]);
/// ```
///
/// # Generics
///
/// Generic structs are supported. The `From` impl carries the same bounds:
///
/// ```rust
/// use struct_to_enum::FieldName;
///
/// #[derive(FieldName)]
/// struct Pair<A, B> {
///     first: A,
///     second: B,
/// }
///
/// // Generated: enum PairFieldName { First, Second }
/// assert_eq!(PairFieldName::First, PairFieldName::First);
/// ```
///
/// # Combining with other derives
///
/// Use `#[stem_name_derive]` and `#[stem_name_attr]` to pass anything to the generated enum.
/// This works with crates like [`strum`](https://docs.rs/strum):
///
/// ```rust
/// use struct_to_enum::FieldName;
/// use strum_macros::EnumString;
///
/// #[derive(FieldName)]
/// #[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
/// #[stem_name_attr(strum(ascii_case_insensitive))]
/// struct Query {
///     user_id: u64,
///     status: String,
/// }
///
/// let variant: QueryFieldName = "userid".parse().unwrap(); // case-insensitive
/// assert_eq!(variant, QueryFieldName::UserId);
/// ```
///
/// # Generated items example
///
/// For a struct `Foo` with `N` non-skipped fields, the macro generates:
///
/// ```text
/// enum FooFieldName { Field1, Field2, ... }
///
/// ```
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
