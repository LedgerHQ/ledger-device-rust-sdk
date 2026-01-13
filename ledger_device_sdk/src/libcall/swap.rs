#[cfg(any(
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p",
    feature = "nano_nbgl"
))]
use crate::nbgl::NbglSpinner;
use crate::testing::debug_print;
use ledger_secure_sdk_sys::{
    check_address_parameters_t, create_transaction_parameters_t, get_printable_amount_parameters_t,
    libargs_s__bindgen_ty_1, libargs_t, MAX_PRINTABLE_AMOUNT_SIZE,
};

extern crate alloc;

pub const DEFAULT_COIN_CONFIG_BUF_SIZE: usize = 16;
pub const DEFAULT_ADDRESS_BUF_SIZE: usize = 64;
pub const DEFAULT_ADDRESS_EXTRA_ID_BUF_SIZE: usize = 32;

const DPATH_STAGE_SIZE: usize = 16;
const AMOUNT_BUF_SIZE: usize = 16;

/// Common swap error codes for Exchange integration.
///
/// These error codes are standardized across all Ledger applications to ensure
/// consistent error reporting when called by the Exchange app during swap transactions.
///
/// The upper byte of the 2-byte error code must be one of these values. The lower byte
/// can be set by the application to provide additional refinement.
///
/// This enum matches the C SDK definition in `swap_error_code_helpers.h`.
///
/// # Error Response Format
///
/// When returning an error in swap context, the RAPDU data should begin with:
/// - **Byte 0**: One of these common error codes (upper byte)
/// - **Byte 1**: Application-specific error code (lower byte, can be 0x00)
/// - **Remaining bytes**: Optional error details (messages, field values, etc.)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwapErrorCommonCode {
    /// Internal application error.
    ///
    /// Forward to the Firmware team for analysis.
    ErrorInternal = 0x00,

    /// The amount does not match the one validated in Exchange.
    ///
    /// Use when the transaction amount differs from what the user approved in Exchange.
    ErrorWrongAmount = 0x01,

    /// The destination address does not match the one validated in Exchange.
    ///
    /// Use when the transaction destination differs from the Exchange-validated address.
    ErrorWrongDestination = 0x02,

    /// The fees are different from what was validated in Exchange.
    ///
    /// Use when transaction fees don't match Exchange expectations.
    ErrorWrongFees = 0x03,

    /// The method used is invalid in Exchange context.
    ///
    /// Use when an unsupported transaction method/type is encountered.
    ErrorWrongMethod = 0x04,

    /// The mode used for the cross-chain hash validation is not supported.
    ///
    /// Only relevant for applications that handle cross chain swap, not all applications.
    ErrorCrosschainWrongMode = 0x05,

    /// The method used is invalid in cross-chain Exchange context.
    ///
    /// Only relevant for applications that handle cross chain swap, not all applications.
    ErrorCrosschainWrongMethod = 0x06,

    /// The hash for the cross-chain transaction does not match the validated value.
    ///
    /// Only relevant for applications that handle cross chain swap, not all applications.
    ErrorCrosschainWrongHash = 0x07,

    /// A generic or unspecified error not covered by specific error codes.
    ///
    /// Refer to the remaining bytes of the RAPDU data for further details.
    /// Use this when the error doesn't fit into any of the above categories.
    ErrorGeneric = 0xFF,
}

/// Trait for application-specific swap error codes.
///
/// This trait must be implemented by application-defined error code enums
/// to allow them to be used with [`SwapError`]. The trait ensures the error
/// code can be converted to a u8 for the APDU response.
///
/// # Example
///
/// ```rust,ignore
/// #[repr(u8)]
/// #[derive(Clone, Copy)]
/// pub enum MyAppErrorCode {
///     Default = 0x00,
///     SpecialCase = 0x01,
/// }
///
/// impl SwapAppErrorCodeTrait for MyAppErrorCode {
///     fn as_u8(self) -> u8 {
///         self as u8
///     }
/// }
/// ```
pub trait SwapAppErrorCodeTrait: Copy {
    /// Convert the error code to a u8 byte value.
    fn as_u8(self) -> u8;
}

/// Swap error containing the 2-byte error code and optional descriptive message.
///
/// This structure encapsulates the complete error information for swap failures:
/// - Upper byte: Common error code from [`SwapErrorCommonCode`]
/// - Lower byte: Application-specific error code (must implement [`SwapAppErrorCodeTrait`])
/// - Message: Optional human-readable error description with actual values
///
/// # Usage
///
/// The error bytes should be prepended to the APDU response before the optional message:
/// ```rust,ignore
/// comm.append(&[error.common_code as u8, error.app_code.as_u8()]);
/// if let Some(ref msg) = error.message {
///     comm.append(msg.as_bytes());
/// }
/// comm.reply(sw);
/// ```
///
/// # Generic Parameter
///
/// * `T` - Application-specific error code type implementing [`SwapAppErrorCodeTrait`]
pub struct SwapError<T: SwapAppErrorCodeTrait> {
    /// Common error code (upper byte) from SDK
    pub common_code: SwapErrorCommonCode,
    /// Application-specific error code (lower byte)
    pub app_code: T,
    /// Optional descriptive error message with actual values for debugging
    pub message: Option<alloc::string::String>,
}
impl<T: SwapAppErrorCodeTrait> SwapError<T> {
    /// Create a new SwapError with a formatted message.
    pub fn with_message(
        common_code: SwapErrorCommonCode,
        app_code: T,
        message: alloc::string::String,
    ) -> Self {
        Self {
            common_code,
            app_code,
            message: Some(message),
        }
    }

    /// Create a new SwapError without a message.
    pub fn without_message(common_code: SwapErrorCommonCode, app_code: T) -> Self {
        Self {
            common_code,
            app_code,
            message: None,
        }
    }
}
/// Helper function to read a null-terminated C string into a fixed-size buffer
/// Returns the buffer and the actual length read
/// Prints a warning if truncation occurs
fn read_c_string<const N: usize>(ptr: *const i8) -> ([u8; N], usize) {
    let mut buffer = [0u8; N];

    if ptr.is_null() {
        return (buffer, 0);
    }

    let mut length = 0usize;
    let mut c = unsafe { *ptr.add(length) };

    while c != '\0' as i8 && length < N {
        buffer[length] = c as u8;
        length += 1;
        c = unsafe { *ptr.add(length) };
    }

    // Check if truncation occurred
    if c != '\0' as i8 && length == N {
        debug_print("WARNING: C string truncated\n");
    }

    (buffer, length)
}

pub struct CheckAddressParams<
    const COIN_CONFIG_BUF_SIZE: usize = DEFAULT_COIN_CONFIG_BUF_SIZE,
    const ADDRESS_BUF_SIZE: usize = DEFAULT_ADDRESS_BUF_SIZE,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize = DEFAULT_ADDRESS_EXTRA_ID_BUF_SIZE,
> {
    pub coin_config: [u8; COIN_CONFIG_BUF_SIZE],
    pub coin_config_len: usize,
    pub dpath: [u8; DPATH_STAGE_SIZE * 4],
    pub dpath_len: usize,
    pub ref_address: [u8; ADDRESS_BUF_SIZE],
    pub ref_address_len: usize,
    pub result: *mut i32,
}

impl<
        const COIN_CONFIG_BUF_SIZE: usize,
        const ADDRESS_BUF_SIZE: usize,
        const ADDRESS_EXTRA_ID_BUF_SIZE: usize,
    > Default
    for CheckAddressParams<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE>
{
    fn default() -> Self {
        CheckAddressParams {
            coin_config: [0; COIN_CONFIG_BUF_SIZE],
            coin_config_len: 0,
            dpath: [0; DPATH_STAGE_SIZE * 4],
            dpath_len: 0,
            ref_address: [0; ADDRESS_BUF_SIZE],
            ref_address_len: 0,
            result: core::ptr::null_mut(),
        }
    }
}

pub struct PrintableAmountParams<
    const COIN_CONFIG_BUF_SIZE: usize = DEFAULT_COIN_CONFIG_BUF_SIZE,
    // Unused const generic parameter here, to allow type inference in `swap_return` fn
    const ADDRESS_BUF_SIZE: usize = DEFAULT_ADDRESS_BUF_SIZE,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize = DEFAULT_ADDRESS_EXTRA_ID_BUF_SIZE,
> {
    pub coin_config: [u8; COIN_CONFIG_BUF_SIZE],
    pub coin_config_len: usize,
    pub amount: [u8; AMOUNT_BUF_SIZE],
    pub amount_len: usize,
    pub amount_str: *mut i8,
    pub is_fee: bool,
}

impl<
        const COIN_CONFIG_BUF_SIZE: usize,
        const ADDRESS_BUF_SIZE: usize,
        const ADDRESS_EXTRA_ID_BUF_SIZE: usize,
    > Default
    for PrintableAmountParams<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE>
{
    fn default() -> Self {
        PrintableAmountParams {
            coin_config: [0; COIN_CONFIG_BUF_SIZE],
            coin_config_len: 0,
            amount: [0; AMOUNT_BUF_SIZE],
            amount_len: 0,
            amount_str: core::ptr::null_mut(),
            is_fee: false,
        }
    }
}

pub struct CreateTxParams<
    const COIN_CONFIG_BUF_SIZE: usize = DEFAULT_COIN_CONFIG_BUF_SIZE,
    const ADDRESS_BUF_SIZE: usize = DEFAULT_ADDRESS_BUF_SIZE,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize = DEFAULT_ADDRESS_EXTRA_ID_BUF_SIZE,
> {
    pub coin_config: [u8; COIN_CONFIG_BUF_SIZE],
    pub coin_config_len: usize,
    pub amount: [u8; AMOUNT_BUF_SIZE],
    pub amount_len: usize,
    pub fee_amount: [u8; AMOUNT_BUF_SIZE],
    pub fee_amount_len: usize,
    pub dest_address: [u8; ADDRESS_BUF_SIZE],
    pub dest_address_len: usize,
    pub dest_address_extra_id: [u8; ADDRESS_EXTRA_ID_BUF_SIZE],
    pub dest_address_extra_id_len: usize,
    pub result: *mut u8,
}

impl<
        const COIN_CONFIG_BUF_SIZE: usize,
        const ADDRESS_BUF_SIZE: usize,
        const ADDRESS_EXTRA_ID_BUF_SIZE: usize,
    > Default
    for CreateTxParams<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE>
{
    fn default() -> Self {
        CreateTxParams {
            coin_config: [0; COIN_CONFIG_BUF_SIZE],
            coin_config_len: 0,
            amount: [0; AMOUNT_BUF_SIZE],
            amount_len: 0,
            fee_amount: [0; AMOUNT_BUF_SIZE],
            fee_amount_len: 0,
            dest_address: [0; ADDRESS_BUF_SIZE],
            dest_address_len: 0,
            dest_address_extra_id: [0; ADDRESS_EXTRA_ID_BUF_SIZE],
            dest_address_extra_id_len: 0,
            result: core::ptr::null_mut(),
        }
    }
}

pub fn get_check_address_params<
    const COIN_CONFIG_BUF_SIZE: usize,
    const ADDRESS_BUF_SIZE: usize,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize,
>(
    arg0: u32,
) -> CheckAddressParams<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE> {
    debug_print("=> get_check_address_params\n");

    let mut libarg: libargs_t = libargs_t::default();

    let arg = arg0 as *const u32;

    libarg.id = unsafe { *arg };
    libarg.command = unsafe { *arg.add(1) };
    libarg.unused = unsafe { *arg.add(2) };

    libarg.__bindgen_anon_1 = unsafe { *(arg.add(3) as *const libargs_s__bindgen_ty_1) };

    let params: check_address_parameters_t =
        unsafe { *(libarg.__bindgen_anon_1.check_address as *const check_address_parameters_t) };

    let mut check_address_params: CheckAddressParams<
        COIN_CONFIG_BUF_SIZE,
        ADDRESS_BUF_SIZE,
        ADDRESS_EXTRA_ID_BUF_SIZE,
    > = Default::default();

    debug_print("==> GET_COIN_CONFIG_LENGTH\n");
    check_address_params.coin_config_len = params.coin_configuration_length as usize;

    debug_print("==> GET_COIN_CONFIG \n");
    unsafe {
        params.coin_configuration.copy_to_nonoverlapping(
            check_address_params.coin_config.as_mut_ptr(),
            check_address_params
                .coin_config_len
                .min(COIN_CONFIG_BUF_SIZE),
        );
    }

    debug_print("==> GET_DPATH_LENGTH\n");
    check_address_params.dpath_len =
        DPATH_STAGE_SIZE.min(unsafe { *(params.address_parameters as *const u8) as usize });

    debug_print("==> GET_DPATH \n");
    for i in 1..1 + check_address_params.dpath_len * 4 {
        check_address_params.dpath[i - 1] = unsafe { *(params.address_parameters.add(i)) };
    }

    debug_print("==> GET_REF_ADDRESS\n");
    let (address, address_len) =
        read_c_string::<ADDRESS_BUF_SIZE>(params.address_to_check as *const i8);
    check_address_params.ref_address = address;
    check_address_params.ref_address_len = address_len;

    check_address_params.result = unsafe {
        &(*(libarg.__bindgen_anon_1.check_address as *mut check_address_parameters_t)).result
            as *const i32 as *mut i32
    };

    check_address_params
}

pub fn get_printable_amount_params<
    const COIN_CONFIG_BUF_SIZE: usize,
    const ADDRESS_BUF_SIZE: usize,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize,
>(
    arg0: u32,
) -> PrintableAmountParams<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE> {
    debug_print("=> get_printable_amount_params\n");

    let mut libarg: libargs_t = libargs_t::default();

    let arg = arg0 as *const u32;

    libarg.id = unsafe { *arg };
    libarg.command = unsafe { *arg.add(1) };
    libarg.unused = unsafe { *arg.add(2) };

    libarg.__bindgen_anon_1 = unsafe { *(arg.add(3) as *const libargs_s__bindgen_ty_1) };

    let params: get_printable_amount_parameters_t = unsafe {
        *(libarg.__bindgen_anon_1.get_printable_amount as *const get_printable_amount_parameters_t)
    };

    let mut printable_amount_params: PrintableAmountParams<
        COIN_CONFIG_BUF_SIZE,
        ADDRESS_BUF_SIZE,
        ADDRESS_EXTRA_ID_BUF_SIZE,
    > = Default::default();

    debug_print("==> GET_COIN_CONFIG_LENGTH\n");
    printable_amount_params.coin_config_len = params.coin_configuration_length as usize;

    debug_print("==> GET_COIN_CONFIG \n");
    unsafe {
        params.coin_configuration.copy_to_nonoverlapping(
            printable_amount_params.coin_config.as_mut_ptr(),
            printable_amount_params
                .coin_config_len
                .min(COIN_CONFIG_BUF_SIZE),
        );
    }

    debug_print("==> GET_IS_FEE\n");
    printable_amount_params.is_fee = params.is_fee == true;

    debug_print("==> GET_AMOUNT_LENGTH\n");
    printable_amount_params.amount_len = AMOUNT_BUF_SIZE.min(params.amount_length as usize);

    debug_print("==> GET_AMOUNT\n");
    for i in 0..printable_amount_params.amount_len {
        printable_amount_params.amount[AMOUNT_BUF_SIZE - printable_amount_params.amount_len + i] =
            unsafe { *(params.amount.add(i)) };
    }

    debug_print("==> GET_AMOUNT_STR\n");
    printable_amount_params.amount_str = unsafe {
        &(*(libarg.__bindgen_anon_1.get_printable_amount as *mut get_printable_amount_parameters_t))
            .printable_amount as *const core::ffi::c_char as *mut i8
    };

    printable_amount_params
}

extern "C" {
    fn c_reset_bss();
    fn c_boot_std();
}

pub fn sign_tx_params<
    const COIN_CONFIG_BUF_SIZE: usize,
    const ADDRESS_BUF_SIZE: usize,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize,
>(
    arg0: u32,
) -> CreateTxParams<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE> {
    debug_print("=> sign_tx_params\n");

    let mut libarg: libargs_t = libargs_t::default();

    let arg = arg0 as *const u32;

    libarg.id = unsafe { *arg };
    libarg.command = unsafe { *arg.add(1) };
    libarg.unused = unsafe { *arg.add(2) };

    libarg.__bindgen_anon_1 = unsafe { *(arg.add(3) as *const libargs_s__bindgen_ty_1) };

    let params: create_transaction_parameters_t = unsafe {
        *(libarg.__bindgen_anon_1.create_transaction as *const create_transaction_parameters_t)
    };

    let mut create_tx_params: CreateTxParams<
        COIN_CONFIG_BUF_SIZE,
        ADDRESS_BUF_SIZE,
        ADDRESS_EXTRA_ID_BUF_SIZE,
    > = Default::default();

    debug_print("==> GET_COIN_CONFIG_LENGTH\n");
    create_tx_params.coin_config_len = params.coin_configuration_length as usize;

    debug_print("==> GET_COIN_CONFIG \n");
    unsafe {
        params.coin_configuration.copy_to_nonoverlapping(
            create_tx_params.coin_config.as_mut_ptr(),
            create_tx_params.coin_config_len.min(COIN_CONFIG_BUF_SIZE),
        );
    }

    debug_print("==> GET_AMOUNT\n");
    create_tx_params.amount_len = AMOUNT_BUF_SIZE.min(params.amount_length as usize);
    for i in 0..create_tx_params.amount_len {
        create_tx_params.amount[AMOUNT_BUF_SIZE - create_tx_params.amount_len + i] =
            unsafe { *(params.amount.add(i)) };
    }

    debug_print("==> GET_FEE\n");
    create_tx_params.fee_amount_len = AMOUNT_BUF_SIZE.min(params.fee_amount_length as usize);
    for i in 0..create_tx_params.fee_amount_len {
        create_tx_params.fee_amount[AMOUNT_BUF_SIZE - create_tx_params.fee_amount_len + i] =
            unsafe { *(params.fee_amount.add(i)) };
    }

    debug_print("==> GET_DESTINATION_ADDRESS\n");
    let (address, address_len) =
        read_c_string::<ADDRESS_BUF_SIZE>(params.destination_address as *const i8);
    create_tx_params.dest_address = address;
    create_tx_params.dest_address_len = address_len;

    debug_print("==> GET_DESTINATION_ADDRESS_EXTRA_ID\n");
    let (extra_id, extra_id_len) = read_c_string::<ADDRESS_EXTRA_ID_BUF_SIZE>(
        params.destination_address_extra_id as *const i8,
    );
    create_tx_params.dest_address_extra_id = extra_id;
    create_tx_params.dest_address_extra_id_len = extra_id_len;

    create_tx_params.result = unsafe {
        &(*(libarg.__bindgen_anon_1.create_transaction as *mut create_transaction_parameters_t))
            .result as *const u8 as *mut u8
    };

    /* Reset BSS and complete application boot */
    unsafe {
        c_reset_bss();
        c_boot_std();
    }

    #[cfg(any(
        target_os = "stax",
        target_os = "flex",
        target_os = "apex_p",
        feature = "nano_nbgl"
    ))]
    NbglSpinner::new().show("Signing");

    create_tx_params
}

pub enum SwapResult<
    'a,
    const COIN_CONFIG_BUF_SIZE: usize = DEFAULT_COIN_CONFIG_BUF_SIZE,
    const ADDRESS_BUF_SIZE: usize = DEFAULT_ADDRESS_BUF_SIZE,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize = DEFAULT_ADDRESS_EXTRA_ID_BUF_SIZE,
> {
    CheckAddressResult(
        &'a mut CheckAddressParams<
            COIN_CONFIG_BUF_SIZE,
            ADDRESS_BUF_SIZE,
            ADDRESS_EXTRA_ID_BUF_SIZE,
        >,
        i32,
    ),
    PrintableAmountResult(
        &'a mut PrintableAmountParams<
            COIN_CONFIG_BUF_SIZE,
            ADDRESS_BUF_SIZE,
            ADDRESS_EXTRA_ID_BUF_SIZE,
        >,
        &'a str,
    ),
    CreateTxResult(
        &'a mut CreateTxParams<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE>,
        u8,
    ),
}

pub fn swap_return<
    const COIN_CONFIG_BUF_SIZE: usize,
    const ADDRESS_BUF_SIZE: usize,
    const ADDRESS_EXTRA_ID_BUF_SIZE: usize,
>(
    res: SwapResult<COIN_CONFIG_BUF_SIZE, ADDRESS_BUF_SIZE, ADDRESS_EXTRA_ID_BUF_SIZE>,
) {
    match res {
        SwapResult::CheckAddressResult(&mut ref p, r) => {
            unsafe { *(p.result) = r };
        }
        SwapResult::PrintableAmountResult(&mut ref p, s) => {
            if s.len() < (MAX_PRINTABLE_AMOUNT_SIZE - 1).try_into().unwrap() {
                for (i, c) in s.chars().enumerate() {
                    unsafe { *(p.amount_str.add(i)) = c as i8 };
                }
                unsafe { *(p.amount_str.add(s.len())) = '\0' as i8 };
            } else {
                unsafe { *(p.amount_str) = '\0' as i8 };
            }
        }
        SwapResult::CreateTxResult(&mut ref p, r) => {
            unsafe { *(p.result) = r };
        }
    }
    unsafe { ledger_secure_sdk_sys::os_lib_end() };
}
