#![allow(dead_code)]

extern crate struct_to_enum;

use struct_to_enum::FieldType;

// Single level of nesting

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct InnerSimple {
    x: i32,
    y: i32,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct OuterSimple {
    a: bool,
    #[stem_type(nested)]
    inner: InnerSimple,
}

// Single level (RGB color in Pixel)

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

// Two levels deep

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DeepInner {
    z: f64,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DeepMiddle {
    m: i32,
    #[stem_type(nested)]
    deep: DeepInner,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct DeepOuter {
    top: bool,
    #[stem_type(nested)]
    middle: DeepMiddle,
}

// Three levels deep (Entity -> Transform -> Vec3)

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

// Two sibling nested fields with distinct field names

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct LeftInner {
    lx: i32,
    ly: i32,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct RightInner {
    rx: i32,
    ry: i32,
}

#[derive(FieldType)]
#[stem_type_derive(Debug, Clone, PartialEq)]
struct TwoNested {
    #[stem_type(skip)]
    skippa: String,
    own: u8,
    #[stem_type(nested)]
    left: LeftInner,
    #[stem_type(nested)]
    right: RightInner,
}

// Two sibling nested with skip on outer (Stats)

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

// Mixed skip and nested in same struct

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

use crate::nested::complex_nesting::{ABCDEFGHIJKLMNOP, ABCDEFGHIJKLMNOPFieldType};

// Complex nesting
mod complex_nesting {
    use struct_to_enum::FieldType;

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct AB {
        pub a: u32,
        pub b: u32,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct E {
        pub e: u32,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct CDEF {
        pub c: i32,
        pub d: i32,
        #[stem_type(nested)]
        pub e: E,
        pub f: i32,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct JKL {
        pub j: i32,
        pub k: i32,
        pub l: i32,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct GHIJKL {
        pub g: i32,
        pub h: i32,
        pub i: i32,
        #[stem_type(nested)]
        pub jkl: JKL,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct P {
        pub p: i32,
    }
    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct O {
        pub o: i32,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct OP {
        #[stem_type(nested)]
        pub o: O,
        #[stem_type(nested)]
        pub p: P,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct GHIJKLMNOP {
        #[stem_type(nested)]
        pub ghijkl: GHIJKL,
        pub m: i32,
        pub n: i32,
        #[stem_type(nested)]
        pub op: OP,
    }

    #[derive(FieldType, Default)]
    #[stem_type_derive(Debug, Clone, PartialEq)]
    pub struct ABCDEFGHIJKLMNOP {
        #[stem_type(nested)]
        pub ab: AB,
        #[stem_type(nested)]
        pub cdf: CDEF,
        #[stem_type(nested)]
        pub ghijklmnop: GHIJKLMNOP,
    }
}

#[test]
fn complex_fields_field_type() {
    use ABCDEFGHIJKLMNOPFieldType as AlpName;
    let a = ABCDEFGHIJKLMNOP::default();
    let letters: [AlpName; 16] = a.into();

    assert_eq!(
        letters,
        [
            AlpName::A(0),
            AlpName::B(0),
            AlpName::C(0),
            AlpName::D(0),
            AlpName::E(0),
            AlpName::F(0),
            AlpName::G(0),
            AlpName::H(0),
            AlpName::I(0),
            AlpName::J(0),
            AlpName::K(0),
            AlpName::L(0),
            AlpName::M(0),
            AlpName::N(0),
            AlpName::O(0),
            AlpName::P(0),
        ]
    )
}

#[test]
fn nested_field_type_variants() {
    let _top = DeepOuterFieldType::Top(true);
    let _m = DeepOuterFieldType::M(42);
    let _z = DeepOuterFieldType::Z(3.14);
    let v = DeepOuterFieldType::Top(false);
    match v {
        DeepOuterFieldType::Top(_) => (),
        DeepOuterFieldType::M(_) => (),
        DeepOuterFieldType::Z(_) => (),
    }
}

#[test]
fn nested_single_level_into() {
    let outer = OuterSimple {
        a: true,
        inner: InnerSimple { x: 1, y: 2 },
    };
    let fields: [OuterSimpleFieldType; 3] = outer.into();
    assert_eq!(fields[0], OuterSimpleFieldType::A(true));
    assert_eq!(fields[1], OuterSimpleFieldType::X(1));
    assert_eq!(fields[2], OuterSimpleFieldType::Y(2));
}

#[test]
fn nested_pixel_single_level() {
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

#[test]
fn nested_two_levels_into() {
    let outer = DeepOuter {
        top: true,
        middle: DeepMiddle {
            m: 7,
            deep: DeepInner { z: 2.5 },
        },
    };
    let fields: [DeepOuterFieldType; 3] = outer.into();
    assert_eq!(fields[0], DeepOuterFieldType::Top(true));
    assert_eq!(fields[1], DeepOuterFieldType::M(7));
    assert_eq!(fields[2], DeepOuterFieldType::Z(2.5));
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
    // Declaration order: id, transform->(scale, vx, vy, vz), active
    assert_eq!(fields[0], EntityFieldType::Id(42));
    assert_eq!(fields[1], EntityFieldType::Scale(1.5));
    assert_eq!(fields[2], EntityFieldType::Vx(1.0));
    assert_eq!(fields[3], EntityFieldType::Vy(2.0));
    assert_eq!(fields[4], EntityFieldType::Vz(3.0));
    assert_eq!(fields[5], EntityFieldType::Active(true));
}

#[test]
fn nested_two_nested_fields_into() {
    let s = TwoNested {
        own: 99,
        left: LeftInner { lx: 1, ly: 2 },
        right: RightInner { rx: 3, ry: 4 },
        skippa: "test".to_string(),
    };
    let fields: [TwoNestedFieldType; 5] = s.into();
    assert_eq!(fields[0], TwoNestedFieldType::Own(99));
    assert_eq!(fields[1], TwoNestedFieldType::Lx(1));
    assert_eq!(fields[2], TwoNestedFieldType::Ly(2));
    assert_eq!(fields[3], TwoNestedFieldType::Rx(3));
    assert_eq!(fields[4], TwoNestedFieldType::Ry(4));
}

#[test]
fn nested_two_sibling_nested_with_skip() {
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
    let fields: [StatsFieldType; 5] = s.into();
    assert_eq!(fields[0], StatsFieldType::Count(100));
    assert_eq!(fields[1], StatsFieldType::XMin(-1.0));
    assert_eq!(fields[2], StatsFieldType::XMax(1.0));
    assert_eq!(fields[3], StatsFieldType::YMin(0.0));
    assert_eq!(fields[4], StatsFieldType::YMax(10.0));
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
    // Declaration order: a, inner->(p, q), d
    assert_eq!(fields[0], MixedTypeFieldType::A(10));
    assert_eq!(fields[1], MixedTypeFieldType::P(true));
    assert_eq!(fields[2], MixedTypeFieldType::Q(false));
    assert_eq!(fields[3], MixedTypeFieldType::D(99));
}
