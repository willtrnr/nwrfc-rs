pub mod connection;
pub mod error;
pub mod function;
pub mod parameter;
pub mod structure;
pub mod table;
pub mod uc;

#[cfg(feature = "deadpool")]
pub mod pool;

pub use crate::{
    connection::{RfcConnection, RfcConnectionBuilder},
    error::RfcErrorInfo,
    function::RfcFunction,
    parameter::RfcParameter,
    structure::RfcStructure,
    table::RfcTable,
};

#[allow(clippy::single_component_path_imports)]
mod macros {
    macro_rules! is_rc_err {
        ($expr:expr) => {
            $expr != sapnwrfc_sys::_RFC_RC::RFC_OK
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
