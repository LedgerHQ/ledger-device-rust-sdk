#[cfg(any(target_os = "stax", target_os = "flex"))]
use crate::nbgl::NbglSpinner;
use crate::testing::debug_print;
use ledger_secure_sdk_sys::{
    check_address_parameters_t, create_transaction_parameters_t, get_printable_amount_parameters_t,
    libargs_s__bindgen_ty_1, libargs_t,
};

pub struct CheckAddressParams {
    pub dpath: [u8; 64],
    pub dpath_len: usize,
    pub ref_address: [u8; 64],
    pub ref_address_len: usize,
    pub result: *mut i32,
}

impl Default for CheckAddressParams {
    fn default() -> Self {
        CheckAddressParams {
            dpath: [0; 64],
            dpath_len: 0,
            ref_address: [0; 64],
            ref_address_len: 0,
            result: core::ptr::null_mut(),
        }
    }
}

pub struct PrintableAmountParams {
    pub amount: [u8; 16],
    pub amount_len: usize,
    pub amount_str: *mut i8,
}

impl Default for PrintableAmountParams {
    fn default() -> Self {
        PrintableAmountParams {
            amount: [0; 16],
            amount_len: 0,
            amount_str: core::ptr::null_mut(),
        }
    }
}

pub struct CreateTxParams {
    pub amount: [u8; 16],
    pub amount_len: usize,
    pub fee_amount: [u8; 16],
    pub fee_amount_len: usize,
    pub dest_address: [u8; 64],
    pub dest_address_len: usize,
    pub result: *mut u8,
}

impl Default for CreateTxParams {
    fn default() -> Self {
        CreateTxParams {
            amount: [0; 16],
            amount_len: 0,
            fee_amount: [0; 16],
            fee_amount_len: 0,
            dest_address: [0; 64],
            dest_address_len: 0,
            result: core::ptr::null_mut(),
        }
    }
}

pub fn get_check_address_params(arg0: u32) -> CheckAddressParams {
    unsafe {
        debug_print("=> get_check_address_params\n");

        let mut libarg: libargs_t = libargs_t::default();

        let arg = arg0 as *const u32;

        libarg.id = *arg;
        libarg.command = *arg.add(1);
        libarg.unused = *arg.add(2);

        libarg.__bindgen_anon_1 = *(arg.add(3) as *const libargs_s__bindgen_ty_1);

        let params: check_address_parameters_t =
            *(libarg.__bindgen_anon_1.check_address as *const check_address_parameters_t);

        let mut check_address_params: CheckAddressParams = Default::default();

        debug_print("==> GET_DPATH_LENGTH\n");
        check_address_params.dpath_len = *(params.address_parameters as *const u8) as usize;

        debug_print("==> GET_DPATH \n");
        for i in 1..1 + check_address_params.dpath_len * 4 {
            check_address_params.dpath[i - 1] = *(params.address_parameters.add(i));
        }

        debug_print("==> GET_REF_ADDRESS\n");
        let mut address_length = 0usize;
        while *(params.address_to_check.wrapping_add(address_length)) != '\0' as i8 {
            check_address_params.ref_address[address_length] =
                *(params.address_to_check.wrapping_add(address_length)) as u8;
            address_length += 1;
        }
        check_address_params.ref_address_len = address_length;

        check_address_params.result = &(*(libarg.__bindgen_anon_1.check_address
            as *mut check_address_parameters_t))
            .result as *const i32 as *mut i32;

        check_address_params
    }
}

pub fn get_printable_amount_params(arg0: u32) -> PrintableAmountParams {
    unsafe {
        debug_print("=> get_printable_amount_params\n");

        let mut libarg: libargs_t = libargs_t::default();

        let arg = arg0 as *const u32;

        libarg.id = *arg;
        libarg.command = *arg.add(1);
        libarg.unused = *arg.add(2);

        libarg.__bindgen_anon_1 = *(arg.add(3) as *const libargs_s__bindgen_ty_1);

        let params: get_printable_amount_parameters_t =
            *(libarg.__bindgen_anon_1.get_printable_amount
                as *const get_printable_amount_parameters_t);

        let mut printable_amount_params: PrintableAmountParams = Default::default();

        debug_print("==> GET_AMOUNT_LENGTH\n");
        printable_amount_params.amount_len = params.amount_length as usize;

        debug_print("==> GET_AMOUNT\n");
        for i in 0..printable_amount_params.amount_len {
            printable_amount_params.amount[16 - printable_amount_params.amount_len + i] =
                *(params.amount.add(i));
        }

        debug_print("==> GET_AMOUNT_STR\n");
        printable_amount_params.amount_str = &(*(libarg.__bindgen_anon_1.get_printable_amount
            as *mut get_printable_amount_parameters_t))
            .printable_amount as *const i8 as *mut i8;

        printable_amount_params
    }
}

extern "C" {
    fn c_reset_bss();
    fn c_boot_std();
}

pub fn sign_tx_params(arg0: u32) -> CreateTxParams {
    unsafe {
        debug_print("=> sign_tx_params\n");

        let mut libarg: libargs_t = libargs_t::default();

        let arg = arg0 as *const u32;

        libarg.id = *arg;
        libarg.command = *arg.add(1);
        libarg.unused = *arg.add(2);

        libarg.__bindgen_anon_1 = *(arg.add(3) as *const libargs_s__bindgen_ty_1);

        let params: create_transaction_parameters_t =
            *(libarg.__bindgen_anon_1.create_transaction as *const create_transaction_parameters_t);

        let mut create_tx_params: CreateTxParams = Default::default();

        debug_print("==> GET_AMOUNT\n");
        create_tx_params.amount_len = params.amount_length as usize;
        for i in 0..create_tx_params.amount_len {
            create_tx_params.amount[16 - create_tx_params.amount_len + i] = *(params.amount.add(i));
        }

        debug_print("==> GET_FEE\n");
        create_tx_params.fee_amount_len = params.fee_amount_length as usize;
        for i in 0..create_tx_params.fee_amount_len {
            create_tx_params.fee_amount[16 - create_tx_params.fee_amount_len + i] =
                *(params.fee_amount.add(i));
        }

        debug_print("==> GET_DESTINATION_ADDRESS\n");
        let mut dest_address_length = 0usize;
        while *(params.destination_address.wrapping_add(dest_address_length)) != '\0' as i8 {
            create_tx_params.dest_address[dest_address_length] =
                *(params.destination_address.wrapping_add(dest_address_length)) as u8;
            dest_address_length += 1;
        }
        create_tx_params.dest_address_len = dest_address_length;

        create_tx_params.result = &(*(libarg.__bindgen_anon_1.create_transaction
            as *mut create_transaction_parameters_t))
            .result as *const u8 as *mut u8;

        /* Reset BSS and complete application boot */
        c_reset_bss();
        c_boot_std();

        #[cfg(any(target_os = "stax", target_os = "flex"))]
        NbglSpinner::new().text("Signing").show();

        create_tx_params
    }
}

pub enum SwapResult<'a> {
    CheckAddressResult(&'a mut CheckAddressParams, i32),
    PrintableAmountResult(&'a mut PrintableAmountParams, &'a str),
    CreateTxResult(&'a mut CreateTxParams, u8),
}

pub fn swap_return(res: SwapResult) {
    unsafe {
        match res {
            SwapResult::CheckAddressResult(&mut ref p, r) => {
                *(p.result) = r;
            }
            SwapResult::PrintableAmountResult(&mut ref p, s) => {
                for (i, c) in s.chars().enumerate() {
                    *(p.amount_str.add(i)) = c as i8;
                }
                *(p.amount_str.add(s.len())) = '\0' as i8;
            }
            SwapResult::CreateTxResult(&mut ref p, r) => {
                *(p.result) = r;
            }
        }
        ledger_secure_sdk_sys::os_lib_end();
    }
}
