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

pub fn get_check_address_params(arg0: u32) -> CheckAddressParams {
    unsafe {
        debug_print("GET_CHECK_ADDRESS_PARAMS\n");

        let mut libarg: libargs_t = libargs_t::default();

        let arg = arg0 as *const u32;

        libarg.id = *arg;
        libarg.command = *arg.add(1);
        libarg.unused = *arg.add(2);

        libarg.__bindgen_anon_1 = *(arg.add(3) as *const libargs_s__bindgen_ty_1);

        let params: check_address_parameters_t =
            *(libarg.__bindgen_anon_1.check_address as *const check_address_parameters_t);

        let mut check_address_params: CheckAddressParams = Default::default();

        debug_print("GET_DPATH_LENGTH\n");
        check_address_params.dpath_len = *(params.address_parameters as *const u8) as usize;

        debug_print("GET_DPATH \n");
        for i in 1..1 + check_address_params.dpath_len * 4 {
            check_address_params.dpath[i - 1] = *(params.address_parameters.add(i));
        }

        debug_print("GET_REF_ADDRESS\n");
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
        debug_print("GET_PRINTABLE_AMOUNT_PARAMS\n");

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

        debug_print("GET_AMOUNT_LENGTH\n");
        printable_amount_params.amount_len = params.amount_length as usize;

        debug_print("GET_AMOUNT\n");
        for i in 0..printable_amount_params.amount_len {
            printable_amount_params.amount[16 - printable_amount_params.amount_len + i] =
                *(params.amount.add(i));
        }

        debug_print("GET_AMOUNT_STR\n");
        printable_amount_params.amount_str = &(*(libarg.__bindgen_anon_1.get_printable_amount
            as *mut get_printable_amount_parameters_t))
            .printable_amount as *const i8 as *mut i8;

        printable_amount_params
    }
}
