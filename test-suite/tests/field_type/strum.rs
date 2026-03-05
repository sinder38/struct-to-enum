#![allow(dead_code)]

extern crate struct_to_enum;
extern crate strum;
extern crate strum_macros;

use struct_to_enum::FieldType;
use strum::VariantNames;

// VariantNames via stem_type_derive + serialize_all via stem_type_attr

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq, strum_macros::VariantNames)]
#[stem_type_attr(strum(serialize_all = "SCREAMING-KEBAB-CASE"))]
struct TestDeriveArguments {
    first: i32,
    second_field: bool,
}

// snake_case serialize_all

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq, strum_macros::VariantNames)]
#[stem_type_attr(strum(serialize_all = "snake_case"))]
struct AttrTest {
    my_field: i32,
    another_field: bool,
}

// Combined with FieldName strum on same struct (FieldType side only here)

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq, strum_macros::VariantNames)]
#[stem_type_attr(strum(serialize_all = "SCREAMING-KEBAB-CASE"))]
struct StrumCombinedType {
    user_id: u64,
    display_name: String,
    #[stem_type(skip)]
    internal_token: String,
    is_active: bool,
}

// Short alias prefix (ste_)

#[derive(FieldType)]
#[ste_type_derive(Debug, Clone, PartialEq)]
struct AliasTest {
    value: u64,
    #[ste_type(skip)]
    secret: String,
    label: &'static str,
}

#[test]
fn derive_macro_arguments_screaming_kebab() {
    assert_eq!(
        TestDeriveArgumentsFieldType::VARIANTS,
        ["FIRST", "SECOND-FIELD"]
    );
}

#[test]
fn stem_type_attr_snake_case() {
    assert_eq!(AttrTestFieldType::VARIANTS, ["my_field", "another_field"]);
}

#[test]
fn strum_combined_variant_names() {
    assert_eq!(
        StrumCombinedTypeFieldType::VARIANTS,
        ["USER-ID", "DISPLAY-NAME", "IS-ACTIVE"]
    );
}

#[test]
fn short_alias_skip_and_variants() {
    let v = AliasTestFieldType::Value(42u64);
    let l = AliasTestFieldType::Label("hi");
    match v {
        AliasTestFieldType::Value(_) => (),
        AliasTestFieldType::Label(_) => (),
    }
    match l {
        AliasTestFieldType::Value(_) => (),
        AliasTestFieldType::Label(_) => (),
    }

    let s = AliasTest {
        value: 99,
        secret: "hidden".to_string(),
        label: "shown",
    };
    let fields: [AliasTestFieldType; 2] = s.into();
    assert_eq!(fields[0], AliasTestFieldType::Value(99));
    assert_eq!(fields[1], AliasTestFieldType::Label("shown"));
}
