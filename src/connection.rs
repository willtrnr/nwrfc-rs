use crate::{
    error::{Result, RfcErrorInfo},
    function::RfcFunction,
    macros::{check_rc_ok, is_rc_err},
    uc,
};
use libsapnwrfc_sys::{
    self, RfcCloseConnection, RfcCreateFunction, RfcGetFunctionDesc, RfcOpenConnection, RfcPing,
    SAP_UC,
};
use std::{collections::HashMap, ptr};

/// An SAP NW RFC connection.
#[derive(Debug)]
#[repr(transparent)]
pub struct RfcConnection {
    handle: libsapnwrfc_sys::RFC_CONNECTION_HANDLE,
}

impl RfcConnection {
    pub(crate) fn new(params: Vec<(Vec<SAP_UC>, Vec<SAP_UC>)>) -> Result<RfcConnection> {
        let conn_params: Vec<_> = params
            .iter()
            .map(|(k, v)| libsapnwrfc_sys::RFC_CONNECTION_PARAMETER {
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

    /// Open a connection to a destination specified in an `sapnwrfc.ini` file.
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
        let desc =
            unsafe { RfcGetFunctionDesc(self.handle, uc_name.as_ptr(), err_info.as_mut_ptr()) };
        if desc.is_null() {
            return Err(err_info);
        }
        let func = unsafe { RfcCreateFunction(desc, err_info.as_mut_ptr()) };
        if func.is_null() {
            return Err(err_info);
        }
        Ok(RfcFunction::new(&self.handle, func, desc))
    }
}

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

unsafe impl Send for RfcConnection {}

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
