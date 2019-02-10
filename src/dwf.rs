#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn to_c_bool(v: bool) -> BOOL {
    if v {
        true_ as BOOL
    } else {
        false_ as BOOL
    }
}
