use crate::{
    data_container::{macros::rfc_data_delegates, RfcDataContainer},
    error::{Result, RfcErrorInfo},
    macros::{check_rc_ok, is_rc_err},
};
use sapnwrfc_sys::{
    self, RfcDestroyFunction, RfcDestroyFunctionDesc, RfcGetParameterDescByName, RfcInvoke,
    RFC_CONNECTION_HANDLE, RFC_FUNCTION_DESC_HANDLE, RFC_FUNCTION_HANDLE, _RFC_RC,
};

/// A remote enabled RFC function module.
#[derive(Debug)]
pub struct RfcFunction<'conn> {
    conn_handle: &'conn RFC_CONNECTION_HANDLE,
    handle: RFC_FUNCTION_HANDLE,
    desc: RFC_FUNCTION_DESC_HANDLE,
    data: RfcDataContainer,
}

impl<'conn> RfcFunction<'conn> {
    pub(crate) fn new(
        conn_handle: &'conn RFC_CONNECTION_HANDLE,
        handle: RFC_FUNCTION_HANDLE,
        desc: RFC_FUNCTION_DESC_HANDLE,
    ) -> Self {
        Self {
            conn_handle,
            handle,
            desc,
            data: RfcDataContainer::new(handle),
        }
    }

    pub fn invoke(&self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcInvoke(*self.conn_handle, self.handle));
        }
        Ok(())
    }

    rfc_data_delegates!(self.data, |name, desc| {
        unsafe {
            check_rc_ok!(RfcGetParameterDescByName(
                self.desc,
                name.as_ptr(),
                &mut desc
            ));
        }
        println!("{:?}", &desc);
    });
}

impl Drop for RfcFunction<'_> {
    fn drop(&mut self) {
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            if is_rc_err!(RfcDestroyFunction(self.handle, err_info.as_mut_ptr())) {
                log::warn!("Function destroy failed: {}", err_info);
            }

            let rc = RfcDestroyFunctionDesc(self.desc, err_info.as_mut_ptr());
            // Call to RfcDestroyFunctionDesc fails with RFC_ILLEGAL_STATE when the function
            // description is held in a cache. The can safely be silenced for this case.
            if is_rc_err!(rc) && rc != _RFC_RC::RFC_ILLEGAL_STATE {
                log::warn!("Function description destroy failed: {}", err_info);
            }
        }
    }
}

unsafe impl Send for RfcFunction<'_> {}
