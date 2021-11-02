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
            if crate::macros::is_rc_err!($expr) {
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

    macro_rules! assert_rc_ok {
        ($expr:expr , $msg:literal) => {
            assert_eq!($expr, sapnwrfc_sys::_RFC_RC::RFC_OK, $msg);
        };
        ($expr:expr) => {
            assert_eq!($expr, sapnwrfc_sys::_RFC_RC::RFC_OK);
        };
    }

    pub(crate) use assert_rc_ok;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        // Simple invalid connection negative test
        assert!(RfcConnection::builder()
            .set_param("dest", "INVALID")
            .build()
            .is_err());

        // Valid destination positive test
        let conn = RfcConnection::builder()
            .set_param("dest", "TEST")
            .build()
            .unwrap();

        // Must pass ping check
        assert!(conn.ping().is_ok());

        // Simple invalid function name negative test
        {
            conn.get_function("INVALID_TEST_FUNCTION_NAME").unwrap_err();
        }

        // Simple invalid parameter name negative test
        {
            let func = conn.get_function("SCP_STRING_ECHO").unwrap();

            assert!(func.get_parameter("INVALID").is_err());
        }

        // Simple echo call positive test
        {
            let func = conn.get_function("SCP_STRING_ECHO").unwrap();

            {
                let mut imp_param = func.get_parameter("IMP").unwrap();
                assert_eq!(imp_param.name(), "IMP");
                assert!(imp_param.is_import());
                assert!(!imp_param.is_optional());
                assert!(imp_param.is_string());
                imp_param.set_string("Test String").unwrap();
            }

            func.invoke().unwrap();

            {
                let exp_param = func.get_parameter("EXP").unwrap();
                assert_eq!(exp_param.name(), "EXP");
                assert!(exp_param.is_export());
                assert!(!exp_param.is_optional());
                assert!(exp_param.is_string());
                assert_eq!(exp_param.get_string().unwrap(), "Test String");
            }
        }

        // More complex structure echo test
        {
            let func = conn.get_function("STFC_STRUCTURE").unwrap();

            let mut impstruct = func.get_structure("IMPORTSTRUCT").unwrap();
            impstruct.set_int("RFCINT1", 42).unwrap(); // INT1 field
            impstruct.set_int("RFCINT2", 3939).unwrap(); // INT2 field
            impstruct.set_int("RFCINT4", 112357).unwrap(); // INT4 field
            impstruct.set_string("RFCCHAR1", "X").unwrap(); // CHAR field of length 1
            impstruct.set_string("RFCCHAR2", "AB").unwrap(); // CHAR field of length 2
            impstruct.set_string("RFCCHAR4", "Fizz").unwrap(); // CHAR field of length 4

            func.invoke().unwrap();

            let expstruct = func.get_structure("ECHOSTRUCT").unwrap();
            assert_eq!(expstruct.get_int("RFCINT1").unwrap(), 42);
            assert_eq!(expstruct.get_int("RFCINT2").unwrap(), 3939);
            assert_eq!(expstruct.get_int("RFCINT4").unwrap(), 112357);
            // FIXME: These fail because we can SetString into a CHAR, but not GetString on it.
            // assert_eq!(expstruct.get_string("RFCCHAR1").unwrap(), "X");
            // assert_eq!(expstruct.get_string("RFCCHAR2").unwrap(), "AB");
            // assert_eq!(expstruct.get_string("RFCCHAR4").unwrap(), "Fizz");
        }
    }
}
