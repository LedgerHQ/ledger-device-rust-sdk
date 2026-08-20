//! Tag/value pairs carrying an optional value extension (alias).
//!
//! A review displays a list of `[tag, value]` pairs. When a value is too long
//! to fit, or is a human-readable stand-in for something else (an ENS name for
//! an address, an address book entry, an address better shown as a QR code),
//! the C API lets the pair carry a [`nbgl_contentValueExt_t`] describing how to
//! expand it. NBGL then draws a `>` affordance next to the value and opens a
//! modal with the full content when the user taps it.
//!
//! [`Field`] stays the plain `{name, value}` pair it has always been.
//! [`TagValue`] is the richer variant that can carry a [`FieldExtension`], and
//! any `Field` converts into one, so the two can be mixed freely:
//!
//! ```no_run
//! # use ledger_device_sdk::nbgl::{Field, FieldExtension, TagValue};
//! let pairs = [
//!     TagValue::from(&Field { name: "Amount", value: "1.5 ETH" }),
//!     TagValue {
//!         name: "To",
//!         value: "vitalik.eth",
//!         extension: Some(FieldExtension::ens(
//!             "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
//!         )),
//!     },
//! ];
//! ```

use super::*;
use alloc::boxed::Box;

/// Origin of a value alias, mapped to `nbgl_contentValueAliasType_t`.
///
/// This decides what NBGL draws in the modal opened from the `>` affordance.
/// You rarely name it directly — the [`FieldExtension`] constructors each pick
/// the matching variant.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum AliasType {
    /// Plain full-value expansion, with an optional explanation in gray.
    #[default]
    None,
    /// The value is an ENS name. NBGL adds its own provenance note
    /// ("ENS names are resolved by Ledger backend.").
    Ens,
    /// The value comes from the Address Book.
    AddressBook,
    /// The value is an address to be displayed as a QR code.
    QrCode,
    /// The value expands into a list of `[type, content]` info rows.
    InfoList,
    /// The value expands into a nested tag/value list.
    TagValueList,
}

impl AliasType {
    fn to_c_type(self) -> nbgl_contentValueAliasType_t {
        match self {
            AliasType::None => NO_ALIAS_TYPE,
            AliasType::Ens => ENS_ALIAS,
            AliasType::AddressBook => ADDRESS_BOOK_ALIAS,
            AliasType::QrCode => QR_CODE_ALIAS,
            AliasType::InfoList => INFO_LIST_ALIAS,
            AliasType::TagValueList => TAG_VALUE_LIST_ALIAS,
        }
    }
}

/// Content of an alias that expands into a nested list.
#[derive(Copy, Clone)]
enum NestedContent<'a> {
    InfoList(&'a [Field<'a>]),
    TagValueList(&'a [Field<'a>]),
}

/// Additional information attached to a [`TagValue`], letting the user expand a
/// shortened or symbolic value into its full form.
///
/// Build one with a constructor naming the alias kind, then chain the optional
/// setters:
///
/// ```no_run
/// # use ledger_device_sdk::nbgl::FieldExtension;
/// // An address shown as a QR code, titled in the modal.
/// FieldExtension::qr_code("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq")
///     .title("Receive address");
///
/// // An address book entry, with the saved name under the alias.
/// FieldExtension::address_book("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
///     .alias_sub_name("Hardware wallet");
/// ```
pub struct FieldExtension<'a> {
    full_value: &'a str,
    alias_sub_name: Option<&'a str>,
    explanation: Option<&'a str>,
    title: Option<&'a str>,
    back_text: Option<&'a str>,
    nested: Option<NestedContent<'a>>,
    alias_type: AliasType,
}

impl<'a> FieldExtension<'a> {
    fn with_type(full_value: &'a str, alias_type: AliasType) -> FieldExtension<'a> {
        FieldExtension {
            full_value,
            alias_sub_name: None,
            explanation: None,
            title: None,
            back_text: None,
            nested: None,
            alias_type,
        }
    }

    /// Expands into the untruncated value, with no particular provenance.
    /// Pair with [`Self::explanation`] to caption it.
    pub fn full_value(full_value: &'a str) -> FieldExtension<'a> {
        Self::with_type(full_value, AliasType::None)
    }

    /// The displayed value is an ENS name resolving to `full_value`.
    /// NBGL supplies its own explanation for this kind, so
    /// [`Self::explanation`] is ignored.
    pub fn ens(full_value: &'a str) -> FieldExtension<'a> {
        Self::with_type(full_value, AliasType::Ens)
    }

    /// The displayed value names an Address Book entry for `full_value`.
    /// [`Self::alias_sub_name`] gives the saved name; [`Self::explanation`]
    /// carries a trusted name, shown in addition rather than instead.
    pub fn address_book(full_value: &'a str) -> FieldExtension<'a> {
        Self::with_type(full_value, AliasType::AddressBook)
    }

    /// The value is an address to render as a QR code. [`Self::title`] captions
    /// the code (defaulting to the value itself) and [`Self::explanation`] is
    /// drawn underneath.
    ///
    /// Requires the C SDK to be built with `NBGL_QRCODE`; without it NBGL
    /// draws an empty modal.
    pub fn qr_code(value: &'a str) -> FieldExtension<'a> {
        Self::with_type(value, AliasType::QrCode)
    }

    /// The value expands into a list of info rows, each `Field`'s `name` used
    /// as the bold type and `value` as its content.
    pub fn info_list(infos: &'a [Field<'a>]) -> FieldExtension<'a> {
        FieldExtension {
            nested: Some(NestedContent::InfoList(infos)),
            ..Self::with_type("", AliasType::InfoList)
        }
    }

    /// The value expands into a nested tag/value list. The nested pairs are
    /// plain — they cannot themselves carry extensions.
    pub fn tag_value_list(pairs: &'a [Field<'a>]) -> FieldExtension<'a> {
        FieldExtension {
            nested: Some(NestedContent::TagValueList(pairs)),
            ..Self::with_type("", AliasType::TagValueList)
        }
    }

    /// Sets the name shown under the alias and in the details view.
    pub fn alias_sub_name(self, alias_sub_name: &'a str) -> FieldExtension<'a> {
        FieldExtension {
            alias_sub_name: Some(alias_sub_name),
            ..self
        }
    }

    /// Sets the gray explanatory text describing where the alias comes from.
    pub fn explanation(self, explanation: &'a str) -> FieldExtension<'a> {
        FieldExtension {
            explanation: Some(explanation),
            ..self
        }
    }

    /// Sets the QR code caption. Only used when the alias is a
    /// [`AliasType::QrCode`].
    pub fn title(self, title: &'a str) -> FieldExtension<'a> {
        FieldExtension {
            title: Some(title),
            ..self
        }
    }

    /// Sets the title of the modal opened by the alias. Defaults to the tag
    /// name of the pair carrying this extension.
    pub fn back_text(self, back_text: &'a str) -> FieldExtension<'a> {
        FieldExtension {
            back_text: Some(back_text),
            ..self
        }
    }

    /// Returns the kind of alias this extension describes.
    pub fn alias_type(&self) -> AliasType {
        self.alias_type
    }
}

/// A `[tag, value]` pair that may carry a [`FieldExtension`].
///
/// This is [`Field`] plus the optional extension. Use `TagValue::from(&field)`
/// (or `.into()`) to lift a plain `Field`.
pub struct TagValue<'a> {
    /// Tag name, displayed in bold.
    pub name: &'a str,
    /// Value, displayed next to or below the tag.
    pub value: &'a str,
    /// When set, NBGL draws a `>` affordance opening a modal built from it.
    pub extension: Option<FieldExtension<'a>>,
}

impl<'a> From<&Field<'a>> for TagValue<'a> {
    fn from(field: &Field<'a>) -> TagValue<'a> {
        TagValue {
            name: field.name,
            value: field.value,
            extension: None,
        }
    }
}

/// Lifts a slice of plain [`Field`]s into [`TagValue`]s carrying no extension.
///
/// Lets the `show`/`show_ext` pairs on each use case share a single
/// implementation instead of duplicating the C plumbing.
pub(crate) fn to_tag_values<'a>(fields: &'a [Field<'a>]) -> Vec<TagValue<'a>> {
    fields.iter().map(|f| f.into()).collect()
}

/// Owned C strings and pointer arrays backing an info list.
///
/// Boxed so its address is stable once something points at `list`. Shared with
/// the tip box, which carries the same `nbgl_contentInfoList_t`.
pub(crate) struct CInfoList {
    _types: Vec<CString>,
    _contents: Vec<CString>,
    _types_ptr: Vec<*const c_char>,
    _contents_ptr: Vec<*const c_char>,
    list: nbgl_contentInfoList_t,
}

impl CInfoList {
    pub(crate) fn new(infos: &[Field]) -> Box<CInfoList> {
        let types: Vec<CString> = infos
            .iter()
            .map(|f| CString::new(f.name).unwrap())
            .collect();
        let contents: Vec<CString> = infos
            .iter()
            .map(|f| CString::new(f.value).unwrap())
            .collect();
        let types_ptr: Vec<*const c_char> = types.iter().map(|s| s.as_ptr()).collect();
        let contents_ptr: Vec<*const c_char> = contents.iter().map(|s| s.as_ptr()).collect();

        // Read the buffer addresses before moving the vectors into the box:
        // moving a Vec leaves its heap allocation in place, so these stay valid.
        let list = nbgl_contentInfoList_t {
            infoTypes: types_ptr.as_ptr(),
            infoContents: contents_ptr.as_ptr(),
            nbInfos: types_ptr.len() as u8,
            ..Default::default()
        };

        Box::new(CInfoList {
            _types: types,
            _contents: contents,
            _types_ptr: types_ptr,
            _contents_ptr: contents_ptr,
            list,
        })
    }

    /// The `nbgl_contentInfoList_t` borrowing this list's buffers.
    pub(crate) fn as_c_type(&self) -> nbgl_contentInfoList_t {
        self.list
    }
}

/// Owned C strings and pair array backing a nested tag/value list.
struct CNestedTagValueList {
    _cfields: Vec<CField>,
    _pairs: Vec<nbgl_contentTagValue_t>,
    list: nbgl_contentTagValueList_t,
}

impl CNestedTagValueList {
    fn new(fields: &[Field]) -> Box<CNestedTagValueList> {
        let cfields: Vec<CField> = fields.iter().map(|f| f.into()).collect();
        let pairs: Vec<nbgl_contentTagValue_t> = cfields.iter().map(|f| f.into()).collect();

        let list = nbgl_contentTagValueList_t {
            pairs: pairs.as_ptr(),
            nbPairs: pairs.len() as u8,
            ..Default::default()
        };

        Box::new(CNestedTagValueList {
            _cfields: cfields,
            _pairs: pairs,
            list,
        })
    }
}

/// Owned C strings backing one `nbgl_contentValueExt_t`.
struct CFieldExtension {
    full_value: CString,
    alias_sub_name: Option<CString>,
    explanation: Option<CString>,
    title: Option<CString>,
    back_text: Option<CString>,
    nested_infos: Option<Box<CInfoList>>,
    nested_pairs: Option<Box<CNestedTagValueList>>,
    alias_type: AliasType,
}

fn opt_cstring(s: Option<&str>) -> Option<CString> {
    s.map(|s| CString::new(s).unwrap())
}

fn opt_ptr(s: &Option<CString>) -> *const c_char {
    match s {
        Some(s) => s.as_ptr(),
        None => core::ptr::null(),
    }
}

impl CFieldExtension {
    fn new(ext: &FieldExtension) -> CFieldExtension {
        let (nested_infos, nested_pairs) = match ext.nested {
            Some(NestedContent::InfoList(infos)) => (Some(CInfoList::new(infos)), None),
            Some(NestedContent::TagValueList(pairs)) => {
                (None, Some(CNestedTagValueList::new(pairs)))
            }
            None => (None, None),
        };

        CFieldExtension {
            full_value: CString::new(ext.full_value).unwrap(),
            alias_sub_name: opt_cstring(ext.alias_sub_name),
            explanation: opt_cstring(ext.explanation),
            title: opt_cstring(ext.title),
            back_text: opt_cstring(ext.back_text),
            nested_infos,
            nested_pairs,
            alias_type: ext.alias_type,
        }
    }

    fn to_c_type(&self) -> nbgl_contentValueExt_t {
        // The union is only read for the two list alias types; a null pointer
        // is the correct zero value for every other kind.
        let anon = match (&self.nested_infos, &self.nested_pairs) {
            (Some(infos), _) => nbgl_contentValueExt_t__bindgen_ty_1 {
                infolist: &infos.list as *const nbgl_contentInfoList_t,
            },
            (_, Some(pairs)) => nbgl_contentValueExt_t__bindgen_ty_1 {
                tagValuelist: &pairs.list as *const nbgl_contentTagValueList_t,
            },
            _ => nbgl_contentValueExt_t__bindgen_ty_1 {
                infolist: core::ptr::null(),
            },
        };

        nbgl_contentValueExt_t {
            fullValue: self.full_value.as_ptr(),
            aliasSubName: opt_ptr(&self.alias_sub_name),
            explanation: opt_ptr(&self.explanation),
            title: opt_ptr(&self.title),
            backText: opt_ptr(&self.back_text),
            __bindgen_anon_1: anon,
            aliasType: self.alias_type.to_c_type(),
        }
    }
}

/// Owns every C-side buffer backing a list of tag/value pairs.
///
/// NBGL only borrows the array it is handed, so a value of this type must stay
/// alive for as long as the use case that received it is on screen.
pub(crate) struct CTagValueList {
    _cfields: Vec<CField>,
    _cexts: Vec<CFieldExtension>,
    _exts: Vec<nbgl_contentValueExt_t>,
    pairs: Vec<nbgl_contentTagValue_t>,
}

impl CTagValueList {
    pub(crate) fn new(values: &[TagValue]) -> CTagValueList {
        let cfields: Vec<CField> = values
            .iter()
            .map(|tv| CField {
                name: CString::new(tv.name).unwrap(),
                value: CString::new(tv.value).unwrap(),
            })
            .collect();

        // Stage every extension before building the pairs: `exts` must be
        // complete and final before any pair stores a pointer into it, since a
        // later push could reallocate and leave those pointers dangling.
        let mut cexts: Vec<CFieldExtension> = Vec::new();
        let mut ext_of_pair: Vec<Option<usize>> = Vec::with_capacity(values.len());
        for tv in values.iter() {
            match tv.extension {
                Some(ref ext) => {
                    ext_of_pair.push(Some(cexts.len()));
                    cexts.push(CFieldExtension::new(ext));
                }
                None => ext_of_pair.push(None),
            }
        }
        let exts: Vec<nbgl_contentValueExt_t> = cexts.iter().map(|ext| ext.to_c_type()).collect();

        let pairs: Vec<nbgl_contentTagValue_t> = cfields
            .iter()
            .zip(ext_of_pair.iter())
            .map(|(field, ext_idx)| {
                let mut pair: nbgl_contentTagValue_t = field.into();
                if let Some(idx) = *ext_idx {
                    pair.__bindgen_anon_1.extension = &exts[idx] as *const nbgl_contentValueExt_t;
                    // `extension` shares storage with `valueIcon`; C only reads
                    // it as an extension when this bit is set, so setting the
                    // pointer without the bit would have NBGL treat it as an
                    // icon.
                    pair.set_aliasValue(1);
                }
                pair
            })
            .collect();

        CTagValueList {
            _cfields: cfields,
            _cexts: cexts,
            _exts: exts,
            pairs,
        }
    }

    /// Builds the list from plain fields, none of which carry an extension.
    pub(crate) fn from_fields(fields: &[Field]) -> CTagValueList {
        CTagValueList::new(&to_tag_values(fields))
    }

    pub(crate) fn pairs_ptr(&self) -> *const nbgl_contentTagValue_t {
        self.pairs.as_ptr()
    }

    pub(crate) fn len(&self) -> u8 {
        self.pairs.len() as u8
    }

    /// A `nbgl_contentTagValueList_t` borrowing this list's pairs.
    pub(crate) fn as_c_list(&self) -> nbgl_contentTagValueList_t {
        nbgl_contentTagValueList_t {
            pairs: self.pairs_ptr(),
            nbPairs: self.len(),
            ..Default::default()
        }
    }
}
