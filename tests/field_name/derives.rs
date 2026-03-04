#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldName;

// --- Default derives: Copy, Clone, PartialEq, Eq, Debug are always present ---

#[derive(FieldName)]
struct DefaultDerives {
    alpha: i32,
    beta: String,
}

// --- Custom derives forwarded via stem_name_derive ---

#[derive(FieldName)]
#[stem_name_derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct WithHash {
    key: String,
    value: i32,
}

// --- Short alias prefix (ste_) ---

#[derive(FieldName)]
#[ste_name_derive(Debug, Clone, PartialEq)]
struct AliasDerive {
    foo: i32,
    bar: String,
}

// ----------------------------------------------------------------

#[test]
fn field_name_default_derives() {
    let a = DefaultDerivesFieldName::Alpha;
    let b = a; // Copy
    let c = a.clone(); // Clone
    assert_eq!(a, b); // PartialEq + Eq
    assert_eq!(b, c);
    let _ = format!("{:?}", a); // Debug
}

#[test]
fn field_name_with_hash_derive() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(WithHashFieldName::Key);
    set.insert(WithHashFieldName::Value);
    set.insert(WithHashFieldName::Key); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn field_name_debug_formatting() {
    let fn_v = DefaultDerivesFieldName::Alpha;
    assert_eq!(format!("{:?}", fn_v), "Alpha");

    let fn_v = DefaultDerivesFieldName::Beta;
    assert_eq!(format!("{:?}", fn_v), "Beta");
}

#[test]
fn alias_prefix_derive() {
    let v = AliasDeriveFieldName::Foo;
    let cloned = v.clone();
    assert_eq!(v, cloned);
    let _ = format!("{:?}", v);
}
