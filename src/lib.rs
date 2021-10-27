pub mod bindings;
pub mod error;
pub mod rfc;

pub use crate::{
    error::RfcErrorInfo,
    rfc::{RfcConnection, RfcConnectionBuilder, RfcFunction},
};

mod macros {
    macro_rules! is_rc_err {
        ($expr:expr) => {
            $expr != crate::bindings::_RFC_RC_RFC_OK
        };
    }

    pub(crate) use is_rc_err;

    macro_rules! check_rc_ok {
        ($expr:expr , $error:ident) => {
            if is_rc_err!($expr) {
                return Err($error);
            }
        };
        ($fn:ident ( $($args:expr),+ , ) ) => {
            check_rc_ok!($fn($($args),*));
        };
        ($fn:ident ( $($args:expr),+ ) ) => {
            let mut err_info = crate::error::RfcErrorInfo::new();
            check_rc_ok!($fn($($args),*, &mut err_info), err_info);
        };
        ($fn:ident ( ) ) => {
            let mut err_info = crate::error::RfcErrorInfo::new();
            check_rc_ok!($fn(&mut err_info), err_info);
        };
    }

    pub(crate) use check_rc_ok;

    macro_rules! from_sap_str {
        ($expr:expr) => {
            crate::rfc::SAPUCStr::from_slice_truncate($expr)
        };
    }

    pub(crate) use from_sap_str;

    macro_rules! to_sap_str {
        ($expr:expr) => {
            crate::rfc::SAPUCString::from_str_truncate($expr).into_vec_with_nul()
        };
    }

    pub(crate) use to_sap_str;
}
