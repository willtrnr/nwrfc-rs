use std::{collections::HashMap, ptr};

use crate::{
    bindings::{
        self, RfcCloseConnection, RfcCreateFunction, RfcDestroyFunction, RfcDestroyFunctionDesc,
        RfcGetFunctionDesc, RfcGetInt, RfcGetParameterDescByName, RfcGetString, RfcGetStringLength,
        RfcInvoke, RfcOpenConnection, RfcPing, RfcSetInt, RfcSetString, SAP_UC,
    },
    error::{Result, RfcErrorInfo},
    macros::*,
};

pub type SAPUCStr = widestring::UCStr<crate::bindings::SAP_UC>;
pub type SAPUCString = widestring::UCString<crate::bindings::SAP_UC>;

#[derive(Debug)]
pub struct RfcConnection {
    handle: bindings::RFC_CONNECTION_HANDLE,
}

impl RfcConnection {
    pub(crate) fn new(params: Vec<(Vec<SAP_UC>, Vec<SAP_UC>)>) -> Result<RfcConnection> {
        let conn_params: Vec<_> = params
            .iter()
            .map(|(k, v)| bindings::RFC_CONNECTION_PARAMETER {
                name: k.as_ptr(),
                value: v.as_ptr(),
            })
            .collect();

        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe {
            RfcOpenConnection(
                conn_params.as_ptr(),
                conn_params.len() as u32,
                &mut err_info,
            )
        };
        if handle.is_null() {
            return Err(err_info);
        }

        Ok(Self { handle })
    }

    pub fn builder() -> RfcConnectionBuilder {
        RfcConnectionBuilder::default()
    }

    pub fn for_dest(name: &str) -> Result<RfcConnection> {
        Self::new(vec![(to_sap_str!("dest"), to_sap_str!(name))])
    }

    pub fn ping(&self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcPing(self.handle));
        }
        Ok(())
    }

    pub fn get_function<'conn>(&'conn self, name: &str) -> Result<RfcFunction<'conn>> {
        let mut err_info = RfcErrorInfo::new();

        let desc_handle = unsafe {
            let c_name = to_sap_str!(name);
            RfcGetFunctionDesc(self.handle, c_name.as_ptr(), &mut err_info)
        };
        if desc_handle.is_null() {
            return Err(err_info);
        }

        let func_handle = unsafe { RfcCreateFunction(desc_handle, &mut err_info) };
        if func_handle.is_null() {
            return Err(err_info);
        }

        Ok(RfcFunction::new(&self.handle, desc_handle, func_handle))
    }
}

impl Drop for RfcConnection {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            let mut err_info = RfcErrorInfo::new();
            unsafe {
                if is_rc_err!(RfcCloseConnection(self.handle, &mut err_info)) {
                    log::warn!("Connection close failed: {}", err_info);
                }
            }
            self.handle = ptr::null_mut();
        }
    }
}

#[derive(Clone, Debug)]
pub struct RfcConnectionBuilder {
    params: HashMap<String, String>,
}

impl RfcConnectionBuilder {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn for_dest(name: &str) -> RfcConnectionBuilder {
        let mut params = HashMap::with_capacity(1);
        params.insert("dest".to_owned(), name.to_owned());
        Self { params }
    }

    pub fn set_param<T>(mut self, key: &str, value: T) -> Self
    where
        T: ToString,
    {
        self.params.insert(key.to_owned(), value.to_string());
        self
    }

    pub fn build(self) -> Result<RfcConnection> {
        let params: Vec<_> = self
            .params
            .into_iter()
            .map(|(k, v)| (to_sap_str!(k), to_sap_str!(v)))
            .collect();
        RfcConnection::new(params)
    }
}

impl Default for RfcConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RfcFunction<'conn> {
    conn_handle: &'conn bindings::RFC_CONNECTION_HANDLE,
    desc_handle: bindings::RFC_FUNCTION_DESC_HANDLE,
    func_handle: bindings::RFC_FUNCTION_HANDLE,
}

impl<'conn> RfcFunction<'conn> {
    pub(crate) fn new(
        conn_handle: &'conn bindings::RFC_CONNECTION_HANDLE,
        desc_handle: bindings::RFC_FUNCTION_DESC_HANDLE,
        func_handle: bindings::RFC_FUNCTION_HANDLE,
    ) -> Self {
        Self {
            conn_handle,
            desc_handle,
            func_handle,
        }
    }

    pub fn get_parameter<'param: 'conn>(&'param self, name: &str) -> Result<RfcParameter<'param>> {
        let mut param_desc = bindings::RFC_PARAMETER_DESC::default();
        unsafe {
            let c_name = to_sap_str!(name);
            check_rc_ok!(RfcGetParameterDescByName(
                self.desc_handle,
                c_name.as_ptr(),
                &mut param_desc
            ));
        }
        Ok(RfcParameter::new(&self.func_handle, param_desc))
    }

    pub fn invoke(&self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcInvoke(*self.conn_handle, self.func_handle));
        }
        Ok(())
    }
}

impl<'conn> Drop for RfcFunction<'conn> {
    fn drop(&mut self) {
        let mut err_info = RfcErrorInfo::new();
        if !self.func_handle.is_null() {
            unsafe {
                if is_rc_err!(RfcDestroyFunction(self.func_handle, &mut err_info)) {
                    log::warn!("Function discard failed: {}", err_info);
                }
            }
            self.func_handle = ptr::null_mut();
        }
        if !self.desc_handle.is_null() {
            unsafe {
                if is_rc_err!(RfcDestroyFunctionDesc(self.desc_handle, &mut err_info)) {
                    log::warn!("Function description discard failed: {}", err_info);
                }
            }
            self.desc_handle = ptr::null_mut();
        }
    }
}

pub struct RfcParameter<'cont> {
    handle: &'cont bindings::DATA_CONTAINER_HANDLE,
    desc: bindings::RFC_PARAMETER_DESC,
}

impl<'cont> RfcParameter<'cont> {
    pub(crate) fn new(
        handle: &'cont bindings::DATA_CONTAINER_HANDLE,
        desc: bindings::RFC_PARAMETER_DESC,
    ) -> Self {
        Self { handle, desc }
    }

    pub fn name(&self) -> String {
        from_sap_str!(&self.desc.name)
            .expect("Missing NUL terminator in parameter name")
            .to_string_lossy()
    }

    pub fn set_int(&mut self, value: i32) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcSetInt(*self.handle, self.desc.name.as_ptr(), value));
        }
        Ok(())
    }

    pub fn get_int(&self) -> Result<i32> {
        let mut value: i32 = 0;
        unsafe {
            check_rc_ok!(RfcGetInt(*self.handle, self.desc.name.as_ptr(), &mut value));
        }
        Ok(value)
    }

    pub fn set_string(&mut self, value: &str) -> Result<()> {
        let c_value = to_sap_str!(value);
        unsafe {
            check_rc_ok!(RfcSetString(
                *self.handle,
                self.desc.name.as_ptr(),
                c_value.as_ptr(),
                value.len() as u32
            ));
        }
        Ok(())
    }

    pub fn get_string(&self) -> Result<String> {
        unsafe {
            let mut str_len: u32 = 0;
            check_rc_ok!(RfcGetStringLength(
                *self.handle,
                self.desc.name.as_ptr(),
                &mut str_len
            ));
            str_len += 1;

            let mut actual_len: u32 = 0;
            let mut str_buf: Vec<SAP_UC> = Vec::with_capacity(str_len as usize);
            check_rc_ok!(RfcGetString(
                *self.handle,
                self.desc.name.as_ptr(),
                str_buf.as_mut_ptr(),
                str_len,
                &mut actual_len
            ));
            Ok(SAPUCStr::from_ptr(str_buf.as_ptr(), actual_len as usize)?.to_string()?)
        }
    }
}

#[cfg(feature = "chrono")]
use crate::bindings::{RfcGetDate, RfcSetDate};

#[cfg(feature = "chrono")]
impl<'cont> RfcParameter<'cont> {
    pub fn set_date<Tz>(&mut self, value: chrono::Date<Tz>) -> Result<()>
    where
        Tz: chrono::TimeZone,
        Tz::Offset: std::fmt::Display,
    {
        let mut c_value = to_sap_str!(&value.format("%Y%m%d").to_string());
        assert_eq!(c_value.len() as u32, bindings::SAP_DATE_LN);
        unsafe {
            check_rc_ok!(RfcSetDate(
                *self.handle,
                self.desc.name.as_ptr(),
                c_value.as_mut_ptr(),
            ));
        }
        Ok(())
    }

    pub fn get_date(&self) -> Result<chrono::Date<chrono::FixedOffset>> {
        let mut date_buf = Vec::with_capacity(bindings::SAP_DATE_LN as usize + 1);
        unsafe {
            check_rc_ok!(RfcGetDate(
                *self.handle,
                self.desc.name.as_ptr(),
                date_buf.as_mut_ptr(),
            ));
            let date_str = SAPUCStr::from_ptr(date_buf.as_ptr(), bindings::SAP_DATE_LN as usize)?
                .to_string()?;
            Ok(chrono::DateTime::parse_from_str(&date_str, "%Y%m%d")
                .map_err(|err| RfcErrorInfo::custom(&err.to_string()))?
                .date())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let conn = RfcConnection::builder()
            .set_param("dest", "TEST")
            .build()
            .unwrap();
        conn.ping().unwrap();

        let func = conn.get_function("SCP_STRING_ECHO").unwrap();

        func.get_parameter("IMP")
            .unwrap()
            .set_string("Test String")
            .unwrap();

        func.invoke().unwrap();

        assert_eq!(
            func.get_parameter("EXP").unwrap().get_string().unwrap(),
            "Test String"
        );
    }
}
