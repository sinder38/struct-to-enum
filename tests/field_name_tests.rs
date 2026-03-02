#![allow(dead_code)]

extern crate struct_to_enum;
extern crate strum;
extern crate strum_macros;

use struct_to_enum::FieldName;
use strum_macros::EnumString;

#[derive(FieldName)]
struct Test {
    first: i32,
    second_field: Option<String>,
    #[stem_name(skip)]
    third: bool,
    #[stem_name = "skip"]
    fourth: bool,
}

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct TestGen<'a, T: 'a, U>
where
    U: 'a,
{
    first: T,
    second_field: Option<&'a U>,
    #[stem_name(skip)]
    third: &'a T,
    #[stem_name = "skip"]
    fourth: U,
}

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct TestTypesDerive {
    first: i32,
    second: bool,
}

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct TestNameDerive {
    first: i32,
    second: bool,
}

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
#[stem_name_attr(strum(ascii_case_insensitive))]
struct TestDeriveArguments {
    first: i32,
    second_field: bool,
}

#[derive(FieldName, serde::Serialize)]
#[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
struct TestEmpty {
    first: i32,
    second_field: bool,
}

#[test]
fn full_field_name_variants() {
    let _field = TestFieldName::First;
    let field = TestFieldName::SecondField;
    match field {
        TestFieldName::First => (),
        TestFieldName::SecondField => (),
    }

    let _field = TestGenFieldName::First;
    let field = TestGenFieldName::SecondField;
    match field {
        TestGenFieldName::First => (),
        TestGenFieldName::SecondField => (),
    }

    let _field = TestTypesDeriveFieldName::First;
    let field = TestTypesDeriveFieldName::Second;
    match field {
        TestTypesDeriveFieldName::First => (),
        TestTypesDeriveFieldName::Second => (),
    }

    let _field = TestNameDeriveFieldName::First;
    let field = TestNameDeriveFieldName::Second;
    match field {
        TestNameDeriveFieldName::First => (),
        TestNameDeriveFieldName::Second => (),
    }
}

#[test]
fn derive_field_name() {
    let name = TestFieldName::First;
    assert_eq!(TestFieldName::First, name);
    assert_ne!(TestFieldName::SecondField, name);

    let name = TestGenFieldName::First;
    assert_eq!(TestGenFieldName::First, name);
    assert_ne!(TestGenFieldName::SecondField, name);

    let name = TestTypesDeriveFieldName::First;
    assert_eq!(TestTypesDeriveFieldName::First, name);
    assert_ne!(TestTypesDeriveFieldName::Second, name);

    let name = TestTypesDeriveFieldName::Second;
    assert_eq!(TestTypesDeriveFieldName::Second, name);
    assert_ne!(TestTypesDeriveFieldName::First, name);
}

#[test]
fn field_name_str() {
    assert_eq!(format!("{:?}", TestFieldName::First), "First");
    assert_eq!(format!("{:?}", TestFieldName::SecondField), "SecondField");
}

// --- Nested (flatten) FieldName tests ---
mod inners {
    use struct_to_enum::FieldName;
    #[derive(FieldName)]
    pub struct Inner {
        pub x: i32,
        pub y: String,
    }
}

use inners::*;

#[derive(FieldName)]
struct Outer {
    a: bool,
    #[stem_name(nested)]
    inner: Inner,
}

#[test]
fn nested_flat_field_name_variants() {
    // Inner's variants X and Y are flattened into Outer's enum
    let _a = OuterFieldName::A;
    let _x = OuterFieldName::X;
    let _y = OuterFieldName::Y;

    match OuterFieldName::X {
        OuterFieldName::A => panic!("wrong"),
        OuterFieldName::X => (),
        OuterFieldName::Y => panic!("wrong"),
    }
}

#[test]
fn nested_flat_field_name_array() {
    let outer = Outer {
        a: true,
        inner: Inner {
            x: 0,
            y: String::new(),
        },
    };
    let fields: [OuterFieldName; 3] = (&outer).into();
    assert_eq!(
        fields,
        [OuterFieldName::A, OuterFieldName::X, OuterFieldName::Y]
    );
}

#[derive(FieldName)]
struct DeepInner {
    z: f64,
}

#[derive(FieldName)]
struct DeepMiddle {
    m: i32,
    #[stem_name(nested)]
    deep: DeepInner,
}

#[derive(FieldName)]
struct DeepOuter {
    top: bool,
    #[stem_name(nested)]
    middle: DeepMiddle,
}

#[test]
fn nested_flat_deep() {
    // Multi-level nesting: DeepOuter flattens DeepMiddle which flattens DeepInner
    // Result: DeepOuterFieldName has Top, M, Z
    let outer = DeepOuter {
        top: true,
        middle: DeepMiddle {
            m: 0,
            deep: DeepInner { z: 0.0 },
        },
    };
    let fields: [DeepOuterFieldName; 3] = (&outer).into();
    assert_eq!(
        fields,
        [
            DeepOuterFieldName::Top,
            DeepOuterFieldName::M,
            DeepOuterFieldName::Z
        ]
    );
}

#[test]
fn field_name_fromstr() {
    assert_eq!(
        "FIRST".parse::<TestDeriveArgumentsFieldName>().unwrap(),
        TestDeriveArgumentsFieldName::First
    );
    assert_eq!(
        "first".parse::<TestDeriveArgumentsFieldName>().unwrap(),
        TestDeriveArgumentsFieldName::First
    );
    assert_eq!(
        "secondField"
            .parse::<TestDeriveArgumentsFieldName>()
            .unwrap(),
        TestDeriveArgumentsFieldName::SecondField
    );
}
