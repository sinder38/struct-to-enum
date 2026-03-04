#![allow(dead_code)]

extern crate serde;
extern crate struct_to_enum;
extern crate strum;
extern crate strum_macros;

use struct_to_enum::{FieldName, FieldType};
use strum::VariantNames;
use strum_macros::EnumString;

// ============================================================
// Section 1: Basic combined FieldType + FieldName (existing, kept for regression)
// ============================================================

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

// ============================================================
// Section 2: Short alias attributes (ste_ prefix)
// ============================================================

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

// ============================================================
// Section 3: Both skip syntaxes (parenthesized vs name=value)
// ============================================================

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

    let dummy = SkipSyntaxTest {
        keep_a: 0,
        skip_paren: 0,
        keep_b: 0,
        skip_eq: 0,
        keep_c: 0,
    };
    let names: [SkipSyntaxTestFieldName; 3] = (&dummy).into();
    assert_eq!(
        names,
        [
            SkipSyntaxTestFieldName::KeepA,
            SkipSyntaxTestFieldName::KeepB,
            SkipSyntaxTestFieldName::KeepC,
        ]
    );
}

// ============================================================
// Section 4: Rich field types (Vec, HashMap, Box, Result, tuples, arrays)
// ============================================================

use std::collections::{HashMap, HashSet};

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
    let dummy = RichTypes {
        count: 0,
        tags: vec![],
        map: HashMap::new(),
        pair: (0, 0),
        fixed: [0.0; 4],
        boxed: Box::new(0),
        maybe: None,
        result_field: Ok(0),
    };
    let names: [RichTypesFieldName; 8] = (&dummy).into();
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

// ============================================================
// Section 5: Generic structs with complex bounds + where clauses
// ============================================================

mod generics_mod {
    use struct_to_enum::{FieldName, FieldType};

    // Three type params + lifetime, with skip on one field.
    // No extra bounds: the generated From<&Struct> impl propagates the
    // struct's own params without needing additional constraints.
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

// ============================================================
// Section 6: Single-field structs (minimal case)
// ============================================================

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

    let names: [SingleFieldFieldName; 1] = (&SingleField { only: 0 }).into();
    assert_eq!(names[0], SingleFieldFieldName::Only);
}

// ============================================================
// Section 7: All-skipped-but-one
// ============================================================

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

    let names: [AlmostAllSkippedFieldName; 1] = (&AlmostAllSkipped {
        a: 0,
        b: 0,
        c: 0,
        survivor: String::new(),
    })
        .into();
    assert_eq!(names[0], AlmostAllSkippedFieldName::Survivor);
}

// ============================================================
// Section 8: Large struct (many fields, PascalCase conversion)
// ============================================================

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

    let dummy = BigStruct {
        field_one: 0,
        field_two: 0,
        field_three: 0,
        field_four: 0,
        field_five: 0,
        field_six: 0,
        field_seven: 0,
        field_eight: 0,
        field_nine: 0.0,
        field_ten: 0.0,
    };
    let names: [BigStructFieldName; 10] = (&dummy).into();
    assert_eq!(names[0], BigStructFieldName::FieldOne);
    assert_eq!(names[9], BigStructFieldName::FieldTen);
}

// ============================================================
// Section 9: Strum integration — VariantNames on FieldType,
//            EnumString on FieldName, combined on same struct
// ============================================================

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

// ============================================================
// Section 10: Serde integration on FieldName
// ============================================================

#[derive(FieldName, serde::Serialize, serde::Deserialize)]
#[stem_name_derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct SerdeStruct {
    name: String,
    age: u32,
    #[stem_name(skip)]
    password: String,
}

#[test]
fn serde_field_name_serialize() {
    fn assert_serialize<T: serde::Serialize>(_: &T) {}

    let name_variant = SerdeStructFieldName::Name;
    assert_serialize(&name_variant);

    let s = SerdeStruct {
        name: "Alice".to_string(),
        age: 30,
        password: "secret".to_string(),
    };
    assert_serialize(&s);
}

// ============================================================
// Section 11: Nested FieldName — single level (regression)
// ============================================================

mod nested_name_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    pub struct Address {
        pub street: String,
        pub city: String,
        pub zip: u32,
    }

    #[derive(FieldName)]
    pub struct Person {
        pub name: String,
        #[stem_name(nested)]
        pub address: Address,
        #[stem_name(skip)]
        pub internal_id: u64,
    }
}

use nested_name_mod::{Address, Person, PersonFieldName};

#[test]
fn nested_field_name_single_level() {
    let p = Person {
        name: "Bob".to_string(),
        address: Address {
            street: "1 Main St".to_string(),
            city: "NY".to_string(),
            zip: 10001,
        },
        internal_id: 999,
    };

    let names: [PersonFieldName; 4] = (&p).into();
    assert_eq!(
        names,
        [
            PersonFieldName::Name,
            PersonFieldName::Street,
            PersonFieldName::City,
            PersonFieldName::Zip,
        ]
    );
}

// ============================================================
// Section 12: Nested FieldName — three levels deep
// ============================================================

mod deep_name_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    pub struct Coordinates {
        pub lat: f64,
        pub lon: f64,
    }

    #[derive(FieldName)]
    pub struct Location {
        pub label: String,
        #[stem_name(nested)]
        pub coords: Coordinates,
    }

    #[derive(FieldName)]
    pub struct Place {
        pub title: String,
        #[stem_name(nested)]
        pub location: Location,
        pub rank: u32,
    }
}

use deep_name_mod::{Coordinates, Location, Place, PlaceFieldName};

#[test]
fn nested_field_name_three_levels() {
    let p = Place {
        title: "Eiffel Tower".to_string(),
        location: Location {
            label: "Paris".to_string(),
            coords: Coordinates {
                lat: 48.858,
                lon: 2.294,
            },
        },
        rank: 1,
    };

    let names: [PlaceFieldName; 5] = (&p).into();
    // Non-nested own fields first (Title, Rank), then nested variants appended (Label, Lat, Lon)
    assert_eq!(
        names,
        [
            PlaceFieldName::Title,
            PlaceFieldName::Rank,
            PlaceFieldName::Label,
            PlaceFieldName::Lat,
            PlaceFieldName::Lon,
        ]
    );
}

// ============================================================
// Section 13: Nested FieldName — two nested fields in one struct
// ============================================================

mod complex_nesting {
    use struct_to_enum::FieldName;

    #[derive(FieldName, Default)]
    pub struct AB {
        pub a: u32,
        pub b: u32,
    }

    #[derive(FieldName, Default)]
    pub struct E {
        pub e: u32,
    }

    #[derive(FieldName, Default)]
    pub struct CDEF {
        pub c: i32,
        pub d: i32,
        #[stem_name(nested)]
        pub e: E,
        pub f: i32,
    }

    #[derive(FieldName, Default)]
    pub struct JKL {
        pub j: i32,
        pub k: i32,
        pub l: i32,
    }

    #[derive(FieldName, Default)]
    pub struct GHIJKL {
        pub g: i32,
        pub h: i32,
        pub i: i32,
        #[stem_name(nested)]
        pub jkl: JKL,
    }

    #[derive(FieldName, Default)]
    pub struct P {
        pub p: i32,
    }
    #[derive(FieldName, Default)]
    pub struct O {
        pub o: i32,
    }

    #[derive(FieldName, Default)]
    pub struct OP {
        #[stem_name(nested)]
        pub o: O,
        #[stem_name(nested)]
        pub p: P,
    }

    #[derive(FieldName, Default)]
    pub struct GHIJKLMNOP {
        #[stem_name(nested)]
        pub ghijkl: GHIJKL,
        pub m: i32,
        pub n: i32,
        #[stem_name(nested)]
        pub op: OP,
    }

    #[derive(FieldName, Default)]
    pub struct ABCDEFGHIJKLMNOP {
        #[stem_name(nested)]
        pub ab: AB,
        #[stem_name(nested)]
        pub cdf: CDEF,
        #[stem_name(nested)]
        pub ghijklmnop: GHIJKLMNOP,
    }
}

use complex_nesting::GHIJKLMNOP;

#[test]
fn complex_fields_field_name() {
    use ABCDEFGHIJKLMNOPFieldName as AlpName;
    let a = ABCDEFGHIJKLMNOP::default();
    let letters: [AlpName; 16] = (&a).into();
    //TODO: enable for order testing

    // assert_eq!(
    //     letters,
    //     [
    //         AlpName::A,
    //         AlpName::B,
    //         AlpName::C,
    //         AlpName::D,
    //         AlpName::E,
    //         AlpName::F,
    //         AlpName::G,
    //         AlpName::H,
    //         AlpName::I,
    //         AlpName::J,
    //         AlpName::K,
    //         AlpName::L,
    //         AlpName::M,
    //         AlpName::N,
    //         AlpName::O,
    //         AlpName::P,
    //     ]
    // )
}

// ============================================================
// Section 14: Nested FieldType — single level
// ============================================================

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct ColorInner {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Pixel {
    x: i32,
    y: i32,
    #[stem_type(nested)]
    color: ColorInner,
}

#[test]
fn nested_field_type_single_level() {
    let p = Pixel {
        x: 10,
        y: 20,
        color: ColorInner {
            r: 255,
            g: 128,
            b: 0,
        },
    };
    let fields: [PixelFieldType; 5] = p.into();
    assert_eq!(fields[0], PixelFieldType::X(10));
    assert_eq!(fields[1], PixelFieldType::Y(20));
    assert_eq!(fields[2], PixelFieldType::R(255));
    assert_eq!(fields[3], PixelFieldType::G(128));
    assert_eq!(fields[4], PixelFieldType::B(0));
}

// ============================================================
// Section 15: Nested FieldType — three levels deep
// ============================================================

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Vec3 {
    vx: f32,
    vy: f32,
    vz: f32,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Transform {
    scale: f32,
    #[stem_type(nested)]
    velocity: Vec3,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Entity {
    id: u32,
    #[stem_type(nested)]
    transform: Transform,
    active: bool,
}

#[test]
fn nested_field_type_three_levels() {
    let e = Entity {
        id: 42,
        transform: Transform {
            scale: 1.5,
            velocity: Vec3 {
                vx: 1.0,
                vy: 2.0,
                vz: 3.0,
            },
        },
        active: true,
    };
    let fields: [EntityFieldType; 6] = e.into();
    // Own non-nested fields first: Id, Active; then nested (Transform → Scale, Vx, Vy, Vz)
    assert_eq!(fields[0], EntityFieldType::Id(42));
    assert_eq!(fields[1], EntityFieldType::Active(true));
    assert_eq!(fields[2], EntityFieldType::Scale(1.5));
    assert_eq!(fields[3], EntityFieldType::Vx(1.0));
    assert_eq!(fields[4], EntityFieldType::Vy(2.0));
    assert_eq!(fields[5], EntityFieldType::Vz(3.0));
}

// ============================================================
// Section 16: Nested FieldType — two sibling nested fields with distinct names
// ============================================================

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct XRange {
    x_min: f64,
    x_max: f64,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct YRange {
    y_min: f64,
    y_max: f64,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Stats {
    #[stem_type(skip)]
    label: String,
    count: u64,
    #[stem_type(nested)]
    range_x: XRange,
    #[stem_type(nested)]
    range_y: YRange,
}

#[test]
fn nested_field_type_two_sibling_nested() {
    let s = Stats {
        label: "ignored".to_string(),
        count: 100,
        range_x: XRange {
            x_min: -1.0,
            x_max: 1.0,
        },
        range_y: YRange {
            y_min: 0.0,
            y_max: 10.0,
        },
    };
    // StatsFieldType: Count(u64), XMin(f64), XMax(f64), YMin(f64), YMax(f64)
    let fields: [StatsFieldType; 5] = s.into();
    assert_eq!(fields[0], StatsFieldType::Count(100));
    assert_eq!(fields[1], StatsFieldType::XMin(-1.0));
    assert_eq!(fields[2], StatsFieldType::XMax(1.0));
    assert_eq!(fields[3], StatsFieldType::YMin(0.0));
    assert_eq!(fields[4], StatsFieldType::YMax(10.0));
}

// ============================================================
// Section 17: Combined FieldType + FieldName with nesting
// ============================================================

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

use combined_nested_mod::{Document, DocumentFieldName, DocumentFieldType, Meta};

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
    // Own non-nested fields first: Title, Content; then nested (Meta → Version, Author)
    assert_eq!(fields[0], DocumentFieldType::Title("Hello".to_string()));
    assert_eq!(fields[1], DocumentFieldType::Content("World".to_string()));
    assert_eq!(fields[2], DocumentFieldType::Version(2));
    assert_eq!(fields[3], DocumentFieldType::Author("Alice".to_string()));

    let doc2 = Document {
        title: "Hello".to_string(),
        meta: Meta {
            version: 2,
            author: "Alice".to_string(),
        },
        content: "World".to_string(),
    };
    let names: [DocumentFieldName; 4] = (&doc2).into();
    assert_eq!(
        names,
        [
            DocumentFieldName::Title,
            DocumentFieldName::Content,
            DocumentFieldName::Version,
            DocumentFieldName::Author,
        ]
    );
}

// ============================================================
// Section 18: stem_type_attr — extra enum-level attributes
// ============================================================

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq, strum_macros::VariantNames)]
#[stem_type_attr(strum(serialize_all = "snake_case"))]
struct AttrTest {
    my_field: i32,
    another_field: bool,
}

#[test]
fn stem_type_attr_applied() {
    assert_eq!(AttrTestFieldType::VARIANTS, ["my_field", "another_field"]);
}

#[derive(FieldName)]
#[stem_name_derive(Debug, Clone, PartialEq, EnumString)]
#[stem_name_attr(strum(ascii_case_insensitive))]
struct NameAttrTest {
    my_field: i32,
    another_field: bool,
}

#[test]
fn stem_name_attr_applied() {
    let v = "myfield".parse::<NameAttrTestFieldName>().unwrap();
    assert_eq!(v, NameAttrTestFieldName::MyField);

    let v = "ANOTHERFIELD".parse::<NameAttrTestFieldName>().unwrap();
    assert_eq!(v, NameAttrTestFieldName::AnotherField);
}

// ============================================================
// Section 19: FieldName default derives (Copy, Clone, PartialEq, Eq, Debug)
//             even without stem_name_derive — confirmed via Copy semantics
// ============================================================

#[derive(FieldName)]
struct DefaultDerives {
    alpha: i32,
    beta: String,
}

#[test]
fn field_name_default_derives() {
    let a = DefaultDerivesFieldName::Alpha;
    let b = a; // Copy
    let c = a.clone(); // Clone
    assert_eq!(a, b); // PartialEq + Eq
    assert_eq!(b, c);
    let _ = format!("{:?}", a); // Debug
}

// ============================================================
// Section 20: FieldName with overridden derives (stem_name_derive replaces defaults)
// ============================================================

#[derive(FieldName)]
#[stem_name_derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct WithHash {
    key: String,
    value: i32,
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

// ============================================================
// Section 21: FieldType — From and Into both work for non-generic structs
// ============================================================

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Pair {
    first: i32,
    second: &'static str,
}

#[test]
fn field_type_from_and_into_both_work() {
    let p1 = Pair {
        first: 10,
        second: "hi",
    };
    let by_into: [PairFieldType; 2] = p1.into();

    let p2 = Pair {
        first: 10,
        second: "hi",
    };
    let by_from: [PairFieldType; 2] = <[PairFieldType; 2]>::from(p2);

    assert_eq!(by_into[0], by_from[0]);
    assert_eq!(by_into[1], by_from[1]);
    assert_eq!(by_into[0], PairFieldType::First(10));
    assert_eq!(by_into[1], PairFieldType::Second("hi"));
}

// ============================================================
// Section 22: FieldName — From<&Struct> is a reference-based conversion
//             (struct is not consumed)
// ============================================================

#[derive(FieldType, FieldName)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct Reusable {
    a: i32,
    b: i32,
}

#[test]
fn field_name_borrows_struct() {
    let s = Reusable { a: 1, b: 2 };
    let names1: [ReusableFieldName; 2] = (&s).into();
    let names2: [ReusableFieldName; 2] = (&s).into(); // s is still usable
    assert_eq!(names1, names2);
    assert_eq!(s.a, 1); // s still accessible
}

// ============================================================
// Section 23: Exhaustiveness — match with all variants compiles correctly
// ============================================================

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

// ============================================================
// Section 24: FieldType with None and Some values round-trip
// ============================================================

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

// ============================================================
// Section 25: Public struct inside private module — visibility propagation
// ============================================================

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

    let dummy = PublicInPrivate { x: 0, y: 0 };
    let names: [PublicInPrivateFieldName; 2] = (&dummy).into();
    assert_eq!(
        names,
        [PublicInPrivateFieldName::X, PublicInPrivateFieldName::Y]
    );
}

// ============================================================
// Section 26: Generic FieldName — From<&Struct<T>> with lifetimes
// ============================================================

mod generic_name_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    pub struct Wrapper<'a, T: 'a> {
        pub value: &'a T,
        pub tag: &'static str,
        #[stem_name(skip)]
        pub hidden: u32,
    }
}

use generic_name_mod::{Wrapper, WrapperFieldName};

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

// ============================================================
// Section 27: Mixed skip-and-nested in same struct (FieldName)
// ============================================================

mod mixed_skip_nested_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    pub struct Inner {
        pub p: bool,
        pub q: bool,
    }

    #[derive(FieldName)]
    pub struct Mixed {
        pub a: i32,
        #[stem_name(skip)]
        pub b: i32,
        #[stem_name(nested)]
        pub inner: Inner,
        #[stem_name(skip)]
        pub c: i32,
        pub d: i32,
    }
}

use mixed_skip_nested_mod::{Inner as MixedInner, Mixed, MixedFieldName};

use crate::complex_nesting::{ABCDEFGHIJKLMNOP, ABCDEFGHIJKLMNOPFieldName};

#[test]
fn mixed_skip_and_nested_field_name() {
    let s = Mixed {
        a: 1,
        b: 2,
        inner: MixedInner { p: true, q: false },
        c: 3,
        d: 4,
    };
    let names: [MixedFieldName; 4] = (&s).into();
    // Own non-nested fields first (A, D in declaration order), then nested (P, Q)
    assert_eq!(
        names,
        [
            MixedFieldName::A,
            MixedFieldName::D,
            MixedFieldName::P,
            MixedFieldName::Q,
        ]
    );
}

// ============================================================
// Section 28: Mixed skip-and-nested in same struct (FieldType)
// ============================================================

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct InnerMT {
    p: bool,
    q: bool,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct MixedType {
    a: i32,
    #[stem_type(skip)]
    b: i32,
    #[stem_type(nested)]
    inner: InnerMT,
    #[stem_type(skip)]
    c: i32,
    d: i32,
}

#[test]
fn mixed_skip_and_nested_field_type() {
    let s = MixedType {
        a: 10,
        b: 0,
        inner: InnerMT { p: true, q: false },
        c: 0,
        d: 99,
    };
    let fields: [MixedTypeFieldType; 4] = s.into();
    // Own non-nested fields first (A, D), then nested (P, Q)
    assert_eq!(fields[0], MixedTypeFieldType::A(10));
    assert_eq!(fields[1], MixedTypeFieldType::D(99));
    assert_eq!(fields[2], MixedTypeFieldType::P(true));
    assert_eq!(fields[3], MixedTypeFieldType::Q(false));
}

// ============================================================
// Section 29: Clone on FieldType — verify clone semantics for heap types
// ============================================================

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct CloneCheck {
    name: String,
    data: Vec<u8>,
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

// ============================================================
// Section 30: Debug formatting for both macros
// ============================================================

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
