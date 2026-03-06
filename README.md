<div align="center">
      <h1>Struct to enum</h1>
    <h3>Generate enums from struct fields — ideal for type-safe query column filtering!</h3>

[![Rust](https://github.com/sinder38/struct-to-enum/actions/workflows/ci.yml/badge.svg)](https://github.com/sinder38/struct-to-enum/actions/workflows/ci.yml)
[![Latest Version](https://img.shields.io/crates/v/struct-to-enum.svg)](https://crates.io/crates/struct-to-enum)
[![Rust Documentation](https://docs.rs/struct-to-enum/badge.svg)](https://docs.rs/struct-to-enum)
![Crates.io](https://img.shields.io/crates/d/struct-to-enum)
  <br>Support by giving your ⭐!

</div>

## Overview
`struct-to-enum` crate provides two derive macros for generating enums corresponding to the fields of structs:

- **`FieldName`** — generates a unit enum where each variant represents a field name
- **`FieldType`** — generates an enum where each variant wraps the type of a struct field

Both macros support generics, field skipping, nesting, and custom derives/attributes on the generated enum.

Intended use-case is to generate enums from struct fields, enabling compile-time column filter validation for queries.

## Quick Start
```toml
[dependencies]
struct-to-enum = {version="1.0", features = ["derive"] }
```

## Usage

### `FieldName`

Generates a unit enum `{StructName}FieldName` and implements [`FieldNames<N>`] on the struct,
which provides a `field_names()` static method returning an ordered array of all variants.

```rust
use struct_to_enum::{FieldName, FieldNames};

#[derive(FieldName)]
struct Point {
    x: f32,
    y: f32,
    #[stem_name(skip)]
    label: String,
}

// Generated:
// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// enum PointFieldName { X, Y }

let names: [PointFieldName; 2] = Point::field_names();
assert_eq!(names, [PointFieldName::X, PointFieldName::Y]);
```

#### Nested flattening

Mark a field with `#[stem_name(nested)]` to flatten its `FieldName` variants into the parent enum.
The nested struct must also derive `FieldName`. Nesting can be arbitrarily deep.

```rust
use struct_to_enum::{FieldName, FieldNames};

#[derive(FieldName)]
pub struct Address {
    pub street: String,
    pub city: String,
}

#[derive(FieldName)]
struct Person {
    name: String,
    #[stem_name(nested)]
    address: Address,
}

// PersonFieldName has variants: Name, Street, City  (Address's variants are inlined)

let names: [PersonFieldName; 3] = Person::field_names();
assert_eq!(names, [PersonFieldName::Name, PersonFieldName::Street, PersonFieldName::City]);
```

### `FieldType`

Generates a tuple-variant enum `{StructName}FieldType` and implements `From<Struct>` for
`[FieldType; N]`, converting the struct into an ordered array of variants holding the field values.

```rust
use struct_to_enum::FieldType;

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Config {
    width: u32,
    height: u32,
    #[stem_type(skip)]
    name: String,
}

// Generated:
// #[derive(Debug, Clone, PartialEq)]
// enum ConfigFieldType {
//     Width(u32),
//     Height(u32),
// }

let cfg = Config { width: 1920, height: 1080, name: "hd".into() };
let fields: [ConfigFieldType; 2] = cfg.into();
assert_eq!(fields[0], ConfigFieldType::Width(1920));
assert_eq!(fields[1], ConfigFieldType::Height(1080));
```

#### Nested flattening

Mark a field with `#[stem_type(nested)]` to flatten the nested struct's `FieldType` variants
into the parent enum. The nested struct must also derive `FieldType`. Nesting can be arbitrarily deep.

```rust
use struct_to_enum::FieldType;

#[derive(FieldType)]
#[stem_type_derive(Debug, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, PartialEq)]
struct Pixel {
    x: i32,
    y: i32,
    #[stem_type(nested)]
    color: Color,
}

// PixelFieldType has variants: X(i32), Y(i32), R(u8), G(u8), B(u8)

let p = Pixel { x: 10, y: 20, color: Color { r: 255, g: 128, b: 0 } };
let fields: [PixelFieldType; 5] = p.into();
assert_eq!(fields[0], PixelFieldType::X(10));
assert_eq!(fields[2], PixelFieldType::R(255));
```

#### Generics

Both macros handle generic structs with lifetime and type parameters.

```rust
use struct_to_enum::FieldType;

#[derive(FieldType)]
#[stem_type_derive(Debug, PartialEq)]
struct Pair<A, B> {
    first: A,
    second: B,
}

let fields: [PairFieldType<i32, &str>; 2] = Pair { first: 1_i32, second: "hello" }.into();
assert_eq!(fields[0], PairFieldType::First(1));
```

### Combining both macros

```rust
use struct_to_enum::{FieldName, FieldNames, FieldType};

#[derive(FieldName, FieldType)]
#[stem_type_derive(Debug)]
struct Record {
    id: u64,
    value: f64,
}

// RecordFieldName and RecordFieldType are both generated
let _names: [RecordFieldName; 2] = Record::field_names();
```

## Attributes

### `FieldName` attributes

| Attribute | Location | Description |
|---|---|---|
| `#[stem_name_derive(...)]` | Struct | Merge additional derives onto the generated enum (defaults `Debug, PartialEq, Eq, Clone, Copy` are always kept) |
| `#[stem_name_attr(...)]` | Struct | Add extra attributes to the generated enum |
| `#[stem_name(skip)]` | Field | Exclude this field from the generated enum |
| `#[stem_name(nested)]` | Field | Flatten the field's `FieldName` variants into the parent enum |

### `FieldType` attributes

| Attribute | Location | Description |
|---|---|---|
| `#[stem_type_derive(...)]` | Struct | Specify derives for the generated enum (none by default) |
| `#[stem_type_attr(...)]` | Struct | Add extra attributes to the generated enum |
| `#[stem_type(skip)]` | Field | Exclude this field from the generated enum |
| `#[stem_type(nested)]` | Field | Flatten the field's `FieldType` variants into the parent enum |

> **Attribute aliases:** `stem_name` / `ste_name` and `stem_type` / `ste_type` are interchangeable.

## Example: using with `strum`

```rust
use struct_to_enum::FieldType;
use strum::VariantNames;

#[derive(FieldType)]
#[stem_type_derive(Debug, strum_macros::VariantNames)]
#[stem_type_attr(strum(serialize_all = "SCREAMING-KEBAB-CASE"))]
struct Request {
    user_id: u64,
    payload: Vec<u8>,
}

assert_eq!(
    RequestFieldType::VARIANTS,
    &["USER-ID", "PAYLOAD"]
);
```

## Related Crates

- [field_types](https://github.com/XX/field_types) - the original inspiration for this crate.
  `struct-to-enum` extends the idea with nesting, derive attributes, generics, and more.
- [strum](https://github.com/Peternator7/strum) - solves a similar problem (enum utilities via derive macros);
  a useful companion crate and a reference during development.
  
#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in struct-to-enum by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
