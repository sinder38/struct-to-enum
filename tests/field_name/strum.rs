#![allow(dead_code)]

extern crate struct_to_enum;
extern crate strum;
extern crate strum_macros;

use struct_to_enum::FieldName;
use strum_macros::EnumString;

// --- EnumString via stem_name_derive ---

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
#[stem_name_attr(strum(ascii_case_insensitive))]
struct TestDeriveArguments {
    first: i32,
    second_field: bool,
}

// --- EnumString + ascii_case_insensitive on a larger struct ---

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
#[stem_name_attr(strum(ascii_case_insensitive))]
struct NameAttrTest {
    my_field: i32,
    another_field: bool,
}

// --- Combined with FieldType strum on same struct (FieldName side only here) ---

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
#[stem_name_attr(strum(ascii_case_insensitive))]
struct StrumCombinedName {
    user_id: u64,
    display_name: String,
    #[stem_name(skip)]
    internal_token: String,
    is_active: bool,
}

// ----------------------------------------------------------------

#[test]
fn field_name_fromstr_case_insensitive() {
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

#[test]
fn stem_name_attr_applied() {
    let v = "myfield".parse::<NameAttrTestFieldName>().unwrap();
    assert_eq!(v, NameAttrTestFieldName::MyField);

    let v = "ANOTHERFIELD".parse::<NameAttrTestFieldName>().unwrap();
    assert_eq!(v, NameAttrTestFieldName::AnotherField);
}

#[test]
fn strum_combined_field_name_parse() {
    assert_eq!(
        "userid".parse::<StrumCombinedNameFieldName>().unwrap(),
        StrumCombinedNameFieldName::UserId
    );
    assert_eq!(
        "DISPLAYNAME".parse::<StrumCombinedNameFieldName>().unwrap(),
        StrumCombinedNameFieldName::DisplayName
    );
    assert_eq!(
        "isactive".parse::<StrumCombinedNameFieldName>().unwrap(),
        StrumCombinedNameFieldName::IsActive
    );
}
