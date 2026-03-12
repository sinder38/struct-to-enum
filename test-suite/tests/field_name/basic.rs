#![allow(dead_code)]

use struct_to_enum::{FieldName, FieldNames};

extern crate struct_to_enum;

// Simple struct: skip both syntaxes

#[derive(FieldName)]
struct Test {
    first: i32,
    second_field: Option<String>,
    #[stem_name(skip)]
    third: bool,
    #[stem_name = "skip"]
    fourth: bool,
}

// Additional derives forwarded

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

// Single-field struct

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct SingleField {
    only: i32,
}

// All-but-one skipped

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct AlmostAllSkipped {
    #[stem_name(skip)]
    a: i32,
    #[stem_name(skip)]
    b: i32,
    #[stem_name(skip)]
    c: i32,
    survivor: String,
}

// Both skip syntaxes together

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct SkipSyntaxTest {
    keep_a: i32,
    #[stem_name(skip)]
    skip_paren: i32,
    keep_b: i32,
    #[stem_name = "skip"]
    skip_eq: i32,
    keep_c: i32,
}

// Large struct: PascalCase conversion and field order

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
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

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct ExhaustStruct {
    aaa: u8,
    bbb: u16,
    ccc: u32,
    #[stem_name(skip)]
    ddd: u64,
}

// Empty Struct

#[derive(FieldName)]
struct EmptyStruct {}

fn match_field_name(v: ExhaustStructFieldName) -> &'static str {
    match v {
        ExhaustStructFieldName::Aaa => "aaa",
        ExhaustStructFieldName::Bbb => "bbb",
        ExhaustStructFieldName::Ccc => "ccc",
    }
}

// Struct is not consumed when converting to FieldName

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct Reusable {
    a: i32,
    b: i32,
}

#[test]
fn full_field_name_variants() {
    let _field = TestFieldName::First;
    let field = TestFieldName::SecondField;
    match field {
        TestFieldName::First => (),
        TestFieldName::SecondField => (),
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
    let name = TestTypesDeriveFieldName::First;
    assert_eq!(TestTypesDeriveFieldName::First, name);
    assert_ne!(TestTypesDeriveFieldName::Second, name);

    let name = TestTypesDeriveFieldName::Second;
    assert_eq!(TestTypesDeriveFieldName::Second, name);
    assert_ne!(TestTypesDeriveFieldName::First, name);
}

#[test]
fn field_name_into_array() {
    // third and fourth are both skipped so only 2 variants remain
    let names = <Test as FieldNames<2>>::field_names();
    assert!(matches!(
        names,
        [TestFieldName::First, TestFieldName::SecondField]
    ));
}

#[test]
fn skip_both_syntaxes() {
    let names = <SkipSyntaxTest as FieldNames<3>>::field_names();
    assert_eq!(
        names,
        [
            SkipSyntaxTestFieldName::KeepA,
            SkipSyntaxTestFieldName::KeepB,
            SkipSyntaxTestFieldName::KeepC,
        ]
    );
}

#[test]
fn single_field_struct() {
    let names = <SingleField as FieldNames<1>>::field_names();
    assert_eq!(names[0], SingleFieldFieldName::Only);
}

#[test]
fn almost_all_skipped() {
    let names = <AlmostAllSkipped as FieldNames<1>>::field_names();
    assert_eq!(names[0], AlmostAllSkippedFieldName::Survivor);
}

#[test]
fn large_struct_field_order_preserved() {
    let names = <BigStruct as FieldNames<10>>::field_names();
    assert_eq!(names[0], BigStructFieldName::FieldOne);
    assert_eq!(names[9], BigStructFieldName::FieldTen);
}

#[test]
fn exhaustive_match_compiles_and_runs() {
    assert_eq!(match_field_name(ExhaustStructFieldName::Aaa), "aaa");
    assert_eq!(match_field_name(ExhaustStructFieldName::Bbb), "bbb");
    assert_eq!(match_field_name(ExhaustStructFieldName::Ccc), "ccc");
}

#[test]
fn field_name_borrows_struct() {
    let s = Reusable { a: 1, b: 2 };
    let names1 = <Reusable as FieldNames<2>>::field_names();
    let names2 = <Reusable as FieldNames<2>>::field_names();
    assert_eq!(names1, names2);
    assert_eq!(s.a, 1);
}

#[test]
fn field_names_trait_method() {
    let s = Reusable { a: 1, b: 2 };
    let names = <Reusable as FieldNames<2>>::field_names();
    assert_eq!(names, [ReusableFieldName::A, ReusableFieldName::B]);
    // Struct is still accessible after the call
    assert_eq!(s.a, 1);
}

#[test]
fn field_names_empty() {
    let names = <EmptyStruct as FieldNames<0>>::field_names();
    assert_eq!(names.len(), 0);
}
