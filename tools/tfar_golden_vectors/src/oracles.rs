//! The oracles: the original DSPFilters filters, the SimpleComp compressor and FAudio's F3DAudio,
//! each behind a safe wrapper over the C++ shim.

mod compressor_oracle;
mod filter_oracle;
mod oracle_bindings;
mod parameter_checks;
mod spatial_oracle;

pub use compressor_oracle::{OracleCompressor, OracleCompressorRun, OracleCompressorSettings};
pub use filter_oracle::{ORACLE_MAX_BUTTERWORTH_ORDER, OracleFilter, OracleFilterDesign};
pub use spatial_oracle::{
    ORACLE_MAX_CURVE_POINTS, OracleCone, OracleSpatialResult, OracleSpatialScene,
    f3daudio_calculate,
};
