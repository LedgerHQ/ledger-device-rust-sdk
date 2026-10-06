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
//! translated: this is enforced by the [`PIC`](ledger_secure_sdk_sys::pic) wrapper.
//!
//! ```
//! let mut counter = unsafe { COUNTER.get_mut() };
//! println!("counter value is {}", *counter.get_ref());
//! counter.update(&(*counter.get_ref() - 1));
//! println!("counter value is {}", *counter.get_ref());
//! ```

use AtomicStorageElem::{StorageA, StorageB};
use ledger_secure_sdk_sys::nvm_write;

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
/// Warning: this wrapper does not provide any guarantee about update atomicity.
#[repr(align(64))]
#[derive(Copy, Clone)]
pub struct AlignedStorage<T> {
    /// Stored value.
    /// This is intentionally private to prevent direct write access (this is
    /// stored in Flash, so only the update method can change the value).
    value: T,
}

impl<T> AlignedStorage<T> {
    /// Create a `AlignedStorage<T>` initialized with a given value.
    /// This is to set the initial value of static `AlignedStorage<T>`, as the value
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

    /// Update the value by writing to the NVM memory.
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
/// has not been interrupted. Any value excepted 0 and 0xff may work.
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
            // This is currently guaranteed by the alignment of AlignedStorage.
            storage_a: SafeStorage<T>,
            storage_b: SafeStorage<T>, // We also accept situations where both storages are marked as valid, which
                                       // can happen with tearing. This is not a problem, and we consider the first
                                       // one is the "correct" one.
        }
    };
}

#[cfg(target_os = "nanox")]
atomic_storage!(256);
#[cfg(any(
    target_os = "nanosplus",
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p"
))]
atomic_storage!(512);

pub enum AtomicStorageElem {
    StorageA,
    StorageB,
}

impl<T> AtomicStorage<T>
where
    T: Copy,
{
    /// Create an `AtomicStorage<T>` initialized with a given value.
    pub const fn new(value: &T) -> AtomicStorage<T> {
        AtomicStorage {
            storage_a: SafeStorage::new(*value),
            storage_b: SafeStorage::new(*value),
        }
    }

    /// Returns which storage contains the latest data, or `None` when neither is valid.
    ///
    /// An update validates the storage it writes before it invalidates the other one, so an
    /// interrupted update of a storage holding a value never leaves both invalid. Neither being
    /// valid means the storage holds no value yet: its section was loaded zeroed (Speculos
    /// zeroes `.nvm_data`), or its first update was interrupted before the written storage was
    /// validated, and the next update stores the value again.
    fn which(&self) -> Option<AtomicStorageElem> {
        if self.storage_a.is_valid() {
            Some(StorageA)
        } else if self.storage_b.is_valid() {
            Some(StorageB)
        } else {
            None
        }
    }

    /// The stored value, or `None` when the storage holds no value yet (see [`Self::which`]).
    fn stored(&self) -> Option<&T> {
        match self.which()? {
            StorageA => Some(self.storage_a.get_ref()),
            StorageB => Some(self.storage_b.get_ref()),
        }
    }

    /// Returns the stored value, first storing `init` if the storage was never updated (both
    /// validity flags clear, as in the zeroed `.nvm_data` Speculos loads). Use it where
    /// [`SingleStorage::get_ref`] would panic on such a storage.
    pub fn get_or_init(&mut self, init: &T) -> &T {
        if self.which().is_none() {
            self.update(init);
        }
        self.get_ref()
    }
}

impl<T> SingleStorage<T> for AtomicStorage<T>
where
    T: Copy,
{
    /// Return reference to the stored value.
    ///
    /// # Panics
    ///
    /// Panics if the storage was never updated: its zeroed bytes are not a valid `T` for every
    /// type. [`AtomicStorage::get_or_init`] stores a value first instead.
    fn get_ref(&self) -> &T {
        self.stored().expect("invalidated atomic storage")
    }

    /// Update the value by writing to the NVM memory. A storage that was never updated takes
    /// the value as well.
    /// Warning: this can be vulnerable to tearing - leading to partial write.
    fn update(&mut self, value: &T) {
        match self.which() {
            Some(StorageA) | None => {
                self.storage_b.update(value);
                self.storage_a.invalidate();
            }
            Some(StorageB) => {
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

    /// The allocation flags, or `None` when they hold no value yet: the collection was never
    /// updated (its zeroed `.nvm_data`, as Speculos loads it) and is empty, which is what
    /// [`Collection::new`] stores.
    fn allocation(&self) -> Option<&[u8; N]> {
        self.flags.stored()
    }

    /// Finds and returns a reference to a free slot, or returns None if
    /// all slots are allocated.
    fn find_free_slot(&self) -> Option<usize> {
        match self.allocation() {
            Some(flags) => flags.iter().position(|&e| e != STORAGE_VALID),
            None if N > 0 => Some(0),
            None => None,
        }
    }

    /// Adds an item in the collection. Returns an error if there is not free
    /// slots.
    /// This operation is atomic.
    pub fn add(&mut self, value: &T) -> Result<(), StorageFullError> {
        match self.find_free_slot() {
            Some(i) => {
                self.slots[i].update(value);
                let mut new_flags = self.allocation().copied().unwrap_or([0; N]);
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
        if key >= N {
            return Err(KeyOutOfRange);
        }
        Ok(self
            .allocation()
            .is_some_and(|flags| flags[key] == STORAGE_VALID))
    }

    /// Returns the number of allocated slots.
    pub fn len(&self) -> usize {
        self.count_allocated(N)
    }

    /// Returns true if collection is empty
    pub fn is_empty(&self) -> bool {
        self.allocation()
            .is_none_or(|flags| !flags.contains(&STORAGE_VALID))
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
        self.allocation().map_or(0, |flags| {
            flags
                .iter()
                .take(len)
                .fold(0, |acc, &byte| acc + (byte == STORAGE_VALID) as u32) as usize
        })
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

#[cfg(test)]
mod tests {
    use super::{AtomicStorage, Collection, SingleStorage};
    use crate::NVMData;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[unsafe(link_section = ".nvm_data")]
    static mut NEVER_UPDATED: NVMData<AtomicStorage<[u8; 4]>> =
        NVMData::new(AtomicStorage::new(&[0; 4]));

    // An application started from a zeroed `.nvm_data`, as Speculos loads it for applications,
    // finds both validity flags of a storage it never updated clear: `get_or_init` stores the
    // initial value instead of exposing the zeroed bytes, and later calls keep what was
    // stored. The flags are cleared here explicitly, so the test does not depend on how the
    // test binary was loaded.
    #[test]
    fn atomic_storage_initializes_zeroed_nvm() {
        let pointer = &raw mut NEVER_UPDATED;
        let storage = unsafe { (*pointer).get_mut() };
        storage.storage_a.invalidate();
        storage.storage_b.invalidate();
        assert_eq!(*storage.get_or_init(&[7; 4]), [7; 4]);
        assert_eq!(*storage.get_ref(), [7; 4]);
        storage.update(&[1, 2, 3, 4]);
        assert_eq!(*storage.get_or_init(&[9; 4]), [1, 2, 3, 4]);
        storage.update(&[5, 6, 7, 8]);
        assert_eq!(*storage.get_ref(), [5, 6, 7, 8]);
    }

    // A never-updated storage also takes a plain update.
    #[test]
    fn atomic_storage_updates_zeroed_nvm() {
        let pointer = &raw mut NEVER_UPDATED;
        let storage = unsafe { (*pointer).get_mut() };
        storage.storage_a.invalidate();
        storage.storage_b.invalidate();
        storage.update(&[3; 4]);
        assert_eq!(*storage.get_ref(), [3; 4]);
    }

    #[unsafe(link_section = ".nvm_data")]
    static mut NEVER_UPDATED_COLLECTION: NVMData<Collection<u32, 4>> =
        NVMData::new(Collection::new(0));

    // A collection whose allocation flags were never updated, as in the zeroed `.nvm_data`
    // Speculos loads, is empty, which is what `Collection::new` stores: it is read and added
    // to without panicking, and no `clear` is needed first.
    #[test]
    fn collection_with_zeroed_flags_is_empty() {
        let pointer = &raw mut NEVER_UPDATED_COLLECTION;
        let collection = unsafe { (*pointer).get_mut() };
        collection.flags.storage_a.invalidate();
        collection.flags.storage_b.invalidate();
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.is_empty(), true);
        assert_eq!(collection.remaining(), 4);
        assert_eq!(collection.get(0), None);
        assert_eq!((&*collection).into_iter().next(), None);
        assert_eq!(collection.add(&7).is_ok(), true);
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.get(0), Some(&7));
    }
}
