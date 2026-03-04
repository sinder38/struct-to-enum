#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldName;

// --- Generic struct with lifetime ---

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

// --- Three type params + lifetime ---

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

// --- Generic with lifetime only (no type params) ---

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

// ----------------------------------------------------------------

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
    let c_val = "world".to_string();
    let hidden_val = 0i32;
    let s = MultiGeneric {
        alpha: 42i32,
        beta: true,
        gamma: &c_val,
        hidden: &hidden_val,
    };
    let names: [MultiGenericFieldName; 3] = (&s).into();
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
    let val = 42i32;
    let w = Wrapper {
        value: &val,
        tag: "t",
        hidden: 0,
    };
    let names: [WrapperFieldName; 2] = (&w).into();
    assert_eq!(names, [WrapperFieldName::Value, WrapperFieldName::Tag]);
}
