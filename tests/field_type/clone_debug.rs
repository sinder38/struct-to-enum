#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldType;

// Clone semantics for heap types

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct CloneCheck {
    name: String,
    data: Vec<u8>,
}

// Debug formatting

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DebugCheck {
    number: i64,
    text: String,
}

#[test]
fn field_type_clone_heap_types() {
    let v = CloneCheckFieldType::Name("hello".to_string());
    let cloned = v.clone();
    assert_eq!(v, cloned);

    let v2 = CloneCheckFieldType::Data(vec![1, 2, 3]);
    let cloned2 = v2.clone();
    assert_eq!(v2, cloned2);

    // Original and clone are independent
    drop(v2);
    assert_eq!(cloned2, CloneCheckFieldType::Data(vec![1, 2, 3]));
}

#[test]
fn debug_formatting() {
    let ft = DebugCheckFieldType::Number(-42i64);
    assert_eq!(format!("{:?}", ft), "Number(-42)");

    let ft = DebugCheckFieldType::Text("hello".to_string());
    assert_eq!(format!("{:?}", ft), "Text(\"hello\")");
}
