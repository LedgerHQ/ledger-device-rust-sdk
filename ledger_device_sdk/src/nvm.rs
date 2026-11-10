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
//!
//! # What can be stored
//!
//! A stored value is a copy of its bytes, read back as a `T` from flash, so
//! `T` must be plain data. Every storage requires `T: Copy`, which rules out
//! types owning heap memory (`String`, `Vec`, `Box`, ...), but not every type
//! that is unsound to store:
//!
//! * References and raw pointers point to RAM, which does not hold the same
//!   data from one run of the app to the next.
//! * `bool`, `char`, enums, `NonZero*` and other types with invalid bit
//!   patterns can be read back invalid: an interrupted [`AlignedStorage`]
//!   update leaves a mix of old and new bytes, and a storage never updated
//!   holds zeroes when its section is loaded zeroed (as Speculos does).
//! * Padding bytes are stored too, with whatever they held.
//!
//! Prefer integers and byte arrays such as `[u8; N]`, and encode other values
//! explicitly, checking them when decoding.

use AtomicStorageElem::{StorageA, StorageB};
use ledger_secure_sdk_sys::nvm_write;

// `repr(align(...))` does not accept a constant, so alignments are literals:
// 64 bytes for `AlignedStorage`, and the flash page size of each device for the
// flag and the value of `SafeStorage` and the replicas of `AtomicStorage`.

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
/// Aligned to 64 bytes. This is smaller than a flash page (256 or 512 bytes),
/// so neighbouring values can share a page, and an interrupted update can also
/// damage them.
///
/// Warning: this wrapper does not provide any guarantee about update atomicity.
#[repr(align(64))]
#[derive(Copy, Clone)]
pub struct AlignedStorage<T: Copy> {
    /// Stored value.
    /// This is intentionally private to prevent direct write access (this is
    /// stored in Flash, so only the update method can change the value).
    value: T,
}

impl<T: Copy> AlignedStorage<T> {
    /// Create a `AlignedStorage<T>` initialized with a given value.
    /// This is to set the initial value of static `AlignedStorage<T>`, as the value
    /// member is private.
    pub const fn new(value: T) -> AlignedStorage<T> {
        AlignedStorage { value }
    }
}

impl<T: Copy> SingleStorage<T> for AlignedStorage<T> {
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

/// Wraps a part of a storage so that it starts on a flash page of its own, and
/// is padded to a whole number of pages: updating it never touches the page of
/// another part.
///
/// The alignment is the flash page size, set per device through a macro as
/// `#[repr(align(N))]` does not accept variable 'N'.
macro_rules! page_aligned {
    ($n:expr) => {
        #[repr(align($n))]
        struct PageAligned<T>(T);

        /// Flash page size, matching `PAGE_SIZE` in the device's linker layout.
        const PAGE_SIZE: usize = $n;
    };
}

#[cfg(target_os = "nanox")]
page_aligned!(256);
#[cfg(any(
    target_os = "nanosplus",
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p"
))]
page_aligned!(512);

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
///
/// The flag and the value each lie on flash pages of their own, so writing the
/// flag never touches the value: a storage takes at least two pages.
pub struct SafeStorage<T: Copy> {
    flag: PageAligned<AlignedStorage<u8>>,
    value: PageAligned<AlignedStorage<T>>,
}

impl<T: Copy> SafeStorage<T> {
    pub const fn new(value: T) -> SafeStorage<T> {
        SafeStorage {
            flag: PageAligned(AlignedStorage::new(STORAGE_VALID)),
            value: PageAligned(AlignedStorage::new(value)),
        }
    }

    /// Set the validation flag to zero to mark the content as invalid.
    /// This used for instance by the atomic storage management.
    pub fn invalidate(&mut self) {
        self.flag.0.update(&0);
    }

    /// Returns true if the stored value is not corrupted, false if a previous
    /// update operation has been interrupted.
    pub fn is_valid(&self) -> bool {
        *self.flag.0.get_ref() == STORAGE_VALID
    }
}

impl<T: Copy> SingleStorage<T> for SafeStorage<T> {
    /// Return non-mutable reference to the stored value.
    /// Panic if the storage is not valid (corrupted).
    fn get_ref(&self) -> &T {
        assert_eq!(*self.flag.0.get_ref(), STORAGE_VALID);
        self.value.0.get_ref()
    }

    fn update(&mut self, value: &T) {
        self.flag.0.update(&0);
        self.value.0.update(value);
        self.flag.0.update(&STORAGE_VALID);
    }
}

/// Whether both replicas of an `AtomicStorage<T>` start on a page boundary and
/// share no page.
macro_rules! replicas_on_own_pages {
    ($t:ty) => {{
        let a = core::mem::offset_of!(AtomicStorage<$t>, storage_a);
        let b = core::mem::offset_of!(AtomicStorage<$t>, storage_b);
        let size = core::mem::size_of::<PageAligned<SafeStorage<$t>>>();
        a % PAGE_SIZE == 0 && b % PAGE_SIZE == 0 && a.abs_diff(b) >= size
    }};
}

/// Whether the flag and the value of a `SafeStorage<T>` start on a page
/// boundary and share no page.
macro_rules! flag_and_value_on_own_pages {
    ($t:ty) => {{
        let f = core::mem::offset_of!(SafeStorage<$t>, flag);
        let v = core::mem::offset_of!(SafeStorage<$t>, value);
        let f_size = core::mem::size_of::<PageAligned<AlignedStorage<u8>>>();
        let v_size = core::mem::size_of::<PageAligned<AlignedStorage<$t>>>();
        f % PAGE_SIZE == 0 && v % PAGE_SIZE == 0 && (f + f_size <= v || v + v_size <= f)
    }};
}

// Checked at build time, for every device, for values smaller and larger
// than a page.
const _: () = assert!(flag_and_value_on_own_pages!(u8));
const _: () = assert!(flag_and_value_on_own_pages!(i32));
const _: () = assert!(flag_and_value_on_own_pages!([u8; 10]));
const _: () = assert!(flag_and_value_on_own_pages!([u8; 700]));
const _: () = assert!(replicas_on_own_pages!(u8));
const _: () = assert!(replicas_on_own_pages!(i32));
const _: () = assert!(replicas_on_own_pages!([u8; 10]));
const _: () = assert!(replicas_on_own_pages!([u8; 700]));

/// Non-Volatile data storage with atomic update support.
///
/// Holds two replicas of the value, each a [`SafeStorage`] on flash pages of
/// its own, so it takes at least four pages.
pub struct AtomicStorage<T: Copy> {
    storage_a: PageAligned<SafeStorage<T>>,
    // We also accept situations where both storages are marked as valid, which
    // can happen with tearing. This is not a problem, and we consider the first
    // one is the "correct" one.
    storage_b: PageAligned<SafeStorage<T>>,
}

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
            storage_a: PageAligned(SafeStorage::new(*value)),
            storage_b: PageAligned(SafeStorage::new(*value)),
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
        if self.storage_a.0.is_valid() {
            Some(StorageA)
        } else if self.storage_b.0.is_valid() {
            Some(StorageB)
        } else {
            None
        }
    }

    /// The stored value, or `None` when the storage holds no value yet (see [`Self::which`]).
    fn stored(&self) -> Option<&T> {
        match self.which()? {
            StorageA => Some(self.storage_a.0.get_ref()),
            StorageB => Some(self.storage_b.0.get_ref()),
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

impl<const N: usize> AtomicStorage<[u8; N]> {
    /// Makes the storage that is not current hold the current bytes, so the value the last
    /// update replaced leaves no trace in NVM; writes only when the two differ.
    ///
    /// An update writes the other storage and then invalidates the current one, whose bytes
    /// stay. Writing twice clears them, but a power loss between the two writes leaves them for
    /// good, so an application storing secrets calls this at start to finish such a write.
    /// A storage that was never updated is left as it is.
    ///
    /// Only byte arrays settle: the other storage may hold a write a power loss cut short, which
    /// is still a valid `[u8; N]` but not necessarily a valid value of another type, and the
    /// comparison must cover every stored byte, padding and unused enum payload included, to
    /// tell whether anything of the replaced value is left.
    pub fn settle(&mut self) {
        let (current, other) = if self.storage_a.0.is_valid() {
            (&self.storage_a.0, &self.storage_b.0)
        } else if self.storage_b.0.is_valid() {
            (&self.storage_b.0, &self.storage_a.0)
        } else {
            return;
        };
        if current.value.0.get_ref() == other.value.0.get_ref() {
            return;
        }
        // Writing the current value again lands it in the other storage, which becomes current;
        // the one invalidated holds the same bytes.
        let value = *current.value.0.get_ref();
        self.update(&value);
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
                self.storage_b.0.update(value);
                self.storage_a.0.invalidate();
            }
            Some(StorageB) => {
                self.storage_a.0.update(value);
                self.storage_b.0.invalidate();
            }
        }
    }
}
pub struct KeyOutOfRange;

/// A Non-Volatile fixed-size collection of fixed-size items.
/// Items insertion and deletion are atomic: an interrupted one leaves the
/// collection as it was before or after it.
///
/// Items are not page-isolated (see [`AlignedStorage`]): an interrupted
/// insertion can also damage the items sharing the flash page of the new one.
/// Items update is not implemented because the atomicity of this operation
/// cannot be guaranteed here.
// We use the term `index` to represent the user-facing number of an element in the collection,
// and the term `key` to represent the underlying offset at which the element is located in the collection.
// e.g with `[0, 0, 1, 1, 0, 1, 0]` (with 0s representing free slots and 1s representing allocated slots)
//            ↑  ↑  ↑  ↑  ↑  ↑  ↑
// index:     -  -  0  1  -  2  -
// key:       0, 1, 2, 3, 4, 5, 6
pub struct Collection<T: Copy, const N: usize> {
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
    use super::{AtomicStorage, Collection, STORAGE_VALID, SafeStorage, SingleStorage};
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
        storage.storage_a.0.invalidate();
        storage.storage_b.0.invalidate();
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
        storage.storage_a.0.invalidate();
        storage.storage_b.0.invalidate();
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
        collection.flags.storage_a.0.invalidate();
        collection.flags.storage_b.0.invalidate();
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.is_empty(), true);
        assert_eq!(collection.remaining(), 4);
        assert_eq!(collection.get(0), None);
        assert_eq!((&*collection).into_iter().next(), None);
        assert_eq!(collection.add(&7).is_ok(), true);
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.get(0), Some(&7));
    }

    #[unsafe(link_section = ".nvm_data")]
    static mut SETTLED: NVMData<AtomicStorage<[u8; 4]>> = NVMData::new(AtomicStorage::new(&[0; 4]));

    // An update leaves the replaced value in the storage it invalidates; settling overwrites it
    // with the current value, keeps the current value readable, and writes nothing once both
    // storages agree.
    #[test]
    fn atomic_storage_settle_overwrites_the_replaced_value() {
        let pointer = &raw mut SETTLED;
        let storage = unsafe { (*pointer).get_mut() };
        storage.update(&[1; 4]);
        storage.update(&[2; 4]);
        let replaced = [
            *storage.storage_a.0.value.0.get_ref(),
            *storage.storage_b.0.value.0.get_ref(),
        ];
        assert_eq!(replaced.contains(&[1; 4]), true);
        storage.settle();
        assert_eq!(*storage.storage_a.0.value.0.get_ref(), [2; 4]);
        assert_eq!(*storage.storage_b.0.value.0.get_ref(), [2; 4]);
        assert_eq!(*storage.get_ref(), [2; 4]);
        storage.settle();
        assert_eq!(*storage.get_ref(), [2; 4]);
    }

    // A power loss in the middle of a write leaves the other storage with part of the new
    // bytes and part of the replaced ones; settling still finds them different and overwrites
    // every byte.
    #[test]
    fn atomic_storage_settle_finishes_a_torn_write() {
        let pointer = &raw mut SETTLED;
        let storage = unsafe { (*pointer).get_mut() };
        storage.update(&[3; 4]);
        storage.update(&[4; 4]);
        let other = if storage.storage_a.0.is_valid() {
            &mut storage.storage_b.0
        } else {
            &mut storage.storage_a.0
        };
        other.value.0.update(&[4, 4, 3, 3]);
        storage.settle();
        assert_eq!(*storage.storage_a.0.value.0.get_ref(), [4; 4]);
        assert_eq!(*storage.storage_b.0.value.0.get_ref(), [4; 4]);
        assert_eq!(*storage.get_ref(), [4; 4]);
    }

    #[unsafe(link_section = ".nvm_data")]
    static mut INTERRUPTED: NVMData<AtomicStorage<[u8; 4]>> =
        NVMData::new(AtomicStorage::new(&[0; 4]));

    /// The replica an update of `storage` writes, and the one it invalidates.
    fn replicas(
        storage: &mut AtomicStorage<[u8; 4]>,
        writes_a: bool,
    ) -> (&mut SafeStorage<[u8; 4]>, &mut SafeStorage<[u8; 4]>) {
        if writes_a {
            (&mut storage.storage_a.0, &mut storage.storage_b.0)
        } else {
            (&mut storage.storage_b.0, &mut storage.storage_a.0)
        }
    }

    /// Steps through an update of `storage`, holding `old`, to `new`, and checks
    /// what a power loss after each write leaves. With the flag and the value of
    /// a replica on pages of their own, and the replicas on pages of their own, a
    /// write damages only the part it writes: these are all the states an
    /// interrupted update can leave.
    fn check_interrupted_update(
        storage: &mut AtomicStorage<[u8; 4]>,
        old: [u8; 4],
        new: [u8; 4],
    ) -> Result<(), ()> {
        assert_eq!(*storage.get_ref(), old);
        let writes_a = !storage.storage_a.0.is_valid();
        replicas(storage, writes_a).0.invalidate();
        assert_eq!(*storage.get_ref(), old);
        let torn = [new[0], new[1], old[2], old[3]];
        replicas(storage, writes_a).0.value.0.update(&torn);
        assert_eq!(*storage.get_ref(), old);
        replicas(storage, writes_a).0.value.0.update(&new);
        assert_eq!(*storage.get_ref(), old);
        replicas(storage, writes_a).0.flag.0.update(&STORAGE_VALID);
        // Both replicas are valid until the old one is invalidated: the first one is read.
        assert_eq!(*storage.get_ref(), if writes_a { new } else { old });
        replicas(storage, writes_a).1.invalidate();
        assert_eq!(*storage.get_ref(), new);
        Ok(())
    }

    // An update interrupted after any of its writes, from the first replica to the second and
    // back, leaves the old or the new value.
    #[test]
    fn atomic_storage_interrupted_update_keeps_old_or_new() {
        let pointer = &raw mut INTERRUPTED;
        let storage = unsafe { (*pointer).get_mut() };
        storage.update(&[1; 4]);
        let a_current = storage.storage_a.0.is_valid();
        check_interrupted_update(storage, [1; 4], [2; 4])?;
        assert_eq!(storage.storage_a.0.is_valid(), !a_current);
        check_interrupted_update(storage, [2; 4], [3; 4])?;
        assert_eq!(storage.storage_a.0.is_valid(), a_current);
        storage.update(&[4; 4]);
        assert_eq!(*storage.get_ref(), [4; 4]);
    }
}
