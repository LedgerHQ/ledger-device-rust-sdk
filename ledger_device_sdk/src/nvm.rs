//! High-level library to store data in NVM memory
//!
//! This module provides primitives to store objects in the Flash memory used
//! by the application. It implements basic update mechanisms, eventually with
//! atomicity guarantees against possible tearing.
//!
//! There is no filesystem or NVM allocated in BOLOS. Therefore any object
//! stored by the application uses a fixed space in the program itself.
//!
//! # Examples
//!
//! The following piece of code declares a storage for an integer, with atomic
//! update:
//!
//! ```
//! use ledger_device_sdk::NVMData;
//! use ledger_device_sdk::nvm::AtomicStorage;
//!
//! // This is necessary to store the object in NVM and not in RAM
//! #[link_section=".nvm_data"]
//! static mut COUNTER: NVMData<AtomicStorage<i32>> =
//!     NVMData::new(AtomicStorage::new(&3));
//! ```
//!
//! In the program, `COUNTER` must not be used directly. It is a static variable
//! and using it would require unsafe everytime. Instead, a reference must be
//! taken, so the borrow checker will be able to do its job correctly. This is
//! crucial: the memory location of the stored object may be moved due to
//! atomicity implementation, and the borrow checker should prevent any use of
//! old references to a value which has been updated and moved elsewhere.
//!
//! Furthermore, since the data is stored in Code space, it is relocated during
//! application installation. Therefore the address to this data must be
//! translated: this is enforced by the [`PIC`](crate::PIC) wrapper.
//!
//! ```
//! let mut counter = unsafe { COUNTER.get_mut() };
//! println!("counter value is {}", *counter.get_ref());
//! counter.update(&(*counter.get_ref() - 1));
//! println!("counter value is {}", *counter.get_ref());
//! ```

use ledger_secure_sdk_sys::nvm_write;
use AtomicStorageElem::{StorageA, StorageB};

// Warning: currently alignment is fixed by magic values everywhere, since
// rust does not allow using a constant in repr(align(...))
// This code will work correctly only for the currently set page size of 64.

/// Returned when trying to insert data when no more space is available
pub struct StorageFullError;

/// What storage of single element should implement
///
/// The address of the stored object, returned with get_ref, MUST remain the
/// same until update is called.
///
/// The update method may move the object, for instance with AtomicStorage.
///
/// The borrow checker should prevent keeping references after updating the
/// content, so everything should go fine...
pub trait SingleStorage<T> {
    /// Returns a non-mutable reference to the stored object.
    fn get_ref(&self) -> &T;
    fn update(&mut self, value: &T);
}

/// Wraps a variable stored in Non-Volatile Memory to provide read and update
/// methods.
///
/// Always aligned to the beginning of a page to prevent different
/// AlignedStorage sharing a common Flash page (this is required to implement
/// unfinished write detection in SafeStorage and atomic operations in
/// AtomicStorage).
///
/// Warning: this wrapper does not provide any garantee about update atomicity.
#[repr(align(64))]
#[derive(Copy, Clone)]
pub struct AlignedStorage<T> {
    /// Stored value.
    /// This is intentionally private to prevent direct write access (this is
    /// stored in Flash, so only the update method can change the value).
    value: T,
}

impl<T> AlignedStorage<T> {
    /// Create a Storage<T> initialized with a given value.
    /// This is to set the initial value of static Storage<T>, as the value
    /// member is private.
    pub const fn new(value: T) -> AlignedStorage<T> {
        AlignedStorage { value }
    }
}

impl<T> SingleStorage<T> for AlignedStorage<T> {
    /// Return non-mutable reference to the stored value.
    /// The address is always the same for AlignedStorage.
    fn get_ref(&self) -> &T {
        &self.value
    }

    /// Update the value by writting to the NVM memory.
    /// Warning: this can be vulnerable to tearing - leading to partial write.
    fn update(&mut self, value: &T) {
        unsafe {
            nvm_write(
                &self.value as *const T as *const core::ffi::c_void as *mut core::ffi::c_void,
                value as *const T as *const core::ffi::c_void as *mut core::ffi::c_void,
                core::mem::size_of::<T>() as u32,
            );
            let mut _dummy = &self.value;
        }
    }
}

/// Just a non-zero magic to mark a storage as valid, when the update procedure
/// has not been interupted. Any value excepted 0 and 0xff may work.
const STORAGE_VALID: u8 = 0xa5;

/// Non-Volatile data storage, with a flag to detect corruption if the update
/// has been interrupted somehow.
///
/// During update:
/// 1. The flag is reset to 0
/// 2. The value is updated
/// 3. The flag is restored to STORAGE_VALID
pub struct SafeStorage<T> {
    flag: AlignedStorage<u8>,
    value: AlignedStorage<T>,
}

impl<T> SafeStorage<T> {
    pub const fn new(value: T) -> SafeStorage<T> {
        SafeStorage {
            flag: AlignedStorage::new(STORAGE_VALID),
            value: AlignedStorage::new(value),
        }
    }

    /// Set the validation flag to zero to mark the content as invalid.
    /// This used for instance by the atomic storage management.
    pub fn invalidate(&mut self) {
        self.flag.update(&0);
    }

    /// Returns true if the stored value is not corrupted, false if a previous
    /// update operation has been interrupted.
    pub fn is_valid(&self) -> bool {
        *self.flag.get_ref() == STORAGE_VALID
    }
}

impl<T> SingleStorage<T> for SafeStorage<T> {
    /// Return non-mutable reference to the stored value.
    /// Panic if the storage is not valid (corrupted).
    fn get_ref(&self) -> &T {
        assert_eq!(*self.flag.get_ref(), STORAGE_VALID);
        self.value.get_ref()
    }

    fn update(&mut self, value: &T) {
        self.flag.update(&0);
        self.value.update(value);
        self.flag.update(&STORAGE_VALID);
    }
}

/// Non-Volatile data storage with atomic update support.
/// Takes at minimum twice the size of the data to be stored, plus 2 bytes.
/// Aligning to the required page size is done through a macro
/// as `#[repr(align(N))]` does not accept variable 'N'
macro_rules! atomic_storage {
    ($n:expr) => {
        #[repr(align($n))]
        pub struct AtomicStorage<T> {
            // We must keep the storage B in another page, so when we update the
            // storage A, erasing the page of A won't modify the storage for B.
            // This is currently garanteed by the alignment of AlignedStorage.
            storage_a: SafeStorage<T>,
            storage_b: SafeStorage<T>, // We also accept situations where both storages are marked as valid, which
                                       // can happen with tearing. This is not a problem, and we consider the first
                                       // one is the "correct" one.
        }
    };
}

#[cfg(target_os = "nanox")]
atomic_storage!(256);
#[cfg(any(target_os = "nanosplus", target_os = "stax", target_os = "flex"))]
atomic_storage!(512);

pub enum AtomicStorageElem {
    StorageA,
    StorageB,
}

impl<T> AtomicStorage<T>
where
    T: Copy,
{
    /// Create an AtomicStorage<T> initialized with a given value.
    pub const fn new(value: &T) -> AtomicStorage<T> {
        AtomicStorage {
            storage_a: SafeStorage::new(*value),
            storage_b: SafeStorage::new(*value),
        }
    }

    /// Returns which storage contains the latest valid data.
    ///
    /// # Panics
    ///
    /// Panics if both storage elements are invalid (data corrupton),
    /// although data corruption shall not be possible with tearing.
    fn which(&self) -> AtomicStorageElem {
        if self.storage_a.is_valid() {
            StorageA
        } else if self.storage_b.is_valid() {
            StorageB
        } else {
            panic!("invalidated atomic storage");
        }
    }
}

impl<T> SingleStorage<T> for AtomicStorage<T>
where
    T: Copy,
{
    /// Return reference to the stored value.
    fn get_ref(&self) -> &T {
        match self.which() {
            StorageA => self.storage_a.get_ref(),
            StorageB => self.storage_b.get_ref(),
        }
    }

    /// Update the value by writting to the NVM memory.
    /// Warning: this can be vulnerable to tearing - leading to partial write.
    fn update(&mut self, value: &T) {
        match self.which() {
            StorageA => {
                self.storage_b.update(value);
                self.storage_a.invalidate();
            }
            StorageB => {
                self.storage_a.update(value);
                self.storage_b.invalidate();
            }
        }
    }
}
pub struct KeyOutOfRange;

/// A Non-Volatile fixed-size collection of fixed-size items.
/// Items insertion and deletion are atomic.
/// Items update is not implemented because the atomicity of this operation
/// cannot be guaranteed here.
// We use the term `index` to represent the user-facing number of an element in the collection,
// and the term `key` to represent the underlying offset at which the element is located in the collection.
// e.g with `[0, 0, 1, 1, 0, 1, 0]` (with 0s representing free slots and 1s representing allocated slots)
//            ↑  ↑  ↑  ↑  ↑  ↑  ↑
// index:     -  -  0  1  -  2  -
// key:       0, 1, 2, 3, 4, 5, 6
pub struct Collection<T, const N: usize> {
    flags: AtomicStorage<[u8; N]>,
    slots: [AlignedStorage<T>; N],
}

impl<T, const N: usize> Collection<T, N>
where
    T: Copy,
{
    pub const fn new(value: T) -> Collection<T, N> {
        Collection {
            flags: AtomicStorage::new(&[0; N]),
            slots: [AlignedStorage::new(value); N],
        }
    }

    /// Finds and returns a reference to a free slot, or returns None if
    /// all slots are allocated.
    fn find_free_slot(&self) -> Option<usize> {
        self.flags
            .get_ref()
            .iter()
            .position(|&e| e != STORAGE_VALID)
    }

    /// Adds an item in the collection. Returns an error if there is not free
    /// slots.
    /// This operation is atomic.
    pub fn add(&mut self, value: &T) -> Result<(), StorageFullError> {
        match self.find_free_slot() {
            Some(i) => {
                self.slots[i].update(value);
                let mut new_flags = *self.flags.get_ref();
                new_flags[i] = STORAGE_VALID;
                self.flags.update(&new_flags);
                Ok(())
            }
            None => Err(StorageFullError),
        }
    }

    /// Returns a boolean representing whether the slot at `key` was allocated or not.
    ///
    /// # Errors
    ///
    /// Returns an error if the `key` is out of range.
    fn is_allocated(&self, key: usize) -> Result<bool, KeyOutOfRange> {
        match self.flags.get_ref().get(key) {
            Some(&byte) => {
                if byte == STORAGE_VALID {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Err(KeyOutOfRange),
        }
    }

    /// Returns the number of allocated slots.
    pub fn len(&self) -> usize {
        self.count_allocated(N)
    }

    /// Returns true if collection is empty
    pub fn is_empty(&self) -> bool {
        !self.flags.get_ref().iter().any(|v| *v == STORAGE_VALID)
    }

    /// Returns the maximum number of items the collection can store.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns the remaining number of items which can be added to the
    /// collection.
    pub fn remaining(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Counts the number of allocated slots up until `len`.
    fn count_allocated(&self, len: usize) -> usize {
        self.flags
            .get_ref()
            .iter()
            .take(len)
            .fold(0, |acc, &byte| acc + (byte == STORAGE_VALID) as u32) as usize
    }

    /// Returns the `key` of an item in the internal storage, given the `index`
    /// in the collection. If `index` is too big, None is returned.
    ///
    /// # Arguments
    ///
    /// * `index` - Index in the collection
    fn index_to_key(&self, index: usize) -> Option<usize> {
        // Neat optimization: start by setting `next` to index,
        // because we know we could not have found `index` allocated slots beforehand.
        let mut key = index;
        // Now count the number of allocated slots we have found up
        // until this `index` (without including the slot at `index` itself).
        let mut allocated_count = self.count_allocated(index);
        loop {
            let is_allocated = self.is_allocated(key).ok()?;
            if is_allocated {
                if allocated_count == index {
                    return Some(key);
                }
                allocated_count += 1;
            }
            key += 1;
        }
    }

    /// Returns reference to an item, or None if the index is out of bounds
    ///
    /// # Arguments
    ///
    /// * `index` - Item index
    pub fn get(&self, index: usize) -> Option<&T> {
        match self.index_to_key(index) {
            Some(key) => Some(self.slots[key].get_ref()),
            None => None,
        }
    }

    /// Removes the item located at `index` from the collection.
    ///
    /// # Arguments
    ///
    /// * `index` - Item index
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) {
        let key = self.index_to_key(index).unwrap();
        let mut new_flags = *self.flags.get_ref();
        new_flags[key] = 0;
        self.flags.update(&new_flags);
    }

    /// Removes all the items from the collection.
    /// This operation is atomic.
    pub fn clear(&mut self) {
        self.flags.update(&[0; N]);
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Collection<T, N>
where
    T: Copy,
{
    type Item = &'a T;
    type IntoIter = CollectionIterator<'a, T, N>;

    fn into_iter(self) -> CollectionIterator<'a, T, N> {
        CollectionIterator {
            container: self,
            next_key: 0,
        }
    }
}

pub struct CollectionIterator<'a, T, const N: usize>
where
    T: Copy,
{
    container: &'a Collection<T, N>,
    next_key: usize,
}

impl<'a, T, const N: usize> Iterator for CollectionIterator<'a, T, N>
where
    T: Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> core::option::Option<&'a T> {
        loop {
            let is_allocated = self.container.is_allocated(self.next_key).ok()?;
            self.next_key += 1;
            if is_allocated {
                return Some(self.container.slots[self.next_key - 1].get_ref());
            }
        }
    }
}
