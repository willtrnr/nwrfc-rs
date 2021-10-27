#![allow(clippy::redundant_static_lifetimes)]
#![allow(deref_nullptr)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use libc::{size_t, time_t};

type char16_t = u16;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
