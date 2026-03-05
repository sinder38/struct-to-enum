#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldType;

// Simple struct: skip both syntaxes

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

// Additional derives forwarded

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct TestTypesDerive {
    first: i32,
    second: bool,
}

// Single-field struct

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct SingleField {
    only: i32,
}

// All-but-one skipped

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct AlmostAllSkipped {
    #[stem_type(skip)]
    a: i32,
    #[stem_type(skip)]
    b: i32,
    #[stem_type(skip)]
    c: i32,
    survivor: String,
}

// Both skip syntaxes together

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct SkipSyntaxTest {
    keep_a: i32,
    #[stem_type(skip)]
    skip_paren: i32,
    keep_b: i32,
    #[stem_type = "skip"]
    skip_eq: i32,
    keep_c: i32,
}

// Large struct: PascalCase conversion and field order

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct BigStruct {
    field_one: u8,
    field_two: u16,
    field_three: u32,
    field_four: u64,
    field_five: i8,
    field_six: i16,
    field_seven: i32,
    field_eight: i64,
    field_nine: f32,
    field_ten: f64,
}

// Exhaustiveness

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct ExhaustStruct {
    aaa: u8,
    bbb: u16,
    ccc: u32,
    #[stem_type(skip)]
    ddd: u64,
}

fn match_field_type(v: ExhaustStructFieldType) -> &'static str {
    match v {
        ExhaustStructFieldType::Aaa(_) => "aaa",
        ExhaustStructFieldType::Bbb(_) => "bbb",
        ExhaustStructFieldType::Ccc(_) => "ccc",
    }
}

// Static str field type

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Pair {
    first: i32,
    second: &'static str,
}

#[test]
fn full_field_type_variants() {
    let _field = TestFieldType::First(2);
    let field = TestFieldType::SecondField(None);
    match field {
        TestFieldType::First(_) => (),
        TestFieldType::SecondField(_) => (),
    }

    let first_field = TestTypesDeriveFieldType::First(2);
    match first_field {
        TestTypesDeriveFieldType::First(_) => (),
        TestTypesDeriveFieldType::Second(_) => (),
    }
}

#[test]
fn derive_field_type() {
    let field = TestTypesDeriveFieldType::First(1).clone();
    assert_eq!(TestTypesDeriveFieldType::First(1), field);
    assert_ne!(TestTypesDeriveFieldType::First(2), field);
    assert_eq!("First(1)", format!("{:?}", field));

    let field = TestTypesDeriveFieldType::Second(true).clone();
    assert_eq!(TestTypesDeriveFieldType::Second(true), field);
    assert_ne!(TestTypesDeriveFieldType::Second(false), field);
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
}

#[test]
fn skip_both_syntaxes() {
    let s = SkipSyntaxTest {
        keep_a: 1,
        skip_paren: 2,
        keep_b: 3,
        skip_eq: 4,
        keep_c: 5,
    };
    let fields: [SkipSyntaxTestFieldType; 3] = s.into();
    assert_eq!(fields[0], SkipSyntaxTestFieldType::KeepA(1));
    assert_eq!(fields[1], SkipSyntaxTestFieldType::KeepB(3));
    assert_eq!(fields[2], SkipSyntaxTestFieldType::KeepC(5));
}

#[test]
fn single_field_struct() {
    let s = SingleField { only: 7 };
    let fields: [SingleFieldFieldType; 1] = s.into();
    assert_eq!(fields[0], SingleFieldFieldType::Only(7));
}

#[test]
fn almost_all_skipped() {
    let s = AlmostAllSkipped {
        a: 1,
        b: 2,
        c: 3,
        survivor: "alive".to_string(),
    };
    let fields: [AlmostAllSkippedFieldType; 1] = s.into();
    assert_eq!(
        fields[0],
        AlmostAllSkippedFieldType::Survivor("alive".to_string())
    );
}

#[test]
fn large_struct_field_order_preserved() {
    let s = BigStruct {
        field_one: 1,
        field_two: 2,
        field_three: 3,
        field_four: 4,
        field_five: 5,
        field_six: 6,
        field_seven: 7,
        field_eight: 8,
        field_nine: 9.0,
        field_ten: 10.0,
    };
    let fields: [BigStructFieldType; 10] = s.into();
    assert_eq!(fields[0], BigStructFieldType::FieldOne(1));
    assert_eq!(fields[1], BigStructFieldType::FieldTwo(2));
    assert_eq!(fields[9], BigStructFieldType::FieldTen(10.0));
}

#[test]
fn exhaustive_match_compiles_and_runs() {
    assert_eq!(match_field_type(ExhaustStructFieldType::Aaa(1)), "aaa");
    assert_eq!(match_field_type(ExhaustStructFieldType::Bbb(2)), "bbb");
    assert_eq!(match_field_type(ExhaustStructFieldType::Ccc(3)), "ccc");
}

#[test]
fn field_type_from_and_into_both_work() {
    let p1 = Pair {
        first: 10,
        second: "hi",
    };
    let by_into: [PairFieldType; 2] = p1.into();

    let p2 = Pair {
        first: 10,
        second: "hi",
    };
    let by_from: [PairFieldType; 2] = <[PairFieldType; 2]>::from(p2);

    assert_eq!(by_into[0], by_from[0]);
    assert_eq!(by_into[1], by_from[1]);
    assert_eq!(by_into[0], PairFieldType::First(10));
    assert_eq!(by_into[1], PairFieldType::Second("hi"));
}
