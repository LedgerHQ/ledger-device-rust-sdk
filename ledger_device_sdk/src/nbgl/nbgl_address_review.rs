//! A wrapper around the asynchronous NBGL [nbgl_useCaseAddressReview](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L4453) C API binding.
//!
//! Draws a flow of pages of an extended address verification page.
//! A back key is available on top-left of the screen, except in first page It is possible to go to next page thanks to "tap to continue".
//! All tag/value pairs are provided in the API and the number of pages is automatically
//! computed, the last page being a long press one
use super::*;

/// A builder to create and show an address review flow.
pub struct NbglAddressReview<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
    review_title: CString,
    review_subtitle: CString,
    /// Owns the C strings and extension structs for the pairs; `None` until
    /// one of the `set_tag_value_list` methods is called.
    tag_value_list: Option<CTagValueList>,
}

impl SyncNBGL for NbglAddressReview<'_> {}

impl<'a> Default for NbglAddressReview<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> NbglAddressReview<'a> {
    /// Creates a new address review flow builder.
    pub fn new() -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_title: CString::default(),
            review_subtitle: CString::default(),
            glyph: None,
            tag_value_list: None,
        }
    }
    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAddressReview<'a> {
        NbglAddressReview {
            glyph: Some(glyph),
            ..self
        }
    }

    #[deprecated(note = "Use `review_title` instead")]
    pub fn verify_str(self, verify_str: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_title: CString::new(verify_str).unwrap(),
            ..self
        }
    }

    /// Sets the title to display at the top of the page.
    /// # Arguments
    /// * `review_title` - The title to display at the top of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn review_title(self, review_title: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_title: CString::new(review_title).unwrap(),
            ..self
        }
    }

    /// Sets the subtitle to display below the title at the top of the page.
    /// # Arguments
    /// * `review_subtitle` - The subtitle to display below the title at the top of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn review_subtitle(self, review_subtitle: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_subtitle: CString::new(review_subtitle).unwrap(),
            ..self
        }
    }

    /// Sets the list of tag/value pairs to display in the address review flow.
    /// # Arguments
    /// * `tag_value_list` - A slice of `Field` representing the tag/value pairs to display.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn set_tag_value_list(self, tag_value_list: &[Field]) -> NbglAddressReview<'a> {
        NbglAddressReview {
            tag_value_list: Some(CTagValueList::from_fields(tag_value_list)),
            ..self
        }
    }

    /// Sets the list of tag/value pairs to display in the address review flow,
    /// allowing each pair to carry a [`FieldExtension`].
    /// # Arguments
    /// * `values` - A slice of `TagValue` representing the tag/value pairs to display.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn set_tag_value_list_ext(self, values: &[TagValue]) -> NbglAddressReview<'a> {
        NbglAddressReview {
            tag_value_list: Some(CTagValueList::new(values)),
            ..self
        }
    }

    fn show_internal(&self, address: &str) -> bool {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            let address = CString::new(address).unwrap();

            let tag_value_list = match &self.tag_value_list {
                Some(list) => list.as_c_list(),
                None => nbgl_contentTagValueList_t::default(),
            };

            self.ux_sync_init();
            nbgl_useCaseAddressReview(
                address.as_ptr(),
                &tag_value_list as *const nbgl_contentTagValueList_t,
                &icon as *const nbgl_icon_details_t,
                match self.review_title.is_empty() {
                    true => core::ptr::null(),
                    false => self.review_title.as_ptr(),
                },
                match self.review_subtitle.is_empty() {
                    true => core::ptr::null(),
                    false => self.review_subtitle.as_ptr(),
                },
                Some(choice_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the address, false otherwise.
            match sync_ret {
                SyncNbgl::UxSyncRetApproved => true,
                SyncNbgl::UxSyncRetRejected => false,
                _ => {
                    panic!("Unexpected return value from ux_sync_addressReview");
                }
            }
        }
    }

    /// Shows the address review flow.
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `address` - The address to review.
    /// # Returns
    /// Returns true if the user approved the address, false otherwise.
    #[cfg(feature = "io_new")]
    pub fn show<const N: usize>(&self, _comm: &mut crate::io::Comm<N>, address: &str) -> bool {
        self.show_internal(address)
    }

    /// Shows the address review flow.
    /// # Arguments
    /// * `address` - The address to review.
    /// # Returns
    /// Returns true if the user approved the address, false otherwise.
    #[cfg(not(feature = "io_new"))]
    pub fn show(&self, address: &str) -> bool {
        self.show_internal(address)
    }
}
