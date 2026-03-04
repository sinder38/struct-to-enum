#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldName;

// --- Single level of nesting ---

mod single_level {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    pub struct Inner {
        pub x: i32,
        pub y: String,
    }

    #[derive(FieldName)]
    pub struct Outer {
        pub a: bool,
        #[stem_name(nested)]
        pub inner: Inner,
    }
}

use single_level::{Inner as SingleInner, Outer as SingleOuter, OuterFieldName};

// --- Single level (real-world: Address in Person) ---

mod address_mod {
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

use address_mod::{Address, Person, PersonFieldName};

// --- Two levels deep ---

#[derive(FieldName)]
struct DeepInner {
    z: f64,
}

#[derive(FieldName)]
struct DeepMiddle {
    m: i32,
    #[stem_name(nested)]
    deep: DeepInner,
}

#[derive(FieldName)]
struct DeepOuter {
    top: bool,
    #[stem_name(nested)]
    middle: DeepMiddle,
}

// --- Three levels (Place → Location → Coordinates) ---

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

// --- Two sibling nested fields in same struct ---

mod two_nested_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName, Default)]
    pub struct LeftPart {
        pub lx: i32,
        pub ly: i32,
    }

    #[derive(FieldName, Default)]
    pub struct RightPart {
        pub rx: i32,
        pub ry: i32,
    }

    #[derive(FieldName, Default)]
    pub struct TwoNested {
        pub own: u8,
        #[stem_name(nested)]
        pub left: LeftPart,
        #[stem_name(nested)]
        pub right: RightPart,
    }
}

use two_nested_mod::{LeftPart, RightPart, TwoNested, TwoNestedFieldName};

// --- Mixed skip and nested in same struct ---

mod mixed_skip_nested_mod {
    use struct_to_enum::FieldName;

    #[derive(FieldName)]
    pub struct InnerMixed {
        pub p: bool,
        pub q: bool,
    }

    #[derive(FieldName)]
    pub struct Mixed {
        pub a: i32,
        #[stem_name(skip)]
        pub b: i32,
        #[stem_name(nested)]
        pub inner: InnerMixed,
        #[stem_name(skip)]
        pub c: i32,
        pub d: i32,
    }
}

use mixed_skip_nested_mod::{InnerMixed as MixedInner, Mixed, MixedFieldName};

// ----------------------------------------------------------------

#[test]
fn nested_flat_field_name_variants() {
    let _a = OuterFieldName::A;
    let _x = OuterFieldName::X;
    let _y = OuterFieldName::Y;

    match OuterFieldName::X {
        OuterFieldName::A => panic!("wrong"),
        OuterFieldName::X => (),
        OuterFieldName::Y => panic!("wrong"),
    }
}

#[test]
fn nested_flat_field_name_array() {
    let outer = SingleOuter {
        a: true,
        inner: SingleInner {
            x: 0,
            y: String::new(),
        },
    };
    let fields: [OuterFieldName; 3] = (&outer).into();
    assert_eq!(
        fields,
        [OuterFieldName::A, OuterFieldName::X, OuterFieldName::Y]
    );
}

#[test]
fn nested_field_name_single_level_address() {
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

#[test]
fn nested_flat_deep_two_levels() {
    let outer = DeepOuter {
        top: true,
        middle: DeepMiddle {
            m: 0,
            deep: DeepInner { z: 0.0 },
        },
    };
    let fields: [DeepOuterFieldName; 3] = (&outer).into();
    assert_eq!(
        fields,
        [
            DeepOuterFieldName::Top,
            DeepOuterFieldName::M,
            DeepOuterFieldName::Z
        ]
    );
}

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
    assert_eq!(
        names,
        [
            PlaceFieldName::Title,
            PlaceFieldName::Label,
            PlaceFieldName::Lat,
            PlaceFieldName::Lon,
            PlaceFieldName::Rank,
        ]
    );
}

#[test]
fn nested_two_sibling_fields() {
    let s = TwoNested {
        own: 5,
        left: LeftPart { lx: 1, ly: 2 },
        right: RightPart { rx: 3, ry: 4 },
    };
    let names: [TwoNestedFieldName; 5] = (&s).into();
    assert_eq!(names[0], TwoNestedFieldName::Own);
    assert_eq!(names[1], TwoNestedFieldName::Lx);
    assert_eq!(names[2], TwoNestedFieldName::Ly);
    assert_eq!(names[3], TwoNestedFieldName::Rx);
    assert_eq!(names[4], TwoNestedFieldName::Ry);
}

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
    // Declaration order: a, (b skipped), inner→(p, q), (c skipped), d
    assert_eq!(
        names,
        [
            MixedFieldName::A,
            MixedFieldName::P,
            MixedFieldName::Q,
            MixedFieldName::D,
        ]
    );
}
