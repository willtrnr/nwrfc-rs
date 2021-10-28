use crate::{
    bindings::{
        RFC_ERROR_INFO, _RFC_ERROR_GROUP_EXTERNAL_APPLICATION_FAILURE, _RFC_RC_RFC_UNKNOWN_ERROR,
    },
    rfc::{from_sap_uc, to_sap_uc},
};
use std::{cmp, error, fmt, ptr, result};

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
        let uc_msg = to_sap_uc(message).expect("Invalid custom error message string");
        unsafe {
            ptr::copy_nonoverlapping(
                uc_msg.as_ptr(),
                slf.message.as_mut_ptr(),
                cmp::min(uc_msg.len(), slf.message.len()),
            );
        }
        slf
    }
}

impl fmt::Display for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}: {}",
            from_sap_uc(&self.key).unwrap(),
            from_sap_uc(&self.message).unwrap(),
        ))
    }
}

impl fmt::Debug for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RfcErrorInfo")
            .field("code", &self.code)
            .field("group", &self.group)
            .field("key", &from_sap_uc(&self.key).unwrap())
            .field("message", &from_sap_uc(&self.message).unwrap())
            .field("abapMsgClass", &from_sap_uc(&self.abapMsgClass).unwrap())
            .field("abapMsgType", &from_sap_uc(&self.abapMsgType).unwrap())
            .field("abapMsgNumber", &from_sap_uc(&self.abapMsgNumber).unwrap())
            .field("abapMsgV1", &from_sap_uc(&self.abapMsgV1).unwrap())
            .field("abapMsgV2", &from_sap_uc(&self.abapMsgV2).unwrap())
            .field("abapMsgV3", &from_sap_uc(&self.abapMsgV3).unwrap())
            .field("abapMsgV4", &from_sap_uc(&self.abapMsgV4).unwrap())
            .finish()
    }
}

impl error::Error for RfcErrorInfo {}

impl From<std::string::FromUtf8Error> for RfcErrorInfo {
    fn from(src: std::string::FromUtf8Error) -> Self {
        Self::custom(&src.to_string())
    }
}
