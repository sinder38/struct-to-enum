#![no_std]
#![allow(dead_code)]
extern crate struct_to_enum;

use struct_to_enum::{FieldName, FieldNames, FieldType};

// --- FieldName ---

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct WithSkip {
    keep: u32,
    #[stem_name(skip)]
    skip_me: u32,
}

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct Single {
    only: u8,
}

// --- FieldType ---

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct RgbWithSkip {
    r: u8,
    #[stem_type(skip)]
    internal: u8,
    b: u8,
}

// --- FieldName tests ---

#[test]
fn no_std_field_names_basic() {
    let names: [PointFieldName; 2] = Point::field_names();
    assert_eq!(names, [PointFieldName::X, PointFieldName::Y]);
}

#[test]
fn no_std_field_names_skip() {
    let names: [WithSkipFieldName; 1] = WithSkip::field_names();
    assert_eq!(names, [WithSkipFieldName::Keep]);
}

#[test]
fn no_std_field_names_single() {
    let names: [SingleFieldName; 1] = Single::field_names();
    assert_eq!(names, [SingleFieldName::Only]);
}

#[test]
fn no_std_field_names_trait_method() {
    let names = <Point as FieldNames<2>>::field_names();
    assert_eq!(names[0], PointFieldName::X);
    assert_eq!(names[1], PointFieldName::Y);
}

#[test]
fn no_std_field_names_exhaustive_match() {
    fn describe(v: PointFieldName) -> &'static str {
        match v {
            PointFieldName::X => "x",
            PointFieldName::Y => "y",
        }
    }
    assert_eq!(describe(PointFieldName::X), "x");
    assert_eq!(describe(PointFieldName::Y), "y");
}

// --- FieldType tests ---

#[test]
fn no_std_field_type_basic() {
    let rgb = Rgb { r: 1, g: 2, b: 3 };
    let fields: [RgbFieldType; 3] = rgb.into();
    assert!(matches!(
        fields,
        [RgbFieldType::R(1), RgbFieldType::G(2), RgbFieldType::B(3)]
    ));
}

#[test]
fn no_std_field_type_skip() {
    let rgb = RgbWithSkip {
        r: 10,
        internal: 99,
        b: 20,
    };
    let fields: [RgbWithSkipFieldType; 2] = rgb.into();
    assert!(matches!(
        fields,
        [RgbWithSkipFieldType::R(10), RgbWithSkipFieldType::B(20)]
    ));
}

#[test]
fn no_std_field_type_clone_eq() {
    let rgb = Rgb { r: 5, g: 6, b: 7 };
    let fields: [RgbFieldType; 3] = rgb.into();
    let cloned = fields.clone();
    assert_eq!(fields, cloned);
}
