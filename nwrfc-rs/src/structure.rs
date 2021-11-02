use crate::{
    error::{Result, RfcErrorInfo},
    macros::{assert_rc_ok, check_rc_ok},
    uc,
};
use sapnwrfc_sys::{
    self, RfcGetFieldCount, RfcGetInt, RfcGetString, RfcGetStringLength, RfcGetTypeName, RfcSetInt,
    RfcSetString, RFC_ABAP_NAME, SAP_UC,
};

/// An RFC structure.
pub struct RfcStructure<'func> {
    _container: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
    handle: sapnwrfc_sys::RFC_STRUCTURE_HANDLE,
    desc: sapnwrfc_sys::RFC_TYPE_DESC_HANDLE,
}

impl<'func> RfcStructure<'func> {
    pub(crate) fn new(
        container: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
        handle: sapnwrfc_sys::RFC_STRUCTURE_HANDLE,
        desc: sapnwrfc_sys::RFC_TYPE_DESC_HANDLE,
    ) -> Self {
        Self {
            _container: container,
            handle,
            desc,
        }
    }

    /// Get the type name of the structure.
    pub fn name(&self) -> String {
        let mut err_info = RfcErrorInfo::new();
        let mut uc_name: RFC_ABAP_NAME = Default::default();
        assert_rc_ok!(
            unsafe { RfcGetTypeName(self.desc, uc_name.as_mut_ptr(), err_info.as_mut_ptr()) },
            "Unexpected failure from RfcGetTypeName"
        );
        uc::to_string_truncate(&uc_name).expect("Unexpected type name decoding error")
    }

    /// Get the number of fields in the structure.
    pub fn field_count(&self) -> usize {
        let mut err_info = RfcErrorInfo::new();
        let mut count = 0;
        assert_rc_ok!(
            unsafe { RfcGetFieldCount(self.desc, &mut count, err_info.as_mut_ptr()) },
            "Unexpected failure from RfcGetFieldCount"
        );
        count as usize
    }

    /// Set the field with the given name to a numeric value.
    pub fn set_int(&mut self, name: &str, value: i32) -> Result<()> {
        let uc_name = uc::from_str(name)?;
        unsafe {
            check_rc_ok!(RfcSetInt(self.handle, uc_name.as_ptr(), value));
        }
        Ok(())
    }

    /// Get the integer value of the field with the given name.
    pub fn get_int(&self, name: &str) -> Result<i32> {
        let uc_name = uc::from_str(name)?;
        let mut value: i32 = 0;
        unsafe {
            check_rc_ok!(RfcGetInt(self.handle, uc_name.as_ptr(), &mut value));
        }
        Ok(value)
    }

    /// Set the parameter to a string value. Only valid for EXPORT parameters.
    pub fn set_string(&mut self, name: &str, value: &str) -> Result<()> {
        let uc_name = uc::from_str(name)?;
        let uc_value = uc::from_str(value)?;
        unsafe {
            check_rc_ok!(RfcSetString(
                self.handle,
                uc_name.as_ptr(),
                uc_value.as_ptr(),
                uc_value.len() as u32
            ));
        }
        Ok(())
    }

    /// Get the parameter as a string value. Only valid for IMPORT and EXPORT parameters.
    pub fn get_string(&self, name: &str) -> Result<String> {
        let uc_name = uc::from_str(name)?;
        let mut str_len: u32 = 0;
        unsafe {
            check_rc_ok!(RfcGetStringLength(
                self.handle,
                uc_name.as_ptr(),
                &mut str_len
            ));
        }
        str_len += 1;

        let mut res_len: u32 = 0;
        let mut str_buf: Vec<SAP_UC> = Vec::with_capacity(str_len as usize);
        unsafe {
            check_rc_ok!(RfcGetString(
                self.handle,
                uc_name.as_ptr(),
                str_buf.as_mut_ptr(),
                str_len,
                &mut res_len
            ));
        }
        uc::to_string(&str_buf, res_len)
    }
}

unsafe impl Send for RfcStructure<'_> {}
