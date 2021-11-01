use sapnwrfc_sys;

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
