#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::{FieldName, FieldNames};

// Generic struct with lifetime

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

// Three type params + lifetime

mod generics_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    #[stem_name_derive(Debug, Clone, PartialEq)]
    pub struct MultiGeneric<'a, A, B, C> {
        pub alpha: A,
        pub beta: B,
        pub gamma: &'a C,
        #[stem_name(skip)]
        pub hidden: &'a A,
    }
}

use generics_mod::{MultiGeneric, MultiGenericFieldName};

// Generic with lifetime only (no type params)

mod generic_name_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    #[stem_name_derive(Debug, Clone, PartialEq)]
    pub struct Wrapper<'a, T: 'a> {
        pub value: &'a T,
        pub tag: &'static str,
        #[stem_name(skip)]
        pub hidden: u32,
    }
}

use generic_name_mod::{Wrapper, WrapperFieldName};

#[test]
fn full_generic_field_name_variants() {
    let _field = TestGenFieldName::First;
    let field = TestGenFieldName::SecondField;
    match field {
        TestGenFieldName::First => (),
        TestGenFieldName::SecondField => (),
    }
}

#[test]
fn generic_field_name_equality() {
    let name = TestGenFieldName::First;
    assert_eq!(TestGenFieldName::First, name);
    assert_ne!(TestGenFieldName::SecondField, name);
}

#[test]
fn multi_generic_field_name_from() {
    let names = <MultiGeneric<'_, i32, bool, String> as FieldNames<3>>::field_names();
    assert_eq!(
        names,
        [
            MultiGenericFieldName::Alpha,
            MultiGenericFieldName::Beta,
            MultiGenericFieldName::Gamma,
        ]
    );
}

#[test]
fn generic_field_name_from() {
    let names = <Wrapper<'_, i32> as FieldNames<2>>::field_names();
    assert_eq!(names, [WrapperFieldName::Value, WrapperFieldName::Tag]);
}
