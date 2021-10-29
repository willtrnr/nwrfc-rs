use crate::{
    error::{Result, RfcErrorInfo},
    macros::*,
    uc,
};
use sapnwrfc_sys::{
    self, RfcCloseConnection, RfcCreateFunction, RfcDestroyFunction, RfcDestroyFunctionDesc,
    RfcGetFunctionDesc, RfcGetInt, RfcGetParameterDescByName, RfcGetString, RfcGetStringLength,
    RfcGetStructure, RfcGetTable, RfcInvoke, RfcOpenConnection, RfcPing, RfcSetInt, RfcSetString,
    SAP_UC,
};
use std::{collections::HashMap, ptr};

#[derive(Debug)]
pub struct RfcConnection {
    handle: sapnwrfc_sys::RFC_CONNECTION_HANDLE,
}

impl RfcConnection {
    pub(crate) fn new(params: Vec<(Vec<SAP_UC>, Vec<SAP_UC>)>) -> Result<RfcConnection> {
        let conn_params: Vec<_> = params
            .iter()
            .map(|(k, v)| sapnwrfc_sys::RFC_CONNECTION_PARAMETER {
                name: k.as_ptr(),
                value: v.as_ptr(),
            })
            .collect();

        let mut err_info = RfcErrorInfo::new();
        unsafe {
            let handle = RfcOpenConnection(
                conn_params.as_ptr(),
                conn_params.len() as u32,
                err_info.as_mut_ptr(),
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
        Self::new(vec![(uc::from_str("dest")?, uc::from_str(name)?)])
    }

    pub fn ping(&self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcPing(self.handle));
        }
        Ok(())
    }

    pub fn get_function<'conn>(&'conn self, name: &str) -> Result<RfcFunction<'conn>> {
        let uc_name = uc::from_str(name)?;
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            let desc_handle =
                RfcGetFunctionDesc(self.handle, uc_name.as_ptr(), err_info.as_mut_ptr());
            if desc_handle.is_null() {
                return Err(err_info);
            }

            let func_handle = RfcCreateFunction(desc_handle, err_info.as_mut_ptr());
            if func_handle.is_null() {
                return Err(err_info);
            }

            Ok(RfcFunction::new(&self.handle, desc_handle, func_handle))
        }
    }
}

unsafe impl Send for RfcConnection {}

impl Drop for RfcConnection {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            let mut err_info = RfcErrorInfo::new();
            unsafe {
                if is_rc_err!(RfcCloseConnection(self.handle, err_info.as_mut_ptr())) {
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
            .map(|(k, v)| Ok((uc::from_str(&k)?, uc::from_str(&v)?)))
            .collect();
        RfcConnection::new(params?)
    }
}

impl Default for RfcConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RfcFunction<'conn> {
    conn: &'conn sapnwrfc_sys::RFC_CONNECTION_HANDLE,
    desc: sapnwrfc_sys::RFC_FUNCTION_DESC_HANDLE,
    func: sapnwrfc_sys::RFC_FUNCTION_HANDLE,
}

impl<'conn> RfcFunction<'conn> {
    pub(crate) fn new(
        conn: &'conn sapnwrfc_sys::RFC_CONNECTION_HANDLE,
        desc: sapnwrfc_sys::RFC_FUNCTION_DESC_HANDLE,
        func: sapnwrfc_sys::RFC_FUNCTION_HANDLE,
    ) -> Self {
        Self { conn, desc, func }
    }

    fn get_parameter_desc(&self, name: &str) -> Result<sapnwrfc_sys::RFC_PARAMETER_DESC> {
        let uc_name = uc::from_str(name)?;
        let mut desc = sapnwrfc_sys::RFC_PARAMETER_DESC::default();
        unsafe {
            check_rc_ok!(RfcGetParameterDescByName(
                self.desc,
                uc_name.as_ptr(),
                &mut desc
            ));
        }
        Ok(desc)
    }

    pub fn get_parameter<'param: 'conn>(&'param self, name: &str) -> Result<RfcParameter<'param>> {
        Ok(RfcParameter::new(
            &self.func,
            self.get_parameter_desc(name)?,
        ))
    }

    pub fn get_structure<'param: 'conn>(&'param self, name: &str) -> Result<RfcStructure<'param>> {
        let desc = self.get_parameter_desc(name)?;
        let mut struc: sapnwrfc_sys::RFC_STRUCTURE_HANDLE = ptr::null_mut();
        unsafe {
            check_rc_ok!(RfcGetStructure(self.func, desc.name.as_ptr(), &mut struc));
        }
        Ok(RfcStructure::new(&self.func, desc, struc))
    }

    pub fn get_table<'param: 'conn>(&'param self, name: &str) -> Result<RfcTable<'param>> {
        let desc = self.get_parameter_desc(name)?;
        let mut table: sapnwrfc_sys::RFC_TABLE_HANDLE = ptr::null_mut();
        unsafe {
            check_rc_ok!(RfcGetTable(self.func, desc.name.as_ptr(), &mut table));
        }
        Ok(RfcTable::new(&self.func, desc, table))
    }

    pub fn invoke(&self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcInvoke(*self.conn, self.func));
        }
        Ok(())
    }
}

unsafe impl Send for RfcFunction<'_> {}

impl Drop for RfcFunction<'_> {
    fn drop(&mut self) {
        let mut err_info = RfcErrorInfo::new();
        if !self.func.is_null() {
            unsafe {
                if is_rc_err!(RfcDestroyFunction(self.func, err_info.as_mut_ptr())) {
                    log::warn!("Function discard failed: {}", err_info);
                }
            }
            self.func = ptr::null_mut();
        }
        if !self.desc.is_null() {
            unsafe {
                let rc = RfcDestroyFunctionDesc(self.desc, err_info.as_mut_ptr());
                // Call to RfcDestroyFunctionDesc fails with RFC_ILLEGAL_STATE when
                // the function description is held in the runtime cache. The error
                // can safely be silenced for this case.
                if is_rc_err!(rc) && rc != sapnwrfc_sys::_RFC_RC_RFC_ILLEGAL_STATE {
                    log::warn!("Function description discard failed: {}", err_info);
                }
            }
            self.desc = ptr::null_mut();
        }
    }
}

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

    pub fn name(&self) -> String {
        uc::to_string_truncate(&self.desc.name).expect("Invalid SAP_UC name string")
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
            uc::to_string(str_buf.as_ptr(), res_len)
        }
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
        use sapnwrfc_sys::RfcGetDate;

        let mut date_buf: [SAP_UC; sapnwrfc_sys::SAP_DATE_LN as usize];
        unsafe {
            check_rc_ok!(RfcGetDate(
                *self.handle,
                self.desc.name.as_ptr(),
                date_buf.as_mut_ptr()
            ));
            let date_str = uc::to_string(date_buf.as_ptr(), sapnwrfc_sys::SAP_DATE_LN)?;
            Ok(chrono::DateTime::parse_from_str(&date_str, "%Y%m%d")
                .map_err(|err| RfcErrorInfo::custom(&err.to_string()))?
                .date())
        }
    }
}

pub struct RfcStructure<'func> {
    handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
    desc: sapnwrfc_sys::RFC_PARAMETER_DESC,
    struc: sapnwrfc_sys::RFC_STRUCTURE_HANDLE,
}

impl<'func> RfcStructure<'func> {
    pub(crate) fn new(
        handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
        desc: sapnwrfc_sys::RFC_PARAMETER_DESC,
        struc: sapnwrfc_sys::RFC_STRUCTURE_HANDLE,
    ) -> Self {
        Self {
            handle,
            desc,
            struc,
        }
    }
}

unsafe impl Send for RfcStructure<'_> {}

pub struct RfcTable<'func> {
    handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
    desc: sapnwrfc_sys::RFC_PARAMETER_DESC,
    table: sapnwrfc_sys::RFC_TABLE_HANDLE,
}

impl<'func> RfcTable<'func> {
    pub(crate) fn new(
        handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
        desc: sapnwrfc_sys::RFC_PARAMETER_DESC,
        table: sapnwrfc_sys::RFC_TABLE_HANDLE,
    ) -> Self {
        Self {
            handle,
            desc,
            table,
        }
    }
}

unsafe impl Send for RfcTable<'_> {}

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

    #[test]
    fn negative_smoke_test() {
        RfcConnection::builder()
            .set_param("dest", "INVALID")
            .build()
            .unwrap_err();
    }
}
