// gog-cli output module
// Output formatting - delegates to gog_core::output.

#[allow(unused_imports)]
pub use gog_core::output::{OutputConfig, OutputMode, OutputError, write_json};

/// Build an OutputConfig from CLI global flags.
#[allow(dead_code)]
pub fn make_output_config(
    json: bool,
    plain: bool,
    results_only: bool,
    select: Option<&str>,
) -> Result<OutputConfig, String> {
    let mut cfg = OutputConfig::from_flags(json, plain)?;
    cfg.results_only = results_only;
    if let Some(fields) = select {
        cfg.select_fields = fields
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    Ok(cfg)
}
