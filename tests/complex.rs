#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::{FieldName, FieldType};

#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Test {
    first: i32,
    second_field: Option<String>,
    #[stem_type(skip)]
    #[stem_name = "skip"]
    third: bool,
    #[stem_name = "skip"]
    fourth: bool,
}

impl Test {
    fn foo() -> bool {
        true
    }
}

mod bar {
    use struct_to_enum::{FieldName, FieldType};

    #[derive(FieldType, FieldName)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct TestGen<'a, T: 'a, U>
    where
        U: 'a,
    {
        first: T,
        second_field: Option<&'a U>,
        #[stem_type(skip)]
        #[stem_name = "skip"]
        third: &'a T,
        #[stem_name = "skip"]
        fourth: U,
    }

    impl<'a, T: 'a, U> TestGen<'a, T, U>
    where
        U: 'a,
    {
        pub fn new(first: T, second: &'a U, third: &'a T, fourth: U) -> Self {
            let second_field = Some(second);
            TestGen {
                first,
                second_field,
                third,
                fourth,
            }
        }
    }
}

use bar::{TestGen, TestGenFieldName, TestGenFieldType};

#[test]
fn full_field_types_variants() {
    let _field = TestFieldType::First(2);
    let _field = TestFieldType::Fourth(false);
    let field = TestFieldType::SecondField(None);
    match field {
        TestFieldType::First(_) => (),
        TestFieldType::SecondField(_) => (),
        TestFieldType::Fourth(_) => (),
    }

    let _field = TestFieldName::First;
    let field = TestFieldName::SecondField;
    match field {
        TestFieldName::First => (),
        TestFieldName::SecondField => (),
    }

    let _field = TestGenFieldType::First::<_, bool>(2);
    let _field = TestGenFieldType::Fourth::<i32, _>(false);
    let field = TestGenFieldType::SecondField::<i32, bool>(None);
    match field {
        TestGenFieldType::First(_) => (),
        TestGenFieldType::SecondField(_) => (),
        TestGenFieldType::Fourth(_) => (),
    }

    let _field = TestGenFieldName::First;
    let field = TestGenFieldName::SecondField;
    match field {
        TestGenFieldName::First => (),
        TestGenFieldName::SecondField => (),
    }
}

#[test]
fn derive_field_types() {
    let field = TestFieldType::First(1).clone();
    assert_eq!(TestFieldType::First(1), field);
    assert_ne!(TestFieldType::First(2), field);

    let first_field = TestGenFieldType::First::<_, &str>(2);
    let second_field = TestGenFieldType::SecondField::<i32, &str>(None);
    assert_ne!(first_field, second_field);
    assert_eq!(first_field, first_field.clone());
    assert_eq!("First(2)", format!("{:?}", first_field));

    let field = TestFieldType::SecondField(Some("test".to_string())).clone();
    assert_eq!(TestFieldType::SecondField(Some("test".to_string())), field);
    assert_ne!(TestFieldType::SecondField(Some("".to_string())), field);

    let name = TestFieldName::First;
    assert_eq!(TestFieldName::First, name);
    assert_ne!(TestFieldName::SecondField, name);

    let name = TestFieldName::SecondField;
    assert_eq!(TestFieldName::SecondField, name);
    assert_ne!(TestFieldName::First, name);
}

#[test]
fn into_field_types() {
    let test = Test {
        first: 1,
        second_field: Some("test".to_string()),
        third: true,
        fourth: true,
    };
    let fields: [TestFieldType; 3] = test.into();
    assert!(matches!(fields, [
        TestFieldType::First(1),
        TestFieldType::SecondField(Some(ref s)),
        TestFieldType::Fourth(true),
    ] if s == "test"));

    let test = Test {
        first: 1,
        second_field: Some("test".to_string()),
        third: true,
        fourth: true,
    };
    let names: [TestFieldName; 2] = (&test).into();
    assert!(matches!(
        names,
        [TestFieldName::First, TestFieldName::SecondField]
    ));

    let message = "test".to_string();

    let test = TestGen::new(1, &message, &2, message.clone());
    let fields: [TestGenFieldType<i32, String>; 3] = test.into();
    assert!(matches!(fields, [
                 TestGenFieldType::First(1),
                 TestGenFieldType::SecondField(Some(s)),
                 TestGenFieldType::Fourth(_),
             ] if s == &message));

    let test = TestGen::new(1, &message, &2, message.clone());
    let fields: [TestGenFieldName; 2] = (&test).into();
    assert!(matches!(
        fields,
        [TestGenFieldName::First, TestGenFieldName::SecondField]
    ));
}

#[test]
fn field_name_variants() {
    let name = TestFieldName::First;
    assert_eq!(TestFieldName::First, name);
    assert_ne!(TestFieldName::SecondField, name);

    let name = TestFieldName::SecondField;
    assert_eq!(TestFieldName::SecondField, name);
    assert_ne!(TestFieldName::First, name);
}
