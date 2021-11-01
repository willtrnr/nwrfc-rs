use crate::{
    error::{Result, RfcErrorInfo},
    macros::*,
    uc,
};
use sapnwrfc_sys::{
    self, RfcAppendNewRow, RfcCloseConnection, RfcCreateFunction, RfcDestroyFunction,
    RfcDestroyFunctionDesc, RfcGetCurrentRow, RfcGetFunctionDesc, RfcGetInt,
    RfcGetParameterDescByName, RfcGetRowCount, RfcGetString, RfcGetStringLength, RfcGetStructure,
    RfcGetTable, RfcInvoke, RfcOpenConnection, RfcPing, RfcSetInt, RfcSetString, SAP_UC,
};
use std::{collections::HashMap, ptr};

/// An SAP NW RFC connection.
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
        let handle = unsafe {
            RfcOpenConnection(
                conn_params.as_ptr(),
                conn_params.len() as u32,
                err_info.as_mut_ptr(),
            )
        };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(Self { handle })
    }

    /// Get an empty connection builder to provide parameters for connecting.
    pub fn builder() -> RfcConnectionBuilder {
        RfcConnectionBuilder::default()
    }

    /// Short way to open a connection to a destination specified in an `sapnwrfc.ini` file.
    ///
    /// Equivalent to only setting the `dest` parameter in a connection builder.
    pub fn for_dest(name: &str) -> Result<RfcConnection> {
        Self::new(vec![(uc::from_str("dest")?, uc::from_str(name)?)])
    }

    /// Check if the connection is alive by sending an RFC ping.
    pub fn ping(&self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcPing(self.handle));
        }
        Ok(())
    }

    /// Get a remote enabled function module by name.
    pub fn get_function<'conn>(&'conn self, name: &str) -> Result<RfcFunction<'conn>> {
        let uc_name = uc::from_str(name)?;

        let mut err_info = RfcErrorInfo::new();
        let desc_handle =
            unsafe { RfcGetFunctionDesc(self.handle, uc_name.as_ptr(), err_info.as_mut_ptr()) };
        if desc_handle.is_null() {
            return Err(err_info);
        }
        let func_handle = unsafe { RfcCreateFunction(desc_handle, err_info.as_mut_ptr()) };
        if func_handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcFunction::new(&self.handle, desc_handle, func_handle))
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

/// An RFC connection builder to prepare parameters for opening the connection.
#[derive(Clone, Debug)]
pub struct RfcConnectionBuilder {
    params: HashMap<String, String>,
}

impl RfcConnectionBuilder {
    /// Get a new, empty, builder.
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Set a parameter to a given value.
    ///
    /// Note that all RFC connection parameters are represented as string internally
    /// so setting a value to `0` or `"0"` for instance is equivalent.
    pub fn set_param<T>(mut self, key: &str, value: T) -> Self
    where
        T: ToString,
    {
        self.params.insert(key.to_owned(), value.to_string());
        self
    }

    /// Consume the builder and try connecting with the set parameters.
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

/// A remote enabled function module.
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

    /// Get an IMPORT, EXPORT or TABLE parameter by name.
    pub fn get_parameter<'param: 'conn>(&'param self, name: &str) -> Result<RfcParameter<'param>> {
        let uc_name = uc::from_str(name)?;

        let mut desc = sapnwrfc_sys::RFC_PARAMETER_DESC::default();
        unsafe {
            check_rc_ok!(RfcGetParameterDescByName(
                self.desc,
                uc_name.as_ptr(),
                &mut desc
            ));
        }
        Ok(RfcParameter::new(&self.func, desc))
    }

    /// Get an IMPORT or EXPORT structure parameter.
    pub fn get_structure<'param: 'conn>(&'param self, name: &str) -> Result<RfcStructure<'param>> {
        self.get_parameter(name).and_then(|p| p.as_structure())
    }

    /// Get a TABLE parameter by name.
    pub fn get_table<'param: 'conn>(&'param self, name: &str) -> Result<RfcTable<'param>> {
        self.get_parameter(name).and_then(|p| p.as_table())
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
                if is_rc_err!(rc) && rc != sapnwrfc_sys::_RFC_RC::RFC_ILLEGAL_STATE {
                    log::warn!("Function description discard failed: {}", err_info);
                }
            }
            self.desc = ptr::null_mut();
        }
    }
}

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
            .map_err(|err| RfcErrorInfo::custom(&err.to_string()))?
            .date())
    }
}

pub struct RfcStructure<'func> {
    _handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
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
            _handle: handle,
            desc,
            struc,
        }
    }
}

unsafe impl Send for RfcStructure<'_> {}

pub struct RfcTable<'func> {
    _handle: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
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
            _handle: handle,
            desc,
            table,
        }
    }

    fn row_struct<'row: 'func>(
        &'row self,
        handle: sapnwrfc_sys::RFC_STRUCTURE_HANDLE,
    ) -> RfcStructure<'row> {
        RfcStructure::new(&self.table, self.desc, handle)
    }

    /// Get the number of rows in the table.
    pub fn row_count(&self) -> Result<usize> {
        let mut count = 0;
        unsafe {
            check_rc_ok!(RfcGetRowCount(self.table, &mut count));
        }
        Ok(count as usize)
    }

    /// Ge the current row structue.
    pub fn current_row<'row: 'func>(&'row self) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcGetCurrentRow(self.table, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(self.row_struct(handle))
    }

    /// Add and return a new row to the table.
    pub fn add_row<'row: 'func>(&'row mut self) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcAppendNewRow(self.table, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(self.row_struct(handle))
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
