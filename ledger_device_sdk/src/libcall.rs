use crate::testing::debug_print;
use ledger_secure_sdk_sys::libargs_s__bindgen_ty_1;
use ledger_secure_sdk_sys::{
    check_address_parameters_t, create_transaction_parameters_t, get_printable_amount_parameters_t,
    libargs_t, os_lib_end, CHECK_ADDRESS, GET_PRINTABLE_AMOUNT, SIGN_TRANSACTION,
};

pub mod string;
use string::CustomString;

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

pub enum LibCallCommand {
    SignTransaction,
    GetPrintableAmount,
    CheckAddress,
}

impl From<u32> for LibCallCommand {
    fn from(command: u32) -> Self {
        match command {
            SIGN_TRANSACTION => LibCallCommand::SignTransaction,
            GET_PRINTABLE_AMOUNT => LibCallCommand::GetPrintableAmount,
            CHECK_ADDRESS => LibCallCommand::CheckAddress,
            _ => panic!("Unknown command"),
        }
    }
}

pub fn get_command(arg0: u32) -> LibCallCommand {
    debug_print("GET_CMD\n");
    let mut libarg: libargs_t = libargs_t::default();

    let arg = arg0 as *const u32;

    libarg.id = unsafe { *arg };
    libarg.command = unsafe { *arg.add(1) };
    libarg.unused = unsafe { *arg.add(2) };

    debug_print("libarg content:\n");
    let id = CustomString::<8>::from(libarg.id);
    debug_print(id.as_str());
    debug_print("\n");
    let cmd = CustomString::<8>::from(libarg.command);
    debug_print(cmd.as_str());
    debug_print("\n");
    let unused = CustomString::<8>::from(libarg.unused);
    debug_print(unused.as_str());
    debug_print("\n");

    libarg.command.into()
}

pub fn get_check_address_params(arg0: u32) -> CheckAddressParams {
    unsafe {
        debug_print("GET_CHECK_ADDRESS_PARAMS\n");

        let mut libarg: libargs_t = libargs_t::default();

        let arg = arg0 as *const u32;

        libarg.id = *arg;
        libarg.command = *arg.add(1);
        libarg.unused = *arg.add(2);

        debug_print("libarg content:\n");
        let id = CustomString::<8>::from(libarg.id);
        debug_print(id.as_str());
        debug_print("\n");
        let cmd = CustomString::<8>::from(libarg.command);
        debug_print(cmd.as_str());
        debug_print("\n");
        let unused = CustomString::<8>::from(libarg.unused);
        debug_print(unused.as_str());
        debug_print("\n");

        libarg.__bindgen_anon_1 = *(arg.add(3) as *const libargs_s__bindgen_ty_1);

        let params: check_address_parameters_t =
            *(libarg.__bindgen_anon_1.check_address as *const check_address_parameters_t);

        let mut check_address_params: CheckAddressParams = Default::default();

        debug_print("Display address_parameters\n");
        for i in 0..10 {
            let s = CustomString::<2>::from(*((params.address_parameters as *const u8).add(i)));
            debug_print(s.as_str());
        }
        debug_print("\n");

        debug_print("GET_DPATH_LENGTH\n");
        check_address_params.dpath_len = *(params.address_parameters as *const u8) as usize;

        if check_address_params.dpath_len == 5 {
            debug_print("dpath_len is 5\n");
        }

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

        // "EFr6nRvgKKeteKoEH7hudt8UHYiu94Liq2yMM7x2AU9U"
        debug_print("Display ref address\n");

        let mut s = CustomString::<44>::new();
        s.arr.copy_from_slice(
            &check_address_params.ref_address[..check_address_params.ref_address_len],
        );
        s.len = check_address_params.ref_address_len;
        debug_print(s.as_str());
        debug_print("\n");

        //(*(libarg.__bindgen_anon_1.check_address as *mut check_address_parameters_t)).result = 1;
        check_address_params.result = (&(*(libarg.__bindgen_anon_1.check_address
            as *mut check_address_parameters_t))
            .result as *const i32 as *mut i32);

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
        printable_amount_params.amount_str = (&(*(libarg.__bindgen_anon_1.get_printable_amount
            as *mut get_printable_amount_parameters_t))
            .printable_amount as *const i8
            as *mut i8);

        printable_amount_params
    }
}

//     match libarg.command {
//         SIGN_TRANSACTION => {
//             debug_print("SIGN_TX\n");
//             let sign_tx_param: create_transaction_parameters_t =
//                 unsafe { *(arg.add(3) as *const create_transaction_parameters_t) };
//         }
//         GET_PRINTABLE_AMOUNT => {
//             debug_print("GET_PRINTABLE_AMOUNT\n");
//             let get_printable_amount_param: get_printable_amount_parameters_t =
//                 unsafe { *(arg.add(3) as *const get_printable_amount_parameters_t) };
//         }
//         CHECK_ADDRESS => {
//             let value = unsafe { *arg.add(3) as *mut check_address_parameters_t };
//             let params: &mut check_address_parameters_t = unsafe { &mut *value };

//             let mut check_address_params: CheckAddressParams = Default::default();

//             check_address_params.dpath_len = params.dpath_len;
//             check_address_params
//                 .dpath
//                 .copy_from_slice(&params.dpath[..params.dpath_len]);

//             check_address_params.ref_address = params.address_to_check as *const u8;
//         }
//         _ => {
//             debug_print("unknown command\n");
//         }
//     }

//     debug_print("end of call app as a lib\n");
//     unsafe {
//         os_lib_end();
//     }
// }
