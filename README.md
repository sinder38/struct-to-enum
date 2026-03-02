# struct-to-enum

Derive macros for generating enums corresponding to the fields of structs.

## Overview

`struct-to-enum` provides two derive macros:

- **`FieldName`** — generates a unit enum where each variant represents a field name
- **`FieldType`** — generates an enum where each variant wraps the type of a struct field

Both macros support generics, field skipping, nesting (yes, you read that right) and custom derives/attributes on the generated enum.

Intended use-case is to generate  enums from struct fields, enabling compile-time column filter validation for queries.

## Usage

### `FieldName`

Generates a unit enum `{StructName}FieldName` and a `From<&Struct>` impl that returns an array of variants in field order.

```rust
use struct_to_enum::FieldName;

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

let p = Point { x: 1.0, y: 2.0, label: "origin".into() };
let names: [PointFieldName; 2] = (&p).into();
assert_eq!(names, [PointFieldName::X, PointFieldName::Y]);
```

#### Nested flattening

Mark a field with `#[stem_name(nested)]` to flatten its `FieldName` variants into the parent enum.

```rust
use struct_to_enum::FieldName;

#[derive(FieldName)]
struct Inner {
    x: i32,
    y: i32,
}

#[derive(FieldName)]
struct Outer {
    a: bool,
    #[stem_name(nested)]
    inner: Inner,
}

// OuterFieldName has variants: A, X, Y
let o = Outer { a: true, inner: Inner { x: 0, y: 1 } };
let names: [OuterFieldName; 3] = (&o).into();
assert_eq!(names, [OuterFieldName::A, OuterFieldName::X, OuterFieldName::Y]);
```

### `FieldType`

Generates a tuple-variant enum `{StructName}FieldType` and a `From<Struct>` impl that returns an array of variants holding the field values.

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
```

#### Generics

Both macros handle generic structs with lifetime and type parameters.

```rust
use struct_to_enum::FieldType;

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Pair<A, B> {
    first: A,
    second: B,
}

let p = Pair { first: 1_i32, second: "hello" };
let fields: [PairFieldType<i32, &str>; 2] = p.into();
```

### Combining both macros

```rust
use struct_to_enum::{FieldName, FieldType};

#[derive(FieldName, FieldType)]
#[stem_type_derive(Debug)]
struct Record {
    id: u64,
    value: f64,
}

// RecordFieldName and RecordFieldType are both generated
```

## Attributes

### `FieldName` attributes

| Attribute | Location | Description |
|---|---|---|
| `#[stem_name_derive(...)]` | Struct | Override derives on the generated enum (default: `Debug, PartialEq, Eq, Clone, Copy`) |
| `#[stem_name_attr(...)]` | Struct | Add extra attributes to the generated enum |
| `#[stem_name(skip)]` | Field | Exclude this field from the generated enum |
| `#[stem_name(nested)]` | Field | Flatten the field's `FieldName` variants into the parent enum |

### `FieldType` attributes

| Attribute | Location | Description |
|---|---|---|
| `#[stem_type_derive(...)]` | Struct | Specify derives for the generated enum |
| `#[stem_type_attr(...)]` | Struct | Add extra attributes to the generated enum |
| `#[stem_type(skip)]` | Field | Exclude this field from the generated enum |

> **Attribute aliases:** `stem_name` / `ste_name` and `stem_type` / `ste_type` are interchangeable.

## Example: using with `strum`

```rust
use struct_to_enum::FieldType;
use strum_macros::VariantNames;

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

## Inspired by:

[Strum](https://github.com/Peternator7/strum)
[Field Types](https://github.com/XX/field_types)

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option
