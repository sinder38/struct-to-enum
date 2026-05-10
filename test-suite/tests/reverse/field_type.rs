#![allow(dead_code)]

use struct_to_enum::VerifyFieldType;

extern crate struct_to_enum;

// Basic: hand-written enum that matches a struct

#[derive(Debug, PartialEq)]
enum ConfigFieldType {
    Width(u32),
    Height(u32),
}

#[derive(VerifyFieldType)]
#[stem_reverse_type(ConfigFieldType)]
struct Config {
    width: u32,
    height: u32,
    #[stem_type(skip)]
    name: String,
}

// Single field

#[derive(Debug, PartialEq)]
enum SingleFieldType {
    Only(i32),
}

#[derive(VerifyFieldType)]
#[stem_reverse_type(SingleFieldType)]
struct SingleField {
    only: i32,
}

// All but one skipped

#[derive(Debug, PartialEq)]
enum SurvivorFieldType {
    Survivor(String),
}

#[derive(VerifyFieldType)]
#[stem_reverse_type(SurvivorFieldType)]
struct AlmostAllSkipped {
    #[stem_type(skip)]
    a: i32,
    #[stem_type(skip)]
    b: i32,
    survivor: String,
}

// Mixed types

#[derive(Debug, PartialEq)]
enum MixedFieldType {
    Name(String),
    Age(u32),
    Active(bool),
}

#[derive(VerifyFieldType)]
#[stem_reverse_type(MixedFieldType)]
struct MixedStruct {
    name: String,
    age: u32,
    active: bool,
}

// Short alias attribute

#[derive(Debug, PartialEq)]
enum AliasFieldType {
    Foo(i32),
    Bar(String),
}

#[derive(VerifyFieldType)]
#[ste_reverse_type(AliasFieldType)]
struct AliasStruct {
    foo: i32,
    bar: String,
}
