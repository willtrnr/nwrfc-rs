use crate::{
    data_container::{macros::rfc_data_delegates, RfcDataContainer},
    error::{Result, RfcErrorInfo},
    macros::{assert_rc_ok, check_rc_ok},
    structure::RfcStructure,
    uc,
};
use sapnwrfc_sys::{
    self, RfcAppendNewRow, RfcDeleteAllRows, RfcDeleteCurrentRow, RfcGetCurrentRow,
    RfcGetFieldCount, RfcGetFieldDescByName, RfcGetRowCount, RfcGetRowType, RfcGetTypeName,
    RfcInsertNewRow, RfcMoveTo, RfcMoveToFirstRow, RfcMoveToLastRow, DATA_CONTAINER_HANDLE,
    RFC_ABAP_NAME, RFC_TABLE_HANDLE, RFC_TYPE_DESC_HANDLE,
};

/// An RFC table.
pub struct RfcTable<'data> {
    _container: &'data DATA_CONTAINER_HANDLE,
    handle: RFC_TABLE_HANDLE,
    desc: RFC_TYPE_DESC_HANDLE,
    data: RfcDataContainer,
}

impl<'data> RfcTable<'data> {
    pub(crate) fn new(
        container: &'data DATA_CONTAINER_HANDLE,
        handle: RFC_TABLE_HANDLE,
        desc: RFC_TYPE_DESC_HANDLE,
    ) -> Self {
        Self {
            _container: container,
            handle,
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

    fn current_row<'row: 'data>(&'row self) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcGetCurrentRow(self.handle, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        let desc = unsafe { RfcGetRowType(self.handle, err_info.as_mut_ptr()) };
        if desc.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.handle, handle, desc))
    }

    /// Get the number of rows in the table.
    pub fn row_count(&self) -> Result<u32> {
        let mut count = 0;
        unsafe {
            check_rc_ok!(RfcGetRowCount(self.handle, &mut count));
        }
        Ok(count)
    }

    /// Get the row at the given index.
    pub fn get_row<'row: 'data>(&'row self, index: u32) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveTo(self.handle, index as u32));
        }
        self.current_row()
    }

    /// Get the first row.
    pub fn get_first_row<'row: 'data>(&'row self) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveToFirstRow(self.handle));
        }
        self.current_row()
    }

    /// Get the last row.
    pub fn get_last_row<'row: 'data>(&'row self) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveToLastRow(self.handle));
        }
        self.current_row()
    }

    /// Append a new row and return it.
    pub fn append_row<'row: 'data>(&'row mut self) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcAppendNewRow(self.handle, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.handle, handle, self.desc))
    }

    /// Insert a new row at the given position and return it.
    pub fn insert_row<'row: 'data>(&'row mut self, index: u32) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            check_rc_ok!(
                RfcMoveTo(self.handle, index, err_info.as_mut_ptr()),
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
    pub fn delete_row(&mut self, index: u32) -> Result<()> {
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            check_rc_ok!(
                RfcMoveTo(self.handle, index, err_info.as_mut_ptr()),
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

    rfc_data_delegates!(self.data, |name, desc| {
        unsafe {
            check_rc_ok!(RfcGetFieldDescByName(self.desc, name.as_ptr(), &mut desc));
        }
    });
}

unsafe impl Send for RfcTable<'_> {}
