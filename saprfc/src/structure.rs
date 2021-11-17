use crate::{
    data_container::{macros::rfc_data_delegates, RfcDataContainer},
    error::RfcErrorInfo,
    macros::{assert_rc_ok, check_rc_ok},
    uc,
};
use sapnwrfc_sys::{
    self, RfcGetFieldCount, RfcGetFieldDescByName, RfcGetTypeName, DATA_CONTAINER_HANDLE,
    RFC_ABAP_NAME, RFC_STRUCTURE_HANDLE, RFC_TYPE_DESC_HANDLE,
};

/// An RFC structure.
pub struct RfcStructure<'data> {
    _container: &'data DATA_CONTAINER_HANDLE,
    desc: RFC_TYPE_DESC_HANDLE,
    data: RfcDataContainer,
}

impl<'data> RfcStructure<'data> {
    pub(crate) fn new(
        container: &'data DATA_CONTAINER_HANDLE,
        handle: RFC_STRUCTURE_HANDLE,
        desc: RFC_TYPE_DESC_HANDLE,
    ) -> Self {
        Self {
            _container: container,
            desc,
            data: RfcDataContainer::new(handle),
        }
    }

    pub fn name(&self) -> String {
        let mut err_info = RfcErrorInfo::new();
        let mut uc_name: RFC_ABAP_NAME = Default::default();
        unsafe {
            assert_rc_ok!(
                RfcGetTypeName(self.desc, uc_name.as_mut_ptr(), err_info.as_mut_ptr()),
                "Unexpected failure with RfcGetTypeName"
            );
        }
        uc::to_string_truncate(&uc_name).expect("Unexpected string decode failure with type name")
    }

    pub fn field_count(&self) -> u32 {
        let mut err_info = RfcErrorInfo::new();
        let mut count = 0;
        unsafe {
            assert_rc_ok!(
                RfcGetFieldCount(self.desc, &mut count, err_info.as_mut_ptr()),
                "Unexpected failure with RfcGetFieldCount"
            );
        }
        count
    }

    rfc_data_delegates!(self.data, |name, desc| {
        unsafe {
            check_rc_ok!(RfcGetFieldDescByName(self.desc, name.as_ptr(), &mut desc));
        }
    });
}

unsafe impl Send for RfcStructure<'_> {}
