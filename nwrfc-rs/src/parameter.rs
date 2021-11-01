use crate::{error::Result, macros::*, structure::RfcStructure, table::RfcTable, uc};
use sapnwrfc_sys::{
    self, RfcGetInt, RfcGetString, RfcGetStringLength, RfcGetStructure, RfcGetTable, RfcSetInt,
    RfcSetString, SAP_UC,
};
use std::ptr;

/// An RFC function parameter.
pub struct RfcParameter<'func> {
    handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
    desc: sapnwrfc_sys::RFC_PARAMETER_DESC,
}

impl<'func> RfcParameter<'func> {
    pub(crate) fn new(
        handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
        desc: sapnwrfc_sys::RFC_PARAMETER_DESC,
    ) -> Self {
        Self { handle, desc }
    }

    /// Get the name of the parameter.
    pub fn name(&self) -> String {
        uc::to_string_truncate(&self.desc.name).expect("Invalid SAP_UC name string")
    }

    /// Set the parameter to a numeric value. Only valid for EXPORT parameters.
    pub fn set_int(&mut self, value: i32) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcSetInt(*self.handle, self.desc.name.as_ptr(), value));
        }
        Ok(())
    }

    /// Get the parameter as an integer value. Only valid for IMPORT and EXPORT parameters.
    pub fn get_int(&self) -> Result<i32> {
        let mut value: i32 = 0;
        unsafe {
            check_rc_ok!(RfcGetInt(*self.handle, self.desc.name.as_ptr(), &mut value));
        }
        Ok(value)
    }

    /// Set the parameter to a string value. Only valid for EXPORT parameters.
    pub fn set_string(&mut self, value: &str) -> Result<()> {
        let uc_value = uc::from_str(value)?;
        unsafe {
            check_rc_ok!(RfcSetString(
                *self.handle,
                self.desc.name.as_ptr(),
                uc_value.as_ptr(),
                value.len() as u32
            ));
        }
        Ok(())
    }

    /// Get the parameter as a string value. Only valid for IMPORT and EXPORT parameters.
    pub fn get_string(&self) -> Result<String> {
        let mut str_len: u32 = 0;
        unsafe {
            check_rc_ok!(RfcGetStringLength(
                *self.handle,
                self.desc.name.as_ptr(),
                &mut str_len
            ));
        }
        str_len += 1;

        let mut res_len: u32 = 0;
        let mut str_buf: Vec<SAP_UC> = Vec::with_capacity(str_len as usize);
        unsafe {
            check_rc_ok!(RfcGetString(
                *self.handle,
                self.desc.name.as_ptr(),
                str_buf.as_mut_ptr(),
                str_len,
                &mut res_len
            ));
        }
        uc::to_string(&str_buf, res_len)
    }

    /// Use this parameter as a structure. Only valid for structure typed IMPORT or EXPORT
    /// parameters.
    pub fn as_structure(self) -> Result<RfcStructure<'func>> {
        let mut struc: sapnwrfc_sys::RFC_STRUCTURE_HANDLE = ptr::null_mut();
        unsafe {
            check_rc_ok!(RfcGetStructure(
                *self.handle,
                self.desc.name.as_ptr(),
                &mut struc
            ));
        }
        Ok(RfcStructure::new(self.handle, self.desc, struc))
    }

    /// Use this parameter as a structure. Only valid for TABLE parameters.
    pub fn as_table(self) -> Result<RfcTable<'func>> {
        let mut table: sapnwrfc_sys::RFC_TABLE_HANDLE = ptr::null_mut();
        unsafe {
            check_rc_ok!(RfcGetTable(
                *self.handle,
                self.desc.name.as_ptr(),
                &mut table
            ));
        }
        Ok(RfcTable::new(self.handle, self.desc, table))
    }
}

#[cfg(feature = "chrono")]
impl RfcParameter<'_> {
    pub fn set_date<Tz>(&mut self, value: chrono::Date<Tz>) -> Result<()>
    where
        Tz: chrono::TimeZone,
        Tz::Offset: std::fmt::Display,
    {
        use sapnwrfc_sys::RfcSetDate;

        let mut uc_value = uc::from_str(&value.format("%Y%m%d").to_string())?;
        unsafe {
            check_rc_ok!(RfcSetDate(
                *self.handle,
                self.desc.name.as_ptr(),
                uc_value.as_mut_ptr()
            ));
        }
        Ok(())
    }

    pub fn get_date(&self) -> Result<chrono::Date<chrono::FixedOffset>> {
        use sapnwrfc_sys::{RfcGetDate, SAP_DATE_LN};

        let mut date_buf: [SAP_UC; SAP_DATE_LN as usize] = [0; SAP_DATE_LN as usize];
        unsafe {
            check_rc_ok!(RfcGetDate(
                *self.handle,
                self.desc.name.as_ptr(),
                date_buf.as_mut_ptr()
            ));
        }
        let date_str = uc::to_string(&date_buf, sapnwrfc_sys::SAP_DATE_LN)?;
        Ok(chrono::DateTime::parse_from_str(&date_str, "%Y%m%d")
            .map_err(|err| crate::error::RfcErrorInfo::custom(&err.to_string()))?
            .date())
    }
}
