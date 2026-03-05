#![allow(dead_code)]
use struct_to_enum::FieldName;
// Utility file for macro expansion debugging (cargo expand --test expand)

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

// #[derive(FieldName)]
struct DeepOuter {
    top: bool,
    // #[stem_name(nested)]
    middle: DeepMiddle,
}

enum DeepOuterFieldName {
    Top,
    M,
    Z,
}

#[automatically_derived]
impl
    ::struct_to_enum::FieldNames<
        {
            let mut _n = 0usize;
            {
                let _ = "Top";
                _n += 1;
            }
            {
                let _ = "M";
                _n += 1;
            }
            {
                let _ = "Z";
                _n += 1;
            }
            _n
        },
    > for DeepOuter
{
    type FieldName = DeepOuterFieldName;
    fn field_names() -> [Self::FieldName; {
        let mut _n = 0usize;
        {
            let _ = "Top";
            _n += 1;
        }
        {
            let _ = "M";
            _n += 1;
        }
        {
            let _ = "Z";
            _n += 1;
        }
        _n
    }] {
        [
            DeepOuterFieldName::Top,
            DeepOuterFieldName::M,
            DeepOuterFieldName::Z,
        ]
    }
}
