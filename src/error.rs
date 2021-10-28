use crate::{
    bindings::{
        RFC_ERROR_INFO, _RFC_ERROR_GROUP_EXTERNAL_APPLICATION_FAILURE, _RFC_RC_RFC_UNKNOWN_ERROR,
    },
    rfc::{str_from_sap_uc, str_to_sap_uc_slice},
};
use std::{error, fmt, result, string};

pub type RfcErrorInfo = RFC_ERROR_INFO;

pub type Result<T> = result::Result<T, RfcErrorInfo>;

impl RfcErrorInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn custom(message: &str) -> Self {
        let mut slf = Self::new();
        slf.code = _RFC_RC_RFC_UNKNOWN_ERROR;
        slf.group = _RFC_ERROR_GROUP_EXTERNAL_APPLICATION_FAILURE;
        str_to_sap_uc_slice(message, &mut slf.message)
            .expect("Invalid custom error message string");
        slf
    }
}

impl fmt::Display for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}: {}",
            str_from_sap_uc(&self.key).unwrap(),
            str_from_sap_uc(&self.message).unwrap(),
        ))
    }
}

impl fmt::Debug for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RfcErrorInfo")
            .field("code", &self.code)
            .field("group", &self.group)
            .field("key", &str_from_sap_uc(&self.key).unwrap())
            .field("message", &str_from_sap_uc(&self.message).unwrap())
            .field(
                "abapMsgClass",
                &str_from_sap_uc(&self.abapMsgClass).unwrap(),
            )
            .field("abapMsgType", &str_from_sap_uc(&self.abapMsgType).unwrap())
            .field(
                "abapMsgNumber",
                &str_from_sap_uc(&self.abapMsgNumber).unwrap(),
            )
            .field("abapMsgV1", &str_from_sap_uc(&self.abapMsgV1).unwrap())
            .field("abapMsgV2", &str_from_sap_uc(&self.abapMsgV2).unwrap())
            .field("abapMsgV3", &str_from_sap_uc(&self.abapMsgV3).unwrap())
            .field("abapMsgV4", &str_from_sap_uc(&self.abapMsgV4).unwrap())
            .finish()
    }
}

impl error::Error for RfcErrorInfo {}

impl From<string::FromUtf8Error> for RfcErrorInfo {
    fn from(src: string::FromUtf8Error) -> Self {
        Self::custom(&src.to_string())
    }
}
