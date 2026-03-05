#![allow(dead_code)]

extern crate struct_to_enum;

use std::collections::HashMap;
use struct_to_enum::FieldType;

// Rich field types: Vec, HashMap, Box, Result, tuples, arrays, nested Option

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct RichTypes {
    count: usize,
    tags: Vec<String>,
    map: HashMap<String, i64>,
    pair: (u8, u8),
    fixed: [f32; 4],
    boxed: Box<i32>,
    maybe: Option<Vec<u8>>,
    result_field: Result<i32, String>,
}

// Nested Option

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct OptionalFields {
    required: i32,
    optional_str: Option<String>,
    optional_num: Option<u64>,
    nested_opt: Option<Option<bool>>,
}

// FieldName counterpart for rich types (validates PascalCase conversion)

use struct_to_enum::FieldName;

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq)]
struct RichTypesNames {
    count: usize,
    tags: Vec<String>,
    map: HashMap<String, i64>,
    pair: (u8, u8),
    fixed: [f32; 4],
    boxed: Box<i32>,
    maybe: Option<Vec<u8>>,
    result_field: Result<i32, String>,
}

#[test]
fn rich_field_types_variants() {
    let v = RichTypesFieldType::Count(0);
    assert!(matches!(v, RichTypesFieldType::Count(0)));

    let v = RichTypesFieldType::Tags(vec!["a".to_string()]);
    assert!(matches!(v, RichTypesFieldType::Tags(_)));

    let v = RichTypesFieldType::Pair((1, 2));
    assert!(matches!(v, RichTypesFieldType::Pair((1, 2))));

    let v = RichTypesFieldType::Fixed([1.0, 2.0, 3.0, 4.0]);
    assert!(matches!(v, RichTypesFieldType::Fixed(_)));

    let v = RichTypesFieldType::Maybe(Some(vec![1, 2]));
    assert!(matches!(v, RichTypesFieldType::Maybe(Some(_))));

    let v = RichTypesFieldType::ResultField(Ok(42));
    assert!(matches!(v, RichTypesFieldType::ResultField(Ok(42))));
}

#[test]
fn rich_field_types_field_name_array() {
    let dummy = RichTypesNames {
        count: 0,
        tags: vec![],
        map: HashMap::new(),
        pair: (0, 0),
        fixed: [0.0; 4],
        boxed: Box::new(0),
        maybe: None,
        result_field: Ok(0),
    };
    let names: [RichTypesNamesFieldName; 8] = (&dummy).into();
    assert_eq!(names[0], RichTypesNamesFieldName::Count);
    assert_eq!(names[1], RichTypesNamesFieldName::Tags);
    assert_eq!(names[2], RichTypesNamesFieldName::Map);
    assert_eq!(names[3], RichTypesNamesFieldName::Pair);
    assert_eq!(names[4], RichTypesNamesFieldName::Fixed);
    assert_eq!(names[5], RichTypesNamesFieldName::Boxed);
    assert_eq!(names[6], RichTypesNamesFieldName::Maybe);
    assert_eq!(names[7], RichTypesNamesFieldName::ResultField);
}

#[test]
fn rich_field_types_into() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), 42i64);

    let s = RichTypes {
        count: 7,
        tags: vec!["x".to_string(), "y".to_string()],
        map: map.clone(),
        pair: (3, 4),
        fixed: [0.1, 0.2, 0.3, 0.4],
        boxed: Box::new(99),
        maybe: Some(vec![1, 2, 3]),
        result_field: Err("oops".to_string()),
    };

    let fields: [RichTypesFieldType; 8] = s.into();
    assert_eq!(fields[0], RichTypesFieldType::Count(7));
    assert_eq!(
        fields[1],
        RichTypesFieldType::Tags(vec!["x".to_string(), "y".to_string()])
    );
    assert_eq!(fields[2], RichTypesFieldType::Map(map));
    assert_eq!(fields[3], RichTypesFieldType::Pair((3, 4)));
    assert_eq!(fields[4], RichTypesFieldType::Fixed([0.1, 0.2, 0.3, 0.4]));
    assert_eq!(fields[5], RichTypesFieldType::Boxed(Box::new(99)));
    assert_eq!(fields[6], RichTypesFieldType::Maybe(Some(vec![1, 2, 3])));
    assert_eq!(
        fields[7],
        RichTypesFieldType::ResultField(Err("oops".to_string()))
    );
}

#[test]
fn optional_fields_round_trip() {
    let s = OptionalFields {
        required: 1,
        optional_str: Some("hello".to_string()),
        optional_num: None,
        nested_opt: Some(Some(true)),
    };
    let fields: [OptionalFieldsFieldType; 4] = s.into();
    assert_eq!(fields[0], OptionalFieldsFieldType::Required(1));
    assert_eq!(
        fields[1],
        OptionalFieldsFieldType::OptionalStr(Some("hello".to_string()))
    );
    assert_eq!(fields[2], OptionalFieldsFieldType::OptionalNum(None));
    assert_eq!(
        fields[3],
        OptionalFieldsFieldType::NestedOpt(Some(Some(true)))
    );
}
