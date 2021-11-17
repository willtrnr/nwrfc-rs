use crate::{
    error::{Result, RfcErrorInfo},
    macros::check_rc_ok,
    structure::RfcStructure,
    table::RfcTable,
    uc,
};
use sapnwrfc_sys::{
    RfcDescribeType, RfcGetChars, RfcGetInt, RfcGetString, RfcGetStringLength, RfcGetStructure,
    RfcGetTable, RfcSetChars, RfcSetInt, RfcSetString, DATA_CONTAINER_HANDLE, RFC_ABAP_NAME,
    RFC_STRUCTURE_HANDLE, RFC_TABLE_HANDLE,
};
use std::ptr;

#[derive(Debug)]
#[repr(transparent)]
pub struct RfcDataContainer {
    handle: DATA_CONTAINER_HANDLE,
}

impl RfcDataContainer {
    pub(crate) fn new(handle: DATA_CONTAINER_HANDLE) -> Self {
        Self { handle }
    }

    pub fn set_int(&mut self, name: &RFC_ABAP_NAME, value: i32) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcSetInt(self.handle, name.as_ptr(), value));
        }
        Ok(())
    }

    pub fn get_int(&self, name: &RFC_ABAP_NAME) -> Result<i32> {
        let mut value: i32 = 0;
        unsafe {
            check_rc_ok!(RfcGetInt(self.handle, name.as_ptr(), &mut value));
        }
        Ok(value)
    }

    pub fn set_chars(&mut self, name: &RFC_ABAP_NAME, value: &str) -> Result<()> {
        let uc_value = uc::from_str(value)?;
        unsafe {
            check_rc_ok!(RfcSetChars(
                self.handle,
                name.as_ptr(),
                uc_value.as_ptr(),
                uc_value.len() as u32
            ));
        }
        Ok(())
    }

    pub fn get_chars(&self, name: &RFC_ABAP_NAME, size: u32) -> Result<String> {
        let mut str_buf = Vec::with_capacity(size as usize);
        unsafe {
            check_rc_ok!(RfcGetChars(
                self.handle,
                name.as_ptr(),
                str_buf.as_mut_ptr(),
                size
            ));
        }
        uc::to_string(&str_buf, size)
    }

    pub fn set_string(&mut self, name: &RFC_ABAP_NAME, value: &str) -> Result<()> {
        let uc_value = uc::from_str(value)?;
        unsafe {
            check_rc_ok!(RfcSetString(
                self.handle,
                name.as_ptr(),
                uc_value.as_ptr(),
                uc_value.len() as u32
            ));
        }
        Ok(())
    }

    pub fn get_string(&self, name: &RFC_ABAP_NAME) -> Result<String> {
        let mut err_info = RfcErrorInfo::new();
        let mut str_len = 0;
        unsafe {
            check_rc_ok!(
                RfcGetStringLength(
                    self.handle,
                    name.as_ptr(),
                    &mut str_len,
                    err_info.as_mut_ptr()
                ),
                err_info
            );
        }
        let mut str_buf = Vec::with_capacity(str_len as usize + 1);
        unsafe {
            check_rc_ok!(
                RfcGetString(
                    self.handle,
                    name.as_ptr(),
                    str_buf.as_mut_ptr(),
                    str_buf.capacity() as u32,
                    &mut str_len,
                    err_info.as_mut_ptr()
                ),
                err_info
            );
        }
        uc::to_string(&str_buf, str_len)
    }

    pub fn get_structure<'param>(
        &'param self,
        name: &RFC_ABAP_NAME,
    ) -> Result<RfcStructure<'param>> {
        let mut err_info = RfcErrorInfo::new();
        let mut struc: RFC_STRUCTURE_HANDLE = ptr::null_mut();
        unsafe {
            check_rc_ok!(
                RfcGetStructure(
                    self.handle,
                    name.as_ptr(),
                    &mut struc,
                    err_info.as_mut_ptr()
                ),
                err_info
            );
        }
        let desc = unsafe { RfcDescribeType(struc, err_info.as_mut_ptr()) };
        if desc.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.handle, struc, desc))
    }

    pub fn get_table<'param>(&'param self, name: &RFC_ABAP_NAME) -> Result<RfcTable<'param>> {
        let mut err_info = RfcErrorInfo::new();
        let mut table: RFC_TABLE_HANDLE = ptr::null_mut();
        unsafe {
            check_rc_ok!(
                RfcGetTable(
                    self.handle,
                    name.as_ptr(),
                    &mut table,
                    err_info.as_mut_ptr()
                ),
                err_info
            );
        }
        let desc = unsafe { RfcDescribeType(table, err_info.as_mut_ptr()) };
        if desc.is_null() {
            return Err(err_info);
        }
        Ok(RfcTable::new(&self.handle, table, desc))
    }

    #[cfg(feature = "chrono")]
    pub fn set_date<Tz>(&mut self, name: &RFC_ABAP_NAME, value: chrono::Date<Tz>) -> Result<()>
    where
        Tz: chrono::TimeZone,
    {
        use chrono::Datelike;
        use sapnwrfc_sys::RfcSetDate;

        let mut uc_value = uc::from_str(&format!(
            "{:04}{:02}{:02}",
            value.year(),
            value.month(),
            value.day(),
        ))?;
        unsafe {
            check_rc_ok!(RfcSetDate(
                self.handle,
                name.as_ptr(),
                uc_value.as_mut_ptr()
            ));
        }
        Ok(())
    }

    #[cfg(feature = "chrono")]
    pub fn get_date(&self, name: &RFC_ABAP_NAME) -> Result<chrono::Date<chrono::FixedOffset>> {
        use sapnwrfc_sys::{RfcGetDate, SAP_DATE};

        let mut date_buf: SAP_DATE = Default::default();
        unsafe {
            check_rc_ok!(RfcGetDate(
                self.handle,
                name.as_ptr(),
                date_buf.as_mut_ptr()
            ));
        }
        let date_str = uc::to_string(&date_buf, sapnwrfc_sys::SAP_DATE_LN)?;
        Ok(chrono::DateTime::parse_from_str(&date_str, "%Y%m%d")
            .map_err(|err| RfcErrorInfo::custom(&err.to_string()))?
            .date())
    }
}

unsafe impl Send for RfcDataContainer {}

#[allow(clippy::single_component_path_imports)]
pub mod macros {
    macro_rules! rfc_data_delegates {
        ($self:ident.$data:ident , | $name:ident , $desc:ident | { $($tt:tt)* }) => {
            pub fn set_int(&mut $self, name: &str, value: i32) -> crate::error::Result<()> {
                $self.$data.set_int(&crate::uc::from_str_to_abap_name(name)?, value)
            }

            pub fn get_int(&$self, name: &str) -> crate::error::Result<i32> {
                $self.$data.get_int(&crate::uc::from_str_to_abap_name(name)?)
            }

            pub fn set_chars(&mut $self, name: &str, value: &str) -> crate::error::Result<()> {
                $self.$data.set_chars(&crate::uc::from_str_to_abap_name(name)?, value)
            }

            pub fn get_chars(&$self, name: &str) -> crate::error::Result<String> {
                let $name = &crate::uc::from_str_to_abap_name(name)?;
                let mut $desc = Default::default();
                $($tt)*
                $self.$data.get_chars(&$name, $desc.ucLength / 2)
            }

            pub fn set_string(&mut $self, name: &str, value: &str) -> crate::error::Result<()> {
                $self.$data.set_string(&crate::uc::from_str_to_abap_name(name)?, value)
            }

            pub fn get_string(&$self, name: &str) -> crate::error::Result<String> {
                $self.$data.get_string(&crate::uc::from_str_to_abap_name(name)?)
            }

            pub fn get_structure<'param>(
                &'param $self,
                name: &str
            ) -> crate::error::Result<crate::structure::RfcStructure<'param>> {
                $self.$data.get_structure(&crate::uc::from_str_to_abap_name(name)?)
            }

            pub fn get_table<'param>(
                &'param $self,
                name: &str
            ) -> crate::error::Result<crate::table::RfcTable<'param>> {
                $self.$data.get_table(&crate::uc::from_str_to_abap_name(name)?)
            }

            #[cfg(feature = "chrono")]
            pub fn set_date<Tz>(&mut $self, name: &str, value: chrono::Date<Tz>) -> crate::error::Result<()>
            where
                Tz: chrono::TimeZone,
                Tz::Offset: std::fmt::Display,
            {
                $self.$data.set_date(&crate::uc::from_str_to_abap_name(name)?, value)
            }

            #[cfg(feature = "chrono")]
            pub fn get_date(&$self, name: &str) -> crate::error::Result<chrono::Date<chrono::FixedOffset>> {
                $self.$data.get_date(&crate::uc::from_str_to_abap_name(name)?)
            }
        };
    }

    pub(crate) use rfc_data_delegates;
}
