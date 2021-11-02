use crate::{
    error::{Result, RfcErrorInfo},
    macros::{check_rc_ok, is_rc_err},
};
use sapnwrfc_sys::{RfcSAPUCToUTF8, RfcUTF8ToSAPUC, RFC_ABAP_NAME, SAP_UC};

pub fn from_str_to_buffer(value: &str, dest: *mut SAP_UC, size: usize) -> Result<u32> {
    let mut size = size as u32;
    let mut res_len: u32 = 0;
    unsafe {
        check_rc_ok!(RfcUTF8ToSAPUC(
            value.as_ptr(),
            value.len() as u32,
            dest,
            &mut size,
            &mut res_len
        ));
    }
    Ok(res_len)
}

pub fn from_str_to_slice(value: &str, dest: &mut [SAP_UC]) -> Result<u32> {
    from_str_to_buffer(value, dest.as_mut_ptr(), dest.len())
}

pub fn from_str_to_abap_name(value: &str) -> Result<RFC_ABAP_NAME> {
    let mut uc_value: RFC_ABAP_NAME = Default::default();
    from_str_to_slice(value, &mut uc_value)?;
    Ok(uc_value)
}

pub fn from_str(value: &str) -> Result<Vec<SAP_UC>> {
    let mut err_info = RfcErrorInfo::new();
    let mut buf = Vec::with_capacity(value.len() + 1);
    let mut buf_len = buf.capacity() as u32;
    let mut res_len: u32 = 0;
    unsafe {
        let rc = RfcUTF8ToSAPUC(
            value.as_ptr(),
            value.len() as u32,
            buf.as_mut_ptr(),
            &mut buf_len,
            &mut res_len,
            err_info.as_mut_ptr(),
        );
        if rc == sapnwrfc_sys::_RFC_RC::RFC_BUFFER_TOO_SMALL {
            buf.reserve_exact(buf_len as usize + 1);
            buf_len = buf.capacity() as u32;
            check_rc_ok!(
                RfcUTF8ToSAPUC(
                    value.as_ptr(),
                    value.len() as u32,
                    buf.as_mut_ptr(),
                    &mut buf_len,
                    &mut res_len,
                    err_info.as_mut_ptr(),
                ),
                err_info
            );
        } else if is_rc_err!(rc) {
            return Err(err_info);
        }
        buf.set_len(res_len as usize);
    }
    Ok(buf)
}

pub fn to_string_truncate(value: &[SAP_UC]) -> Result<String> {
    let uc_len = value
        .iter()
        .position(|&c| c == 0)
        .unwrap_or_else(|| value.len());
    to_string(value, uc_len as u32)
}

pub fn to_string(value: &[SAP_UC], size: u32) -> Result<String> {
    let mut err_info = RfcErrorInfo::new();
    let mut buf = Vec::with_capacity(size as usize + 1);
    let mut buf_len = buf.capacity() as u32;
    let mut res_len: u32 = 0;
    unsafe {
        let rc = RfcSAPUCToUTF8(
            value.as_ptr(),
            size,
            buf.as_mut_ptr(),
            &mut buf_len,
            &mut res_len,
            err_info.as_mut_ptr(),
        );
        if rc == sapnwrfc_sys::_RFC_RC::RFC_BUFFER_TOO_SMALL {
            buf.reserve_exact(buf_len as usize + 1);
            buf_len = buf.capacity() as u32;
            check_rc_ok!(
                RfcSAPUCToUTF8(
                    value.as_ptr(),
                    size,
                    buf.as_mut_ptr(),
                    &mut buf_len,
                    &mut res_len,
                    err_info.as_mut_ptr(),
                ),
                err_info
            );
        } else if is_rc_err!(rc) {
            return Err(err_info);
        }
        buf.set_len(res_len as usize);
    }
    Ok(String::from_utf8(buf)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sap_uc_roundtrip() {
        assert_eq!(to_string_truncate(&from_str("").unwrap()).unwrap(), "",);
        assert_eq!(
            to_string_truncate(&from_str("Test String").unwrap()).unwrap(),
            "Test String",
        );
    }
}
