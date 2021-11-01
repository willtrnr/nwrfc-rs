use crate::{
    error::{Result, RfcErrorInfo},
    macros::*,
    structure::RfcStructure,
};
use sapnwrfc_sys::{
    self, RfcAppendNewRow, RfcDeleteAllRows, RfcDeleteCurrentRow, RfcGetCurrentRow, RfcGetRowCount,
    RfcInsertNewRow, RfcMoveTo, RfcMoveToFirstRow, RfcMoveToLastRow,
};

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

    /// Get the number of rows in the table.
    pub fn row_count(&self) -> Result<usize> {
        let mut count = 0;
        unsafe {
            check_rc_ok!(RfcGetRowCount(self.table, &mut count));
        }
        Ok(count as usize)
    }

    /// Get the row at the given index.
    pub fn get_row<'row: 'func>(&'row mut self, index: usize) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveTo(self.table, index as u32));
        }
        self.current_row()
    }

    /// Get the first row.
    pub fn get_first_row<'row: 'func>(&'row mut self) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveToFirstRow(self.table));
        }
        self.current_row()
    }

    /// Get the first row.
    pub fn get_last_row<'row: 'func>(&'row mut self) -> Result<RfcStructure<'row>> {
        unsafe {
            check_rc_ok!(RfcMoveToLastRow(self.table));
        }
        self.current_row()
    }

    /// Append a new row and return it.
    pub fn append_row<'row: 'func>(&'row mut self) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcAppendNewRow(self.table, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.table, self.desc, handle))
    }

    /// Insert a new row at the given position and return it.
    pub fn insert_row<'row: 'func>(&'row mut self, index: usize) -> Result<RfcStructure<'row>> {
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            check_rc_ok!(
                RfcMoveTo(self.table, index as u32, err_info.as_mut_ptr()),
                err_info
            );
        }
        let handle = unsafe { RfcInsertNewRow(self.table, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.table, self.desc, handle))
    }

    /// Delete the row at the given index.
    pub fn delete_row(&mut self, index: usize) -> Result<()> {
        let mut err_info = RfcErrorInfo::new();
        unsafe {
            check_rc_ok!(
                RfcMoveTo(self.table, index as u32, err_info.as_mut_ptr()),
                err_info
            );
            check_rc_ok!(
                RfcDeleteCurrentRow(self.table, err_info.as_mut_ptr()),
                err_info
            );
        }
        Ok(())
    }

    /// Delete all the rows in the table.
    pub fn clear_rows(&mut self) -> Result<()> {
        unsafe {
            check_rc_ok!(RfcDeleteAllRows(self.table));
        }
        Ok(())
    }

    fn current_row(&self) -> Result<RfcStructure<'_>> {
        let mut err_info = RfcErrorInfo::new();
        let handle = unsafe { RfcGetCurrentRow(self.table, err_info.as_mut_ptr()) };
        if handle.is_null() {
            return Err(err_info);
        }
        Ok(RfcStructure::new(&self.table, self.desc, handle))
    }
}

unsafe impl Send for RfcTable<'_> {}
