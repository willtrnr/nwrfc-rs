pub mod error;
pub mod rfc;
pub mod uc;

pub use crate::{
    error::RfcErrorInfo,
    rfc::{RfcConnection, RfcConnectionBuilder, RfcFunction},
};

mod macros {
    macro_rules! is_rc_err {
        ($expr:expr) => {
            $expr != sapnwrfc_sys::_RFC_RC_RFC_OK
        };
    }

    pub(crate) use is_rc_err;

    macro_rules! check_rc_ok {
        ($expr:expr , $error:ident) => {
            if is_rc_err!($expr) {
                return Err($error);
            }
        };
        ($fn:ident ( $($args:expr),+ ) ) => {
            let mut err_info = crate::error::RfcErrorInfo::new();
            check_rc_ok!($fn($($args),*, err_info.as_mut_ptr()), err_info);
        };
        ($fn:ident ( ) ) => {
            let mut err_info = crate::error::RfcErrorInfo::new();
            check_rc_ok!($fn(&mut err_info), err_info);
        };
    }

    pub(crate) use check_rc_ok;
}
