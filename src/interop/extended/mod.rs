use crate::bindings::das::das_function;
use std::ffi::c_char;
extern "C" {
    pub(crate) fn dasx_verif_fn(fun: *mut das_function, name: *mut c_char) -> bool;
}
