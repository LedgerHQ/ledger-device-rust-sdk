//! Application storage.
//!
//! Safe wrappers around the C SDK's `app_storage` (`lib_standard_app/app_storage.c`): a byte
//! area at the start of the application data region, behind a header carrying a version of
//! the data, the properties of the content and a CRC-32 over header and data. It is the
//! storage the OS is meant to keep across application updates; data kept in `.nvm_data`
//! statics is not.
//!
//! The storage is initialized when the application starts: an uninitialized or corrupted
//! storage gets a fresh, empty header. Its capacity is set at build time by the
//! `APP_STORAGE_SIZE` environment variable (480 bytes by default).
//!
//! This module is only available with the `app_storage` Cargo feature. The
//! `app_storage_settings` and `app_storage_data` features set the matching
//! [`Properties`] in the header.
//!
//! # Examples
//!
//! ```no_run
//! use ledger_device_sdk::app_storage;
//!
//! let mut counter = [0u8; 4];
//! if app_storage::read(&mut counter, 0).is_err() {
//!     // Nothing written yet.
//!     counter = 0u32.to_le_bytes();
//! }
//! let next = u32::from_le_bytes(counter).wrapping_add(1);
//! app_storage::write(&next.to_le_bytes(), 0).unwrap();
//! app_storage::increment_data_version();
//! ```

use ledger_secure_sdk_sys as sys;

/// Error returned by the storage functions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AppStorageError {
    /// The buffer or the range is invalid (for example `offset + len` overflows).
    InvalidArgument,
    /// The range was never written: it ends beyond the current data size.
    NoDataAvailable,
    /// The range ends beyond the storage capacity.
    Overflow,
    /// The storage header is not initialized.
    InvalidHeader,
    /// The CRC does not match the header and data.
    Corrupted,
    /// A status code this module does not know.
    Unknown(i32),
}

impl From<i32> for AppStorageError {
    fn from(code: i32) -> Self {
        match code {
            sys::APP_STORAGE_ERR_INVALID_ARGUMENT => Self::InvalidArgument,
            sys::APP_STORAGE_ERR_NO_DATA_AVAILABLE => Self::NoDataAvailable,
            sys::APP_STORAGE_ERR_OVERFLOW => Self::Overflow,
            sys::APP_STORAGE_ERR_INVALID_HEADER => Self::InvalidHeader,
            sys::APP_STORAGE_ERR_CORRUPTED => Self::Corrupted,
            other => Self::Unknown(other),
        }
    }
}

/// What the storage declares to hold, set at build time by the `app_storage_settings` and
/// `app_storage_data` features.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Properties {
    /// The storage holds settings.
    pub settings: bool,
    /// The storage holds application data.
    pub data: bool,
}

/// Maps a C read/write status to a result: success is the byte count that was asked for.
fn check(status: i32, len: u32) -> Result<(), AppStorageError> {
    match u32::try_from(status) {
        Ok(done) if done == len => Ok(()),
        Ok(_) => Err(AppStorageError::Unknown(status)),
        Err(_) => Err(AppStorageError::from(status)),
    }
}

/// Fills `buf` with the stored bytes starting at `offset`.
///
/// An empty `buf` reads nothing and succeeds.
///
/// # Errors
///
/// [`AppStorageError::NoDataAvailable`] when the range ends beyond what was written so far,
/// [`AppStorageError::InvalidArgument`] when it cannot be expressed.
pub fn read(buf: &mut [u8], offset: u32) -> Result<(), AppStorageError> {
    if buf.is_empty() {
        return Ok(());
    }
    let len = u32::try_from(buf.len()).map_err(|_| AppStorageError::InvalidArgument)?;
    // SAFETY: `buf` is valid for `len` writable bytes; the C side checks the range against
    // the written data size before copying.
    let status = unsafe { sys::app_storage_read(buf.as_mut_ptr().cast(), len, offset) };
    check(status, len)
}

/// Writes `data` at `offset` and updates the data size and the CRC.
///
/// An empty `data` writes nothing and succeeds. The data version is not changed: call
/// [`increment_data_version`] or [`set_data_version`] when the content format changes.
///
/// # Errors
///
/// [`AppStorageError::Overflow`] when the range ends beyond the storage capacity,
/// [`AppStorageError::InvalidArgument`] when it cannot be expressed.
pub fn write(data: &[u8], offset: u32) -> Result<(), AppStorageError> {
    if data.is_empty() {
        return Ok(());
    }
    let len = u32::try_from(data.len()).map_err(|_| AppStorageError::InvalidArgument)?;
    // SAFETY: `data` is valid for `len` readable bytes; the C side checks the range against
    // the storage capacity before writing.
    let status = unsafe { sys::app_storage_write(data.as_ptr().cast(), len, offset) };
    check(status, len)
}

/// Number of bytes written so far: the end of the furthest range ever written.
pub fn size() -> u32 {
    // SAFETY: reads the header of the storage initialized at application start.
    unsafe { sys::app_storage_get_size() }
}

/// Version of the stored data, 1 for a fresh storage.
pub fn data_version() -> u32 {
    // SAFETY: reads the header of the storage initialized at application start.
    unsafe { sys::app_storage_get_data_version() }
}

/// Sets the version of the stored data.
pub fn set_data_version(version: u32) {
    // SAFETY: writes the header field and the CRC of the initialized storage.
    unsafe { sys::app_storage_set_data_version(version) }
}

/// Increments the version of the stored data, wrapping from `u32::MAX` to 1.
pub fn increment_data_version() {
    // SAFETY: writes the header field and the CRC of the initialized storage.
    unsafe { sys::app_storage_increment_data_version() }
}

/// Properties declared in the header.
pub fn properties() -> Properties {
    // SAFETY: reads the header of the storage initialized at application start.
    let bits = u32::from(unsafe { sys::app_storage_get_properties() });
    Properties {
        settings: bits & sys::APP_STORAGE_PROP_SETTINGS != 0,
        data: bits & sys::APP_STORAGE_PROP_DATA != 0,
    }
}

/// Erases the stored data and writes a fresh header: size 0, data version 1.
pub fn reset() {
    // SAFETY: rewrites the storage region declared by the C SDK.
    unsafe { sys::app_storage_reset() }
}

// The tests share one storage, so each starts from `reset()`. They assume the default
// capacity of 480 bytes (APP_STORAGE_SIZE unset).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const CAPACITY: u32 = 480;

    // A fresh storage holds nothing: reading reports the range as never written.
    #[test]
    fn test_app_storage_reset_is_empty() {
        reset();
        assert_eq!(size(), 0);
        assert_eq!(data_version(), 1);
        let mut buf = [0u8; 1];
        assert_eq!(read(&mut buf, 0), Err(AppStorageError::NoDataAvailable));
    }

    // Written bytes read back, and the size follows the furthest written range.
    #[test]
    fn test_app_storage_write_read_round_trip() {
        reset();
        assert_eq!(write(&[1, 2, 3, 4], 8), Ok(()));
        assert_eq!(size(), 12);
        let mut buf = [0u8; 4];
        assert_eq!(read(&mut buf, 8), Ok(()));
        assert_eq!(buf, [1, 2, 3, 4]);
        // Bytes below the written range exist and are zero.
        let mut low = [0xffu8; 8];
        assert_eq!(read(&mut low, 0), Ok(()));
        assert_eq!(low, [0u8; 8]);
        // Reading past the size fails instead of returning stale bytes.
        assert_eq!(read(&mut buf, 9), Err(AppStorageError::NoDataAvailable));
    }

    // The last byte of the capacity is writable, one more is not.
    #[test]
    fn test_app_storage_capacity_bound() {
        reset();
        assert_eq!(write(&[0xaa], CAPACITY - 1), Ok(()));
        assert_eq!(size(), CAPACITY);
        assert_eq!(write(&[0xaa], CAPACITY), Err(AppStorageError::Overflow));
    }

    // A range whose end does not fit in u32 is rejected, not wrapped.
    #[test]
    fn test_app_storage_offset_overflow() {
        reset();
        assert_eq!(
            write(&[1, 2], u32::MAX),
            Err(AppStorageError::InvalidArgument)
        );
        let mut buf = [0u8; 2];
        assert_eq!(
            read(&mut buf, u32::MAX),
            Err(AppStorageError::InvalidArgument)
        );
    }

    // Empty buffers succeed without touching the storage.
    #[test]
    fn test_app_storage_empty_buffers() {
        reset();
        assert_eq!(write(&[], 0), Ok(()));
        assert_eq!(read(&mut [], 0), Ok(()));
        assert_eq!(size(), 0);
    }

    // The data version is set and incremented; reset brings it back to 1.
    #[test]
    fn test_app_storage_data_version() {
        reset();
        set_data_version(7);
        assert_eq!(data_version(), 7);
        increment_data_version();
        assert_eq!(data_version(), 8);
        reset();
        assert_eq!(data_version(), 1);
    }

    // The header declares exactly the properties selected by the features.
    #[test]
    fn test_app_storage_properties_follow_features() {
        assert_eq!(
            properties(),
            Properties {
                settings: cfg!(feature = "app_storage_settings"),
                data: cfg!(feature = "app_storage_data"),
            }
        );
    }
}
