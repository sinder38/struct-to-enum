#![allow(dead_code)]

use struct_to_enum::VerifyFieldName;

extern crate struct_to_enum;

// Basic: hand-written enum that matches a struct

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum UserFieldName {
    Id,
    UserName,
}

#[derive(VerifyFieldName)]
#[stem_reverse_name(UserFieldName)]
struct User {
    id: u64,
    user_name: String,
    #[stem_name(skip)]
    internal_token: String,
}

// Single field

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SingleFieldName {
    Only,
}

#[derive(VerifyFieldName)]
#[stem_reverse_name(SingleFieldName)]
struct SingleField {
    only: i32,
}

// All but one skipped

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SurvivorFieldName {
    Survivor,
}

#[derive(VerifyFieldName)]
#[stem_reverse_name(SurvivorFieldName)]
struct AlmostAllSkipped {
    #[stem_name(skip)]
    a: i32,
    #[stem_name(skip)]
    b: i32,
    survivor: String,
}

// Multiple fields with PascalCase conversion

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BigFieldName {
    FieldOne,
    FieldTwo,
    FieldThree,
}

#[derive(VerifyFieldName)]
#[stem_reverse_name(BigFieldName)]
struct BigStruct {
    field_one: u8,
    field_two: u16,
    field_three: u32,
}

// Short alias attribute

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AliasFieldName {
    Foo,
    Bar,
}

#[derive(VerifyFieldName)]
#[ste_reverse_name(AliasFieldName)]
struct AliasStruct {
    foo: i32,
    bar: String,
}
