use crate::uc;
use sapnwrfc_sys::{
    RFC_ERROR_INFO, _RFC_ERROR_GROUP_EXTERNAL_APPLICATION_FAILURE, _RFC_RC_RFC_UNKNOWN_ERROR,
};
use std::{error, fmt, result, string};

pub type Result<T> = result::Result<T, RfcErrorInfo>;

#[repr(transparent)]
#[derive(Default)]
pub struct RfcErrorInfo {
    inner: RFC_ERROR_INFO,
}

impl RfcErrorInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn custom(message: &str) -> Self {
        let mut slf = Self::new();
        slf.inner.code = _RFC_RC_RFC_UNKNOWN_ERROR;
        slf.inner.group = _RFC_ERROR_GROUP_EXTERNAL_APPLICATION_FAILURE;
        uc::from_str_to_slice(message, &mut slf.inner.message)
            .expect("Invalid custom error message string");
        slf
    }

    pub fn key(&self) -> String {
        uc::to_string_truncate(&self.inner.key).expect("Invalid RFC error key string")
    }

    pub fn message(&self) -> String {
        uc::to_string_truncate(&self.inner.message).expect("Invalid RFC error message string")
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut RFC_ERROR_INFO {
        &mut self.inner
    }
}

unsafe impl Send for RfcErrorInfo {}

impl fmt::Display for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.key(), self.message()))
    }
}

impl fmt::Debug for RfcErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RfcErrorInfo")
            .field("code", &self.inner.code)
            .field("group", &self.inner.group)
            .field("key", &self.key())
            .field("message", &self.message())
            .field(
                "abapMsgClass",
                &uc::to_string_truncate(&self.inner.abapMsgClass).unwrap(),
            )
            .field(
                "abapMsgType",
                &uc::to_string_truncate(&self.inner.abapMsgType).unwrap(),
            )
            .field(
                "abapMsgNumber",
                &uc::to_string_truncate(&self.inner.abapMsgNumber).unwrap(),
            )
            .field(
                "abapMsgV1",
                &uc::to_string_truncate(&self.inner.abapMsgV1).unwrap(),
            )
            .field(
                "abapMsgV2",
                &uc::to_string_truncate(&self.inner.abapMsgV2).unwrap(),
            )
            .field(
                "abapMsgV3",
                &uc::to_string_truncate(&self.inner.abapMsgV3).unwrap(),
            )
            .field(
                "abapMsgV4",
                &uc::to_string_truncate(&self.inner.abapMsgV4).unwrap(),
            )
            .finish()
    }
}

impl error::Error for RfcErrorInfo {}

impl From<string::FromUtf8Error> for RfcErrorInfo {
    fn from(src: string::FromUtf8Error) -> Self {
        Self::custom(&src.to_string())
    }
}
