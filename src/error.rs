use crate::{
    bindings::{
        RFC_ERROR_INFO, _RFC_ERROR_GROUP_EXTERNAL_APPLICATION_FAILURE, _RFC_RC_RFC_UNKNOWN_ERROR,
    },
    macros::{from_sap_str, to_sap_str},
};
use std::{cmp, error, fmt, ptr};

pub type RfcErrorInfo = RFC_ERROR_INFO;

pub type Result<T> = std::result::Result<T, RfcErrorInfo>;

impl RfcErrorInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn custom(message: &str) -> Self {
        let mut slf = Self::new();
        slf.code = _RFC_RC_RFC_UNKNOWN_ERROR;
        slf.group = _RFC_ERROR_GROUP_EXTERNAL_APPLICATION_FAILURE;
        let c_msg = to_sap_str!(message);
        unsafe {
            ptr::copy_nonoverlapping(
                c_msg.as_ptr(),
                slf.message.as_mut_ptr(),
                cmp::min(c_msg.len(), slf.message.len() - 1),
            );
        }
        slf
    }
}

impl fmt::Display for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}: {}",
            from_sap_str!(&self.key).unwrap().display(),
            from_sap_str!(&self.message).unwrap().display(),
        ))
    }
}

impl fmt::Debug for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RfcErrorInfo")
            .field("code", &self.code)
            .field("group", &self.group)
            .field("key", &from_sap_str!(&self.key))
            .field("message", &from_sap_str!(&self.message))
            .field("abapMsgClass", &from_sap_str!(&self.abapMsgClass))
            .field("abapMsgType", &from_sap_str!(&self.abapMsgType))
            .field("abapMsgNumber", &from_sap_str!(&self.abapMsgNumber))
            .field("abapMsgV1", &from_sap_str!(&self.abapMsgV1))
            .field("abapMsgV2", &from_sap_str!(&self.abapMsgV2))
            .field("abapMsgV3", &from_sap_str!(&self.abapMsgV3))
            .field("abapMsgV4", &from_sap_str!(&self.abapMsgV4))
            .finish()
    }
}

impl error::Error for RfcErrorInfo {}

impl From<std::string::FromUtf16Error> for RfcErrorInfo {
    fn from(src: std::string::FromUtf16Error) -> Self {
        Self::custom(&src.to_string())
    }
}

impl<T> From<widestring::error::NulError<T>> for RfcErrorInfo {
    fn from(src: widestring::error::NulError<T>) -> Self {
        Self::custom(&src.to_string())
    }
}
