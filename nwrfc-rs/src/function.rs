use crate::{
    error::{Result, RfcErrorInfo},
    macros::{check_rc_ok, is_rc_err},
    parameter::RfcParameter,
    structure::RfcStructure,
    table::RfcTable,
    uc,
};
use sapnwrfc_sys::{
    self, RfcDestroyFunction, RfcDestroyFunctionDesc, RfcGetParameterDescByName, RfcInvoke,
};
use std::ptr;

/// A remote enabled RFC function module.
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
