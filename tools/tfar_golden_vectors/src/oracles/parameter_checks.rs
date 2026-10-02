//! The parameter checks every oracle runs before it calls the shim: a positive finite value, and a
//! frequency strictly between 0 and Nyquist. Pure functions, callable from any thread.

use crate::error::{Error, Result};

#[cfg(test)]
#[path = "tests/parameter_checks.rs"]
mod tests;

/// Refuses a value that is not finite and above 0.
pub(crate) fn check_positive(parameter: &'static str, value: f64) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidParameter {
            parameter,
            value,
            requirement: "finite and above 0",
        })
    }
}

/// Refuses a frequency that is not strictly between 0 and `nyquist_hz`.
pub(crate) fn check_below_nyquist(
    parameter: &'static str,
    value: f64,
    nyquist_hz: f64,
) -> Result<()> {
    if value.is_finite() && value > 0.0 && value < nyquist_hz {
        Ok(())
    } else {
        Err(Error::InvalidParameter {
            parameter,
            value,
            requirement: "strictly between 0 and the Nyquist frequency",
        })
    }
}
