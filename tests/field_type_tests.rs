#![allow(dead_code)]

extern crate struct_to_enum;
extern crate strum;
extern crate strum_macros;

use struct_to_enum::FieldType;
use strum::VariantNames;

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Test {
    first: i32,
    second_field: Option<String>,
    #[stem_type(skip)]
    third: bool,
    #[stem_type = "skip"]
    fourth: bool,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct TestGen<'a, T: 'a, U>
where
    U: 'a,
{
    first: T,
    second_field: Option<&'a U>,
    #[stem_type(skip)]
    third: &'a T,
    #[stem_type = "skip"]
    fourth: U,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct TestTypesDerive {
    first: i32,
    second: bool,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct TestTypeDerive {
    first: i32,
    second: bool,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq, strum_macros::VariantNames)]
#[stem_type_attr(strum(serialize_all = "SCREAMING-KEBAB-CASE"))]
struct TestDeriveArguments {
    first: i32,
    second_field: bool,
}

#[test]
fn full_field_type_variants() {
    let _field = TestFieldType::First(2);
    let field = TestFieldType::SecondField(None);
    match field {
        TestFieldType::First(_) => (),
        TestFieldType::SecondField(_) => (),
    }

    let _field = TestGenFieldType::First::<_, &str>(2);
    let field = TestGenFieldType::SecondField::<i32, &str>(None);
    match field {
        TestGenFieldType::First(_) => (),
        TestGenFieldType::SecondField(_) => (),
    }

    let first_field = TestTypesDeriveFieldType::First(2);
    match first_field {
        TestTypesDeriveFieldType::First(_) => (),
        TestTypesDeriveFieldType::Second(_) => (),
    }

    let _field = TestTypeDeriveFieldType::First(2);
    let field = TestTypeDeriveFieldType::Second(false);
    match field {
        TestTypeDeriveFieldType::First(_) => (),
        TestTypeDeriveFieldType::Second(_) => (),
    }
}

#[test]
fn derive_field_type() {
    let first_field = TestGenFieldType::First::<_, &str>(2);
    let second_field = TestGenFieldType::SecondField::<i32, &str>(None);
    assert_ne!(first_field, second_field);
    assert_eq!(first_field, first_field.clone());
    assert_eq!("First(2)", format!("{:?}", first_field));

    let field = TestTypesDeriveFieldType::First(1).clone();
    assert_eq!(TestTypesDeriveFieldType::First(1), field);
    assert_ne!(TestTypesDeriveFieldType::First(2), field);
    assert_eq!("First(1)", format!("{:?}", field));

    let field = TestTypesDeriveFieldType::Second(true).clone();
    assert_eq!(TestTypesDeriveFieldType::Second(true), field);
    assert_ne!(TestTypesDeriveFieldType::Second(false), field);
    assert_eq!("Second(true)", format!("{:?}", field));

    let field = TestTypeDeriveFieldType::First(1).clone();
    assert_eq!(TestTypeDeriveFieldType::First(1), field);
    assert_ne!(TestTypeDeriveFieldType::First(2), field);
    assert_eq!("First(1)", format!("{:?}", field));

    let field = TestTypeDeriveFieldType::Second(true).clone();
    assert_eq!(TestTypeDeriveFieldType::Second(true), field);
    assert_ne!(TestTypeDeriveFieldType::Second(false), field);
    assert_eq!("Second(true)", format!("{:?}", field));
}

#[test]
fn into_field_type() {
    let test = Test {
        first: 1,
        second_field: Some("test".to_string()),
        third: true,
        fourth: true,
    };
    let fields: [TestFieldType; 2] = test.into();
    assert!(matches!(fields, [
        TestFieldType::First(1),
        TestFieldType::SecondField(Some(ref s)),
    ] if s == "test"));

    let message = "test".to_string();
    let test = TestGen {
        first: 1,
        second_field: Some(&message),
        third: &2,
        fourth: message.clone(),
    };
    let fields: [TestGenFieldType<i32, String>; 2] = test.into();
    assert!(matches!(fields, [
        TestGenFieldType::First(1),
        TestGenFieldType::SecondField(Some(s)),
    ] if s == &message));

    let test = TestTypesDerive {
        first: 1,
        second: true,
    };
    let fields: [TestTypesDeriveFieldType; 2] = test.into();
    assert_eq!(TestTypesDeriveFieldType::First(1), fields[0]);
    assert_eq!(TestTypesDeriveFieldType::Second(true), fields[1]);
}

#[test]
fn derive_macro_arguments() {
    assert_eq!(
        TestDeriveArgumentsFieldType::VARIANTS,
        ["FIRST", "SECOND-FIELD"]
    );
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DeepInner {
    z: f64,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DeepMiddle {
    m: i32,
    #[stem_type(nested)]
    deep: DeepInner,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DeepOuter {
    top: bool,
    #[stem_type(nested)]
    middle: DeepMiddle,
}

/// Single level of nesting
#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct InnerSimple {
    x: i32,
    y: i32,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct OuterSimple {
    a: bool,
    #[stem_type(nested)]
    inner: InnerSimple,
}

/// Two nested fields in the same struct (with distinct field names to avoid variant conflicts)
#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct LeftInner {
    lx: i32,
    ly: i32,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct RightInner {
    rx: i32,
    ry: i32,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct TwoNested {
    #[stem_type(skip)]
    skippa: String,
    own: u8,
    #[stem_type(nested)]
    left: LeftInner,
    #[stem_type(nested)]
    right: RightInner,
}

#[test]
fn nested_field_type_variants() {
    // Deep 2-level nesting: DeepOuter { top, middle: DeepMiddle { m, deep: DeepInner { z } } }
    // DeepOuterFieldType should have: Top(bool), M(i32), Z(f64)
    let _top = DeepOuterFieldType::Top(true);
    let _m = DeepOuterFieldType::M(42);
    let _z = DeepOuterFieldType::Z(3.14);
    let v: DeepOuterFieldType = DeepOuterFieldType::Top(false);
    match v {
        DeepOuterFieldType::Top(_) => (),
        DeepOuterFieldType::M(_) => (),
        DeepOuterFieldType::Z(_) => (),
    }
}

#[test]
fn nested_single_level_into() {
    let outer = OuterSimple {
        a: true,
        inner: InnerSimple { x: 1, y: 2 },
    };
    // OuterSimpleFieldType has: A(bool), X(i32), Y(i32)  — 3 variants
    let fields: [OuterSimpleFieldType; 3] = outer.into();
    assert_eq!(fields[0], OuterSimpleFieldType::A(true));
    assert_eq!(fields[1], OuterSimpleFieldType::X(1));
    assert_eq!(fields[2], OuterSimpleFieldType::Y(2));
}

#[test]
fn nested_two_levels_into() {
    let outer = DeepOuter {
        top: true,
        middle: DeepMiddle {
            m: 7,
            deep: DeepInner { z: 2.5 },
        },
    };
    // DeepOuterFieldType has: Top(bool), M(i32), Z(f64)  — 3 variants
    let fields: [DeepOuterFieldType; 3] = outer.into();
    assert_eq!(fields[0], DeepOuterFieldType::Top(true));
    assert_eq!(fields[1], DeepOuterFieldType::M(7));
    assert_eq!(fields[2], DeepOuterFieldType::Z(2.5));
}

#[test]
fn nested_two_nested_fields_into() {
    // TwoNestedFieldType has: Own(u8), Lx(i32), Ly(i32), Rx(i32), Ry(i32)
    let s = TwoNested {
        own: 99,
        left: LeftInner { lx: 1, ly: 2 },
        right: RightInner { rx: 3, ry: 4 },
        skippa: "test".to_string(),
    };
    let fields: [TwoNestedFieldType; 5] = s.into();
    assert_eq!(fields[0], TwoNestedFieldType::Own(99));
    assert_eq!(fields[1], TwoNestedFieldType::Lx(1));
    assert_eq!(fields[2], TwoNestedFieldType::Ly(2));
    assert_eq!(fields[3], TwoNestedFieldType::Rx(3));
    assert_eq!(fields[4], TwoNestedFieldType::Ry(4));
}
