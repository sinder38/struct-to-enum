#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::{FieldName, FieldNames, FieldType};
use strum::VariantNames;
use strum_macros::EnumString;

// Basic combined FieldType + FieldName

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

    let names = <Test as FieldNames<2>>::field_names();
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

    let names2 = <TestGen<'_, i32, String> as FieldNames<2>>::field_names();
    assert!(matches!(
        names2,
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

// Short alias attributes (ste_ prefix)
#[derive(FieldType, FieldName)]
#[ste_type_derive(Debug, Clone, PartialEq)]
struct AliasTest {
    value: u64,
    #[ste_type(skip)]
    #[ste_name(skip)]
    secret: String,
    label: &'static str,
}

#[test]
fn short_alias_attributes() {
    let v = AliasTestFieldType::Value(42u64);
    let l = AliasTestFieldType::Label("hi");
    match v {
        AliasTestFieldType::Value(_) => (),
        AliasTestFieldType::Label(_) => (),
    }
    match l {
        AliasTestFieldType::Value(_) => (),
        AliasTestFieldType::Label(_) => (),
    }

    let name = AliasTestFieldName::Value;
    match name {
        AliasTestFieldName::Value => (),
        AliasTestFieldName::Label => (),
    }

    let s = AliasTest {
        value: 99,
        secret: "hidden".to_string(),
        label: "shown",
    };
    let fields: [AliasTestFieldType; 2] = s.into();
    assert_eq!(fields[0], AliasTestFieldType::Value(99));
    assert_eq!(fields[1], AliasTestFieldType::Label("shown"));
}

// Both skip syntaxes (parenthesized vs name=value)
#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct SkipSyntaxTest {
    keep_a: i32,
    #[stem_type(skip)]
    #[stem_name(skip)]
    skip_paren: i32,
    keep_b: i32,
    #[stem_type = "skip"]
    #[stem_name = "skip"]
    skip_eq: i32,
    keep_c: i32,
}

#[test]
fn both_skip_syntaxes() {
    let s = SkipSyntaxTest {
        keep_a: 1,
        skip_paren: 2,
        keep_b: 3,
        skip_eq: 4,
        keep_c: 5,
    };
    let fields: [SkipSyntaxTestFieldType; 3] = s.into();
    assert_eq!(fields[0], SkipSyntaxTestFieldType::KeepA(1));
    assert_eq!(fields[1], SkipSyntaxTestFieldType::KeepB(3));
    assert_eq!(fields[2], SkipSyntaxTestFieldType::KeepC(5));

    let names = <SkipSyntaxTest as FieldNames<3>>::field_names();
    assert_eq!(
        names,
        [
            SkipSyntaxTestFieldName::KeepA,
            SkipSyntaxTestFieldName::KeepB,
            SkipSyntaxTestFieldName::KeepC,
        ]
    );
}

// Rich field types (Vec, HashMap, Box, Result, tuples, arrays)
use std::collections::HashMap;

#[derive(FieldType, FieldName)]
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

    // All FieldName variants present
    let names = <RichTypes as FieldNames<8>>::field_names();
    assert_eq!(names[0], RichTypesFieldName::Count);
    assert_eq!(names[1], RichTypesFieldName::Tags);
    assert_eq!(names[2], RichTypesFieldName::Map);
    assert_eq!(names[3], RichTypesFieldName::Pair);
    assert_eq!(names[4], RichTypesFieldName::Fixed);
    assert_eq!(names[5], RichTypesFieldName::Boxed);
    assert_eq!(names[6], RichTypesFieldName::Maybe);
    assert_eq!(names[7], RichTypesFieldName::ResultField);
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

// Generic structs with complex bounds + where clauses

mod generics_mod {
    use struct_to_enum::{FieldName, FieldType};

    #[derive(FieldType, FieldName)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct MultiGeneric<'a, A, B, C> {
        pub alpha: A,
        pub beta: B,
        pub gamma: &'a C,
        #[stem_type(skip)]
        #[stem_name(skip)]
        pub hidden: &'a A,
    }
}

use generics_mod::{MultiGeneric, MultiGenericFieldName, MultiGenericFieldType};

#[test]
fn complex_generics_field_type() {
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

#[test]
fn complex_generics_field_name() {
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

// Single-field structs

#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct SingleField {
    only: i32,
}

#[test]
fn single_field_struct() {
    let s = SingleField { only: 7 };
    let fields: [SingleFieldFieldType; 1] = s.into();
    assert_eq!(fields[0], SingleFieldFieldType::Only(7));

    let names = <SingleField as FieldNames<1>>::field_names();
    assert_eq!(names[0], SingleFieldFieldName::Only);
}

// All skipped but one
#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct AlmostAllSkipped {
    #[stem_type(skip)]
    #[stem_name(skip)]
    a: i32,
    #[stem_type(skip)]
    #[stem_name(skip)]
    b: i32,
    #[stem_type(skip)]
    #[stem_name(skip)]
    c: i32,
    survivor: String,
}

#[test]
fn almost_all_skipped() {
    let s = AlmostAllSkipped {
        a: 1,
        b: 2,
        c: 3,
        survivor: "alive".to_string(),
    };
    let fields: [AlmostAllSkippedFieldType; 1] = s.into();
    assert_eq!(
        fields[0],
        AlmostAllSkippedFieldType::Survivor("alive".to_string())
    );

    let names = <AlmostAllSkipped as FieldNames<1>>::field_names();
    assert_eq!(names[0], AlmostAllSkippedFieldName::Survivor);
}

// Large struct

#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct BigStruct {
    field_one: u8,
    field_two: u16,
    field_three: u32,
    field_four: u64,
    field_five: i8,
    field_six: i16,
    field_seven: i32,
    field_eight: i64,
    field_nine: f32,
    field_ten: f64,
}

#[test]
fn large_struct_field_order_preserved() {
    let s = BigStruct {
        field_one: 1,
        field_two: 2,
        field_three: 3,
        field_four: 4,
        field_five: 5,
        field_six: 6,
        field_seven: 7,
        field_eight: 8,
        field_nine: 9.0,
        field_ten: 10.0,
    };

    let fields: [BigStructFieldType; 10] = s.into();
    assert_eq!(fields[0], BigStructFieldType::FieldOne(1));
    assert_eq!(fields[1], BigStructFieldType::FieldTwo(2));
    assert_eq!(fields[2], BigStructFieldType::FieldThree(3));
    assert_eq!(fields[3], BigStructFieldType::FieldFour(4));
    assert_eq!(fields[4], BigStructFieldType::FieldFive(5));
    assert_eq!(fields[5], BigStructFieldType::FieldSix(6));
    assert_eq!(fields[6], BigStructFieldType::FieldSeven(7));
    assert_eq!(fields[7], BigStructFieldType::FieldEight(8));
    assert_eq!(fields[8], BigStructFieldType::FieldNine(9.0));
    assert_eq!(fields[9], BigStructFieldType::FieldTen(10.0));

    let names = <BigStruct as FieldNames<10>>::field_names();
    assert_eq!(names[0], BigStructFieldName::FieldOne);
    assert_eq!(names[9], BigStructFieldName::FieldTen);
}

// Strum integration
#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq, strum_macros::VariantNames)]
#[stem_type_attr(strum(serialize_all = "SCREAMING-KEBAB-CASE"))]
#[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
#[stem_name_attr(strum(ascii_case_insensitive))]
struct StrumCombined {
    user_id: u64,
    display_name: String,
    #[stem_type(skip)]
    #[stem_name(skip)]
    internal_token: String,
    is_active: bool,
}

#[test]
fn strum_combined_variant_names() {
    assert_eq!(
        StrumCombinedFieldType::VARIANTS,
        ["USER-ID", "DISPLAY-NAME", "IS-ACTIVE"]
    );
}

#[test]
fn strum_combined_field_name_parse() {
    assert_eq!(
        "userid".parse::<StrumCombinedFieldName>().unwrap(),
        StrumCombinedFieldName::UserId
    );
    assert_eq!(
        "DISPLAYNAME".parse::<StrumCombinedFieldName>().unwrap(),
        StrumCombinedFieldName::DisplayName
    );
    assert_eq!(
        "isactive".parse::<StrumCombinedFieldName>().unwrap(),
        StrumCombinedFieldName::IsActive
    );
}

#[test]
fn strum_combined_into() {
    let s = StrumCombined {
        user_id: 1,
        display_name: "Alice".to_string(),
        internal_token: "secret".to_string(),
        is_active: true,
    };
    let fields: [StrumCombinedFieldType; 3] = s.into();
    assert_eq!(fields[0], StrumCombinedFieldType::UserId(1));
    assert_eq!(
        fields[1],
        StrumCombinedFieldType::DisplayName("Alice".to_string())
    );
    assert_eq!(fields[2], StrumCombinedFieldType::IsActive(true));
}

// Combined FieldType + FieldName with nesting
#[cfg(all(feature = "nested-type", feature = "nested-name"))]
mod combined_nested_mod {
    use struct_to_enum::{FieldName, FieldType};

    #[derive(FieldType, FieldName)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct Meta {
        pub version: u32,
        pub author: String,
    }

    #[derive(FieldType, FieldName)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct Document {
        pub title: String,
        #[stem_type(nested)]
        #[stem_name(nested)]
        pub meta: Meta,
        pub content: String,
    }
}

#[cfg(all(feature = "nested-type", feature = "nested-name"))]
use combined_nested_mod::{Document, DocumentFieldName, DocumentFieldType, Meta};

#[cfg(all(feature = "nested-type", feature = "nested-name"))]
#[test]
fn combined_nested_field_type_and_name() {
    let doc = Document {
        title: "Hello".to_string(),
        meta: Meta {
            version: 2,
            author: "Alice".to_string(),
        },
        content: "World".to_string(),
    };

    let fields: [DocumentFieldType; 4] = doc.into();
    // Declaration order: title, meta->(version, author), content
    assert_eq!(fields[0], DocumentFieldType::Title("Hello".to_string()));
    assert_eq!(fields[1], DocumentFieldType::Version(2));
    assert_eq!(fields[2], DocumentFieldType::Author("Alice".to_string()));
    assert_eq!(fields[3], DocumentFieldType::Content("World".to_string()));

    let names = <Document as FieldNames<4>>::field_names();
    assert_eq!(
        names,
        [
            DocumentFieldName::Title,
            DocumentFieldName::Version,
            DocumentFieldName::Author,
            DocumentFieldName::Content,
        ]
    );
}

// Section 22: FieldName
#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Reusable {
    a: i32,
    b: i32,
}

#[test]
fn field_name_borrows_struct() {
    let names1 = <Reusable as FieldNames<2>>::field_names();
    let names2 = <Reusable as FieldNames<2>>::field_names();
    assert_eq!(names1, names2);
}

// Match with all variants compiles correctly
#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct ExhaustStruct {
    aaa: u8,
    bbb: u16,
    ccc: u32,
    #[stem_type(skip)]
    #[stem_name(skip)]
    ddd: u64,
}

fn match_field_type(v: ExhaustStructFieldType) -> &'static str {
    match v {
        ExhaustStructFieldType::Aaa(_) => "aaa",
        ExhaustStructFieldType::Bbb(_) => "bbb",
        ExhaustStructFieldType::Ccc(_) => "ccc",
    }
}

fn match_field_name(v: ExhaustStructFieldName) -> &'static str {
    match v {
        ExhaustStructFieldName::Aaa => "aaa",
        ExhaustStructFieldName::Bbb => "bbb",
        ExhaustStructFieldName::Ccc => "ccc",
    }
}

#[test]
fn exhaustive_match_compiles_and_runs() {
    assert_eq!(match_field_type(ExhaustStructFieldType::Aaa(1)), "aaa");
    assert_eq!(match_field_type(ExhaustStructFieldType::Bbb(2)), "bbb");
    assert_eq!(match_field_type(ExhaustStructFieldType::Ccc(3)), "ccc");

    assert_eq!(match_field_name(ExhaustStructFieldName::Aaa), "aaa");
    assert_eq!(match_field_name(ExhaustStructFieldName::Bbb), "bbb");
    assert_eq!(match_field_name(ExhaustStructFieldName::Ccc), "ccc");
}

// FieldType with None and Some values round-trip
#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct OptionalFields {
    required: i32,
    optional_str: Option<String>,
    optional_num: Option<u64>,
    nested_opt: Option<Option<bool>>,
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

// Section 25: Public struct inside private module
mod visibility_mod {
    use struct_to_enum::{FieldName, FieldType};

    #[derive(FieldType, FieldName)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct PublicInPrivate {
        pub x: i32,
        pub y: i32,
    }
}

use visibility_mod::{PublicInPrivate, PublicInPrivateFieldName, PublicInPrivateFieldType};

#[test]
fn public_struct_in_private_module() {
    let s = PublicInPrivate { x: 3, y: 4 };
    let fields: [PublicInPrivateFieldType; 2] = s.into();
    assert_eq!(fields[0], PublicInPrivateFieldType::X(3));
    assert_eq!(fields[1], PublicInPrivateFieldType::Y(4));

    let names = <PublicInPrivate as FieldNames<2>>::field_names();
    assert_eq!(
        names,
        [PublicInPrivateFieldName::X, PublicInPrivateFieldName::Y]
    );
}

// Debug formatting for both macros
#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DebugCheck {
    number: i64,
    text: String,
}

#[test]
fn debug_formatting() {
    let ft = DebugCheckFieldType::Number(-42i64);
    assert_eq!(format!("{:?}", ft), "Number(-42)");

    let ft = DebugCheckFieldType::Text("hello".to_string());
    assert_eq!(format!("{:?}", ft), "Text(\"hello\")");

    let fn_v = DebugCheckFieldName::Number;
    assert_eq!(format!("{:?}", fn_v), "Number");

    let fn_v = DebugCheckFieldName::Text;
    assert_eq!(format!("{:?}", fn_v), "Text");
}
