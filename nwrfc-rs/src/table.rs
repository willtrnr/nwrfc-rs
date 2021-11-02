use crate::{
    error::{Result, RfcErrorInfo},
    macros::{assert_rc_ok, check_rc_ok},
    structure::RfcStructure,
    uc,
};
use sapnwrfc_sys::{
    self, RfcAppendNewRow, RfcDeleteAllRows, RfcDeleteCurrentRow, RfcGetCurrentRow,
    RfcGetFieldCount, RfcGetRowCount, RfcGetTypeName, RfcInsertNewRow, RfcMoveTo,
    RfcMoveToFirstRow, RfcMoveToLastRow, RFC_ABAP_NAME,
};

/// An RFC table.
pub struct RfcTable<'func> {
    _container: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
    handle: sapnwrfc_sys::RFC_TABLE_HANDLE,
    desc: sapnwrfc_sys::RFC_TYPE_DESC_HANDLE,
}

impl<'func> RfcTable<'func> {
    pub(crate) fn new(
        container: &'func sapnwrfc_sys::DATA_CONTAINER_HANDLE,
        handle: sapnwrfc_sys::RFC_TABLE_HANDLE,
        desc: sapnwrfc_sys::RFC_TYPE_DESC_HANDLE,
    ) -> Self {
        Self {
            _container: container,
            handle,
            desc,
        }
    }

    /// Get the type name of the table rows.
    pub fn name(&self) -> String {
        let mut err_info = RfcErrorInfo::new();
        let mut uc_name: RFC_ABAP_NAME = Default::default();
        assert_rc_ok!(
            unsafe { RfcGetTypeName(self.desc, uc_name.as_mut_ptr(), err_info.as_mut_ptr()) },
            "Unexpected failure from RfcGetTypeName"
        );
        uc::to_string_truncate(&uc_name).expect("Unexpected type name decoding error")
    }

    /// Get the number of fields per row.
    pub fn field_count(&self) -> usize {
        let mut err_info = RfcErrorInfo::new();
        let mut count = 0;
        let rc = unsafe { RfcGetFieldCount(self.desc, &mut count, err_info.as_mut_ptr()) };
        assert_rc_ok!(rc, "Unexpected failure from RfcGetFieldCount");
        count as usize
    }

    /// Get the number of rows in the table.
    pub fn row_count(&self) -> Result<usize> {
        let mut count = 0;
        unsafe {
            check_rc_ok!(RfcGetRowCount(self.handle, &mut count));
        }
        Ok(count as usize)
    }

    /// Get the row at the given index.
    pub fn get_row<'row: 'func>(&'row mut self, index: usize) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveTo(self.handle, index as u32));
        }
        self.current_row()
    }

    /// Get the first row.
    pub fn get_first_row<'row: 'func>(&'row mut self) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveToFirstRow(self.handle));
        }
        self.current_row()
    }

    /// Get the first row.
    pub fn get_last_row<'row: 'func>(&'row mut self) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveToLastRow(self.handle));
        }
        self.current_row()
    }

    /// Append a new row and return it.
    pub fn append_row<'row: 'func>(&'row mut self) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcAppendNewRow(self.handle, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.handle, handle, self.desc))
    }

    /// Insert a new row at the given position and return it.
    pub fn insert_row<'row: 'func>(&'row mut self, index: usize) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            check_rc_ok!(
                RfcMoveTo(self.handle, index as u32, err_info.as_mut_ptr()),
                err_info
            );
        }
        let handle = unsafe { RfcInsertNewRow(self.handle, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.handle, handle, self.desc))
    }

    /// Delete the row at the given index.
    pub fn delete_row(&mut self, index: usize) -> Result<()> {
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            check_rc_ok!(
                RfcMoveTo(self.handle, index as u32, err_info.as_mut_ptr()),
                err_info
            );
            check_rc_ok!(
                RfcDeleteCurrentRow(self.handle, err_info.as_mut_ptr()),
                err_info
            );
        }
        Ok(())
    }

    /// Delete all the rows in the table.
    pub fn clear_rows(&mut self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcDeleteAllRows(self.handle));
        }
        Ok(())
    }

    fn current_row(&self) -> Result<RfcStructure<'_>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcGetCurrentRow(self.handle, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.handle, handle, self.desc))
    }
}

unsafe impl Send for RfcTable<'_> {}
