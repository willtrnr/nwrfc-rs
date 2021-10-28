use std::{collections::HashMap, ptr};

use crate::{
    bindings::{
        self, RfcCloseConnection, RfcCreateFunction, RfcDestroyFunction, RfcDestroyFunctionDesc,
        RfcGetFunctionDesc, RfcGetInt, RfcGetParameterDescByName, RfcGetString, RfcGetStringLength,
        RfcGetTable, RfcInvoke, RfcOpenConnection, RfcPing, RfcSAPUCToUTF8, RfcSetInt,
        RfcSetString, RfcUTF8ToSAPUC, SAP_UC,
    },
    error::{Result, RfcErrorInfo},
};

macro_rules! is_rc_ok {
    ($expr:expr) => {
        $expr == crate::bindings::_RFC_RC_RFC_OK
    };
}

macro_rules! is_rc_err {
    ($expr:expr) => {
        !(is_rc_ok!($expr))
    };
}

macro_rules! check_rc_ok {
    ($expr:expr , $error:ident) => {
        if is_rc_err!($expr) {
            return Err($error);
        }
    };
    ($fn:ident ( $($args:expr),+ ) ) => {
        let mut err_info = crate::error::RfcErrorInfo::new();
        check_rc_ok!($fn($($args),*, &mut err_info), err_info);
    };
    ($fn:ident ( ) ) => {
        let mut err_info = crate::error::RfcErrorInfo::new();
        check_rc_ok!($fn(&mut err_info), err_info);
    };
}

pub fn str_to_sap_uc_slice(value: &str, dest: &mut [SAP_UC]) -> Result<u32> {
    let mut dest_len: u32 = dest.len() as u32;
    let mut res_len: u32 = 0;
    unsafe {
        check_rc_ok!(RfcUTF8ToSAPUC(
            value.as_ptr(),
            value.len() as u32,
            dest.as_mut_ptr(),
            &mut dest_len,
            &mut res_len
        ));
    }
    Ok(res_len)
}

pub fn str_to_sap_uc(value: &str) -> Result<Vec<SAP_UC>> {
    let mut err_info = RfcErrorInfo::new();
    let mut buf = Vec::with_capacity(value.len() + 1);
    let mut buf_len: u32 = buf.capacity() as u32;
    let mut res_len: u32 = 0;
    unsafe {
        let rc = RfcUTF8ToSAPUC(
            value.as_ptr(),
            value.len() as u32,
            buf.as_mut_ptr(),
            &mut buf_len,
            &mut res_len,
            &mut err_info,
        );
        if rc == bindings::_RFC_RC_RFC_BUFFER_TOO_SMALL {
            buf.reserve_exact(buf_len as usize + 1);
            buf_len = buf.capacity() as u32;
            check_rc_ok!(
                RfcUTF8ToSAPUC(
                    value.as_ptr(),
                    value.len() as u32,
                    buf.as_mut_ptr(),
                    &mut buf_len,
                    &mut res_len,
                    &mut err_info,
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

pub fn str_from_sap_uc(value: &[SAP_UC]) -> Result<String> {
    let mut err_info = RfcErrorInfo::new();
    let mut str_buf = Vec::with_capacity(value.len() + 1);
    let mut buf_len = str_buf.capacity() as u32;
    let mut res_len: u32 = 0;
    unsafe {
        let rc = RfcSAPUCToUTF8(
            value.as_ptr(),
            value.len() as u32,
            str_buf.as_mut_ptr(),
            &mut buf_len,
            &mut res_len,
            &mut err_info,
        );
        if rc == bindings::_RFC_RC_RFC_BUFFER_TOO_SMALL {
            str_buf.reserve_exact(buf_len as usize + 1);
            buf_len = str_buf.capacity() as u32;
            check_rc_ok!(
                RfcSAPUCToUTF8(
                    value.as_ptr(),
                    value.len() as u32,
                    str_buf.as_mut_ptr(),
                    &mut buf_len,
                    &mut res_len,
                    &mut err_info,
                ),
                err_info
            );
        } else if is_rc_err!(rc) {
            return Err(err_info);
        }
        str_buf.set_len(res_len as usize);
    }
    Ok(String::from_utf8(str_buf)?)
}

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
        unsafe {
            let handle = RfcOpenConnection(
                conn_params.as_ptr(),
                conn_params.len() as u32,
                &mut err_info,
            );
            if handle.is_null() {
                return Err(err_info);
            }
            Ok(Self { handle })
        }
    }

    pub fn builder() -> RfcConnectionBuilder {
        RfcConnectionBuilder::default()
    }

    pub fn for_dest(name: &str) -> Result<RfcConnection> {
        Ok(Self::new(vec![(
            str_to_sap_uc("dest")?,
            str_to_sap_uc(name)?,
        )])?)
    }

    pub fn ping(&self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcPing(self.handle));
        }
        Ok(())
    }

    pub fn get_function<'conn>(&'conn self, name: &str) -> Result<RfcFunction<'conn>> {
        let uc_name = str_to_sap_uc(name)?;
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            let desc_handle = RfcGetFunctionDesc(self.handle, uc_name.as_ptr(), &mut err_info);
            if desc_handle.is_null() {
                return Err(err_info);
            }

            let func_handle = RfcCreateFunction(desc_handle, &mut err_info);
            if func_handle.is_null() {
                return Err(err_info);
            }

            Ok(RfcFunction::new(&self.handle, desc_handle, func_handle))
        }
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
        let params: Result<Vec<_>> = self
            .params
            .into_iter()
            .map(|(k, v)| Ok((str_to_sap_uc(&k)?, str_to_sap_uc(&v)?)))
            .collect();
        Ok(RfcConnection::new(params?)?)
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

    fn get_parameter_desc(&self, name: &str) -> Result<bindings::RFC_PARAMETER_DESC> {
        let uc_name = str_to_sap_uc(name)?;
        let mut desc = bindings::RFC_PARAMETER_DESC::default();
        unsafe {
            check_rc_ok!(RfcGetParameterDescByName(
                self.desc_handle,
                uc_name.as_ptr(),
                &mut desc
            ));
        }
        Ok(desc)
    }

    pub fn get_parameter<'param: 'conn>(&'param self, name: &str) -> Result<RfcParameter<'param>> {
        Ok(RfcParameter::new(
            &self.func_handle,
            self.get_parameter_desc(name)?,
        ))
    }

    pub fn get_table<'table: 'conn>(&'table self, name: &str) -> Result<RfcTable<'table>> {
        let desc = self.get_parameter_desc(name)?;
        let mut table: bindings::RFC_TABLE_HANDLE = ptr::null_mut();
        unsafe {
            check_rc_ok!(RfcGetTable(
                self.func_handle,
                desc.name.as_ptr(),
                &mut table
            ));
        }
        Ok(RfcTable::new(&self.func_handle, desc, table))
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
                let rc = RfcDestroyFunctionDesc(self.desc_handle, &mut err_info);
                // Call to RfcDestroyFunctionDesc fails with RFC_ILLEGAL_STATE when
                // the function description is held in the runtime cache. The error
                // can safely be silenced for this case.
                if is_rc_err!(rc) && rc != bindings::_RFC_RC_RFC_ILLEGAL_STATE {
                    log::warn!("Function description discard failed: {}", err_info);
                }
            }
            self.desc_handle = ptr::null_mut();
        }
    }
}

pub struct RfcParameter<'func> {
    handle: &'func bindings::DATA_CONTAINER_HANDLE,
    desc: bindings::RFC_PARAMETER_DESC,
}

impl<'func> RfcParameter<'func> {
    pub(crate) fn new(
        handle: &'func bindings::DATA_CONTAINER_HANDLE,
        desc: bindings::RFC_PARAMETER_DESC,
    ) -> Self {
        Self { handle, desc }
    }

    pub fn name(&self) -> String {
        str_from_sap_uc(&self.desc.name).expect("Invalid SAP_UC name string")
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
        let uc_value = str_to_sap_uc(value)?;
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

    pub fn get_string(&self) -> Result<String> {
        unsafe {
            let mut str_len: u32 = 0;
            check_rc_ok!(RfcGetStringLength(
                *self.handle,
                self.desc.name.as_ptr(),
                &mut str_len
            ));
            str_len += 1;

            let mut res_len: u32 = 0;
            let mut str_buf: Vec<SAP_UC> = Vec::with_capacity(str_len as usize);
            check_rc_ok!(RfcGetString(
                *self.handle,
                self.desc.name.as_ptr(),
                str_buf.as_mut_ptr(),
                str_len,
                &mut res_len
            ));
            str_buf.set_len(res_len as usize);
            str_from_sap_uc(&str_buf)
        }
    }
}

#[cfg(feature = "chrono")]
impl<'func> RfcParameter<'func> {
    pub fn set_date<Tz>(&mut self, value: chrono::Date<Tz>) -> Result<()>
    where
        Tz: chrono::TimeZone,
        Tz::Offset: std::fmt::Display,
    {
        use crate::bindings::RfcSetDate;
        let mut uc_value = str_to_sap_uc(&value.format("%Y%m%d").to_string())?;
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
        use crate::bindings::RfcGetDate;
        let mut date_buf = Vec::with_capacity(bindings::SAP_DATE_LN as usize);
        unsafe {
            check_rc_ok!(RfcGetDate(
                *self.handle,
                self.desc.name.as_ptr(),
                date_buf.as_mut_ptr()
            ));
            let date_str = str_from_sap_uc(&date_buf)?;
            Ok(chrono::DateTime::parse_from_str(&date_str, "%Y%m%d")
                .map_err(|err| RfcErrorInfo::custom(&err.to_string()))?
                .date())
        }
    }
}

pub struct RfcTable<'func> {
    handle: &'func bindings::DATA_CONTAINER_HANDLE,
    desc: bindings::RFC_PARAMETER_DESC,
    table: bindings::RFC_TABLE_HANDLE,
}

impl<'func> RfcTable<'func> {
    pub(crate) fn new(
        handle: &'func bindings::DATA_CONTAINER_HANDLE,
        desc: bindings::RFC_PARAMETER_DESC,
        table: bindings::RFC_TABLE_HANDLE,
    ) -> Self {
        Self {
            handle,
            desc,
            table,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sap_uc_roundtrip() {
        assert_eq!(str_from_sap_uc(&str_to_sap_uc("").unwrap()).unwrap(), "",);
        assert_eq!(
            str_from_sap_uc(&str_to_sap_uc("Test String").unwrap()).unwrap(),
            "Test String",
        );
    }

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

    #[test]
    fn negative_smoke_test() {
        RfcConnection::builder()
            .set_param("dest", "INVALID")
            .build()
            .unwrap_err();
    }
}
