#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldName;

// FieldName enum can receive serde derives

#[derive(FieldName, serde::Serialize, serde::Deserialize)]
#[stem_name_derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct SerdeStruct {
    name: String,
    age: u32,
    #[stem_name(skip)]
    password: String,
}

// serde::Serialize only on the parent struct, not forwarded to enum

#[derive(FieldName, serde::Serialize)]
#[stem_name_derive(Debug, Clone, PartialEq, serde::Serialize)]
struct TestEmpty {
    first: i32,
    second_field: bool,
}

#[test]
fn serde_field_name_serialize() {
    fn assert_serialize<T: serde::Serialize>(_: &T) {}

    let name_variant = SerdeStructFieldName::Name;
    assert_serialize(&name_variant);

    let s = SerdeStruct {
        name: "Alice".to_string(),
        age: 30,
        password: "secret".to_string(),
    };
    assert_serialize(&s);
}

#[test]
fn serde_field_name_skipped_field_absent() {
    // Only Name and Age variants should exist (password is skipped)
    let _name = SerdeStructFieldName::Name;
    let _age = SerdeStructFieldName::Age;

    match SerdeStructFieldName::Name {
        SerdeStructFieldName::Name => (),
        SerdeStructFieldName::Age => (),
    }
}
