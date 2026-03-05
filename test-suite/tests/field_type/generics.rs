#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldType;

// Generic struct with lifetime

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct TestGen<'a, T: 'a, U>
where
    U: 'a,
{
    first: T,
    second_field: Option<&'a U>,
    #[stem_type(skip)]
    third: &'a T,
    #[stem_type = "skip"]
    fourth: U,
}

// Three type params + lifetime

mod generics_mod {
    use struct_to_enum::FieldType;

    #[derive(FieldType)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct MultiGeneric<'a, A, B, C> {
        pub alpha: A,
        pub beta: B,
        pub gamma: &'a C,
        #[stem_type(skip)]
        pub hidden: &'a A,
    }
}

use generics_mod::{MultiGeneric, MultiGenericFieldType};

#[test]
fn full_generic_field_type_variants() {
    let _field = TestGenFieldType::First::<_, &str>(2);
    let field = TestGenFieldType::SecondField::<i32, bool>(None);
    match field {
        TestGenFieldType::First(_) => (),
        TestGenFieldType::SecondField(_) => (),
    }
}

#[test]
fn derive_generic_field_type() {
    let first_field = TestGenFieldType::First::<_, &str>(2);
    let second_field = TestGenFieldType::SecondField::<i32, &str>(None);
    assert_ne!(first_field, second_field);
    assert_eq!(first_field, first_field.clone());
    assert_eq!("First(2)", format!("{:?}", first_field));
}

#[test]
fn into_generic_field_type() {
    let message = "test".to_string();
    let test = TestGen {
        first: 1i32,
        second_field: Some(&message),
        third: &2i32,
        fourth: message.clone(),
    };
    let fields: [TestGenFieldType<i32, String>; 2] = test.into();
    assert!(matches!(fields, [
        TestGenFieldType::First(1),
        TestGenFieldType::SecondField(Some(s)),
    ] if s == &message));
}

#[test]
fn multi_generic_field_type_from() {
    let c_val = "world".to_string();
    let hidden_val = 0i32;
    let s = MultiGeneric {
        alpha: 42i32,
        beta: true,
        gamma: &c_val,
        hidden: &hidden_val,
    };
    let fields: [MultiGenericFieldType<i32, bool, String>; 3] = s.into();
    assert_eq!(fields[0], MultiGenericFieldType::Alpha(42i32));
    assert_eq!(fields[1], MultiGenericFieldType::Beta(true));
    assert_eq!(fields[2], MultiGenericFieldType::Gamma(&c_val));
}
