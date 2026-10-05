//! The tip box shown on a review's first page.
//!
//! A tip box is a touchable strip under the review title. Touching it opens a
//! modal listing `[type, content]` info rows — typically used to explain what
//! the app could not decode, or where the data came from.
//!
//! ```no_run
//! # use ledger_device_sdk::nbgl::{Field, TipBox};
//! let infos = [Field { name: "Contract", value: "0xA0b8...eB48" }];
//! TipBox::new("Why can't this be decoded?", &infos)
//!     .modal_title("Blind signing");
//! ```
//!
//! Only the `INFOS_LIST` content type is available: the union in
//! `nbgl_tipBox_t` has no other member.

use super::*;
use alloc::boxed::Box;

/// A touchable tip box on a review's first page, mapped to `nbgl_tipBox_t`.
pub struct TipBox<'a> {
    text: &'a str,
    icon: Option<&'a NbglGlyph<'a>>,
    modal_title: Option<&'a str>,
    infos: &'a [Field<'a>],
}

impl<'a> TipBox<'a> {
    /// Creates a tip box labelled `text`, opening a modal listing `infos`.
    ///
    /// Each [`Field`]'s `name` is the row's bold type and `value` its content.
    pub fn new(text: &'a str, infos: &'a [Field<'a>]) -> TipBox<'a> {
        TipBox {
            text,
            icon: None,
            modal_title: None,
            infos,
        }
    }

    /// Icon drawn at the start of the tip box.
    pub fn icon(self, icon: &'a NbglGlyph<'a>) -> TipBox<'a> {
        TipBox {
            icon: Some(icon),
            ..self
        }
    }

    /// Title of the modal opened when the tip box is touched.
    pub fn modal_title(self, modal_title: &'a str) -> TipBox<'a> {
        TipBox {
            modal_title: Some(modal_title),
            ..self
        }
    }
}

/// Owns every C-side buffer backing an `nbgl_tipBox_t`.
///
/// NBGL only borrows the struct it is handed, so a value of this type must
/// outlive the use-case call that received it.
pub(crate) struct CTipBox {
    text: CString,
    icon: Option<Box<nbgl_icon_details_t>>,
    modal_title: Option<CString>,
    infos: Box<CInfoList>,
}

impl CTipBox {
    pub(crate) fn new(tip_box: &TipBox) -> CTipBox {
        CTipBox {
            text: CString::new(tip_box.text).unwrap(),
            icon: tip_box.icon.map(|g| Box::new(g.into())),
            modal_title: tip_box.modal_title.map(|s| CString::new(s).unwrap()),
            infos: CInfoList::new(tip_box.infos),
        }
    }

    pub(crate) fn as_c_type(&self) -> nbgl_tipBox_t {
        nbgl_tipBox_t {
            text: self.text.as_ptr(),
            icon: match &self.icon {
                Some(icon) => &**icon as *const nbgl_icon_details_t,
                None => core::ptr::null(),
            },
            modalTitle: match &self.modal_title {
                Some(title) => title.as_ptr(),
                None => core::ptr::null(),
            },
            type_: INFOS_LIST,
            __bindgen_anon_1: nbgl_tipBox_t__bindgen_ty_1 {
                infos: self.infos.as_c_type(),
            },
        }
    }
}
