// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::slice;

pub enum Error {}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Integer {
    inner: i16,
}

impl Integer {
    pub fn is_zero(self) -> bool {
        self.inner == 0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Long {
    inner: i32,
}

impl Long {
    pub fn is_zero(self) -> bool {
        self.inner == 0
    }
}
pub struct Boolean {
    inner: bool,
}

impl Boolean {
    pub fn r#true() -> Self {
        Boolean { inner: true }
    }

    pub fn r#false() -> Self {
        Boolean { inner: false }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum Variant {
    Boolean(Boolean),
    Integer(Integer),
    Long(Long),
}

#[repr(C)]
pub struct String {
    ptr: *mut u16,
    len: u32,
    capacity: u32,
}

impl Drop for String {
    fn drop(&mut self) {
        // SAFETY: values from Vec.into_raw_parts().
        let vec: Vec<u16> =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        drop(vec)
    }
}

impl String {
    pub fn to_rust_string_lossy(&self) -> std::string::String {
        let slice = self.as_slice();
        let len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
        let slice = &slice[..len];
        std::string::String::from_utf16_lossy(slice)
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe { slice::from_raw_parts(self.ptr, self.len as usize) }
    }

    #[unsafe(no_mangle)]
    /// # Safety
    /// The parameters must meet the requirements of `slice::from_raw_parts`.
    /// This function copies the data, so the lifetime of the input data is unimportant.
    pub unsafe extern "C" fn alloc_string(data: *const u16, len: u32) -> String {
        let slice = unsafe { slice::from_raw_parts(data, len as usize) };
        slice.to_vec().into()
    }
}

impl From<&str> for String {
    fn from(value: &str) -> Self {
        Self::from(value.encode_utf16().collect::<Vec<u16>>())
    }
}

impl From<Vec<u16>> for String {
    fn from(value: Vec<u16>) -> Self {
        let (ptr, len, capacity) = value.into_raw_parts();
        match (u32::try_from(len), u32::try_from(capacity)) {
            (Ok(len), Ok(capacity)) => Self { ptr, len, capacity },
            _ => panic!("String too long"),
        }
    }
}
