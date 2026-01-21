use ledger_secure_sdk_sys::{libargs_t, CHECK_ADDRESS, GET_PRINTABLE_AMOUNT, SIGN_TRANSACTION};

pub mod string;
pub mod swap;

// Re-export commonly used swap types for convenience
pub use swap::{SwapAppErrorCodeTrait, SwapError, SwapErrorCommonCode};

pub enum LibCallCommand {
    SwapSignTransaction,
    SwapGetPrintableAmount,
    SwapCheckAddress,
}

impl From<u32> for LibCallCommand {
    fn from(command: u32) -> Self {
        match command {
            SIGN_TRANSACTION => LibCallCommand::SwapSignTransaction,
            GET_PRINTABLE_AMOUNT => LibCallCommand::SwapGetPrintableAmount,
            CHECK_ADDRESS => LibCallCommand::SwapCheckAddress,
            _ => panic!("Unknown command"),
        }
    }
}

pub fn get_command(arg0: u32) -> LibCallCommand {
    crate::log::info!("GET_CMD");
    let mut libarg: libargs_t = libargs_t::default();

    let arg = arg0 as *const u32;

    libarg.id = unsafe { *arg };
    libarg.command = unsafe { *arg.add(1) };
    libarg.unused = unsafe { *arg.add(2) };
    libarg.command.into()
}
