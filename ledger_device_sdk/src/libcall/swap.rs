#[cfg(any(target_os = "stax", target_os = "flex"))]
use crate::nbgl::NbglSpinner;
use crate::testing::debug_print;
use ledger_secure_sdk_sys::{
    check_address_parameters_t, create_transaction_parameters_t, get_printable_amount_parameters_t,
    libargs_s__bindgen_ty_1, libargs_t, MAX_PRINTABLE_AMOUNT_SIZE,
};

const DPATH_STAGE_SIZE: usize = 16;
const ADDRESS_BUF_SIZE: usize = 64;
const AMOUNT_BUF_SIZE: usize = 16;

pub struct CheckAddressParams {
    pub dpath: [u8; DPATH_STAGE_SIZE * 4],
    pub dpath_len: usize,
    pub ref_address: [u8; ADDRESS_BUF_SIZE],
    pub ref_address_len: usize,
    pub result: *mut i32,
}

impl Default for CheckAddressParams {
    fn default() -> Self {
        CheckAddressParams {
            dpath: [0; DPATH_STAGE_SIZE * 4],
            dpath_len: 0,
            ref_address: [0; ADDRESS_BUF_SIZE],
            ref_address_len: 0,
            result: core::ptr::null_mut(),
        }
    }
}

pub struct PrintableAmountParams {
    pub amount: [u8; AMOUNT_BUF_SIZE],
    pub amount_len: usize,
    pub amount_str: *mut i8,
    pub is_fee: bool,
}

impl Default for PrintableAmountParams {
    fn default() -> Self {
        PrintableAmountParams {
            amount: [0; AMOUNT_BUF_SIZE],
            amount_len: 0,
            amount_str: core::ptr::null_mut(),
            is_fee: false,
        }
    }
}

pub struct CreateTxParams {
    pub amount: [u8; AMOUNT_BUF_SIZE],
    pub amount_len: usize,
    pub fee_amount: [u8; AMOUNT_BUF_SIZE],
    pub fee_amount_len: usize,
    pub dest_address: [u8; ADDRESS_BUF_SIZE],
    pub dest_address_len: usize,
    pub result: *mut u8,
}

impl Default for CreateTxParams {
    fn default() -> Self {
        CreateTxParams {
            amount: [0; AMOUNT_BUF_SIZE],
            amount_len: 0,
            fee_amount: [0; AMOUNT_BUF_SIZE],
            fee_amount_len: 0,
            dest_address: [0; ADDRESS_BUF_SIZE],
            dest_address_len: 0,
            result: core::ptr::null_mut(),
        }
    }
}

pub fn get_check_address_params(arg0: u32) -> CheckAddressParams {
    debug_print("=> get_check_address_params\n");

    let mut libarg: libargs_t = libargs_t::default();

    let arg = arg0 as *const u32;

    libarg.id = unsafe { *arg };
    libarg.command = unsafe { *arg.add(1) };
    libarg.unused = unsafe { *arg.add(2) };

    libarg.__bindgen_anon_1 = unsafe { *(arg.add(3) as *const libargs_s__bindgen_ty_1) };

    let params: check_address_parameters_t =
        unsafe { *(libarg.__bindgen_anon_1.check_address as *const check_address_parameters_t) };

    let mut check_address_params: CheckAddressParams = Default::default();

    debug_print("==> GET_DPATH_LENGTH\n");
    check_address_params.dpath_len =
        DPATH_STAGE_SIZE.min(unsafe { *(params.address_parameters as *const u8) as usize });

    debug_print("==> GET_DPATH \n");
    for i in 1..1 + check_address_params.dpath_len * 4 {
        check_address_params.dpath[i - 1] = unsafe { *(params.address_parameters.add(i)) };
    }

    debug_print("==> GET_REF_ADDRESS\n");
    let mut address_length = 0usize;
    let mut c = unsafe { *(params.address_to_check.add(address_length)) };
    while c != '\0' as i8 && address_length < ADDRESS_BUF_SIZE {
        check_address_params.ref_address[address_length] = c as u8;
        address_length += 1;
        c = unsafe { *(params.address_to_check.add(address_length)) };
    }
    check_address_params.ref_address_len = address_length;

    check_address_params.result = unsafe {
        &(*(libarg.__bindgen_anon_1.check_address as *mut check_address_parameters_t)).result
            as *const i32 as *mut i32
    };

    check_address_params
}

pub fn get_printable_amount_params(arg0: u32) -> PrintableAmountParams {
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

    let mut printable_amount_params: PrintableAmountParams = Default::default();

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
            .printable_amount as *const i8 as *mut i8
    };

    printable_amount_params
}

extern "C" {
    fn c_reset_bss();
    fn c_boot_std();
}

pub fn sign_tx_params(arg0: u32) -> CreateTxParams {
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

    let mut create_tx_params: CreateTxParams = Default::default();

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
    let mut dest_address_length = 0usize;
    let mut c = unsafe { *params.destination_address.add(dest_address_length) };
    while c != '\0' as i8 && dest_address_length < ADDRESS_BUF_SIZE {
        create_tx_params.dest_address[dest_address_length] = c as u8;
        dest_address_length += 1;
        c = unsafe { *params.destination_address.add(dest_address_length) };
    }
    create_tx_params.dest_address_len = dest_address_length;

    create_tx_params.result = unsafe {
        &(*(libarg.__bindgen_anon_1.create_transaction as *mut create_transaction_parameters_t))
            .result as *const u8 as *mut u8
    };

    /* Reset BSS and complete application boot */
    unsafe {
        c_reset_bss();
        c_boot_std();
    }

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    NbglSpinner::new().text("Signing").show();

    create_tx_params
}

pub enum SwapResult<'a> {
    CheckAddressResult(&'a mut CheckAddressParams, i32),
    PrintableAmountResult(&'a mut PrintableAmountParams, &'a str),
    CreateTxResult(&'a mut CreateTxParams, u8),
}

pub fn swap_return(res: SwapResult) {
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
