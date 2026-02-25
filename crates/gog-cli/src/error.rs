// gog-cli error module
// Maps service errors to process exit codes.

use gog_core::error::{GogError, exit_codes};

/// Map a GogError to the appropriate process exit code.
#[allow(dead_code)]
pub fn exit_code_for(err: &GogError) -> i32 {
    err.exit_code()
}

/// Map any anyhow::Error to an exit code.
/// If the error contains a GogError, use its specific code.
/// Otherwise use GENERAL_ERROR.
pub fn exit_code_for_anyhow(err: &anyhow::Error) -> i32 {
    if let Some(gog_err) = err.downcast_ref::<GogError>() {
        return gog_err.exit_code();
    }
    exit_codes::GENERAL_ERROR
}

/// Print the error message to stderr and return the exit code.
pub fn report_error(err: &anyhow::Error) -> i32 {
    let msg = if let Some(gog_err) = err.downcast_ref::<GogError>() {
        gog_err.format_for_user()
    } else {
        err.to_string()
    };
    eprintln!("error: {msg}");
    exit_code_for_anyhow(err)
}
