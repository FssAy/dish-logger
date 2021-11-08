use std::os::raw::c_char;
use super::error::Error;


#[repr(C)]
pub struct Export {
    pub message: *const c_char,
    pub error: u8,
}

impl Export {
    pub fn new_token(token: *const c_char) -> Self {
        Self {
            message: token,
            error: 0,
        }
    }

    pub fn new_error(error: Error, message: *const c_char) -> Self {
        Self {
            message,
            error: *error,
        }
    }
}
