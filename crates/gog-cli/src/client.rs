// gog-cli client module
// Shared helpers for building authenticated clients and output from CLI flags.

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::GlobalFlags;
use gog_api::AuthenticatedClient;
use gog_auth::Service;
use gog_core::output::OutputConfig;

/// Resolve the account email from flags or the default account in the keyring.
pub fn require_account(flags: &GlobalFlags) -> Result<String> {
    if let Some(ref email) = flags.account {
        return Ok(email.clone());
    }
    let store = gog_secrets::open_store()?;
    if let Ok(Some(email)) = store.get_default_account(&flags.client) {
        return Ok(email);
    }
    bail!("no account specified (use --account or run 'gog auth add' first)")
}

/// Build an authenticated API client for the given Google service.
pub async fn build_client(service: Service, flags: &GlobalFlags) -> Result<AuthenticatedClient> {
    let email = require_account(flags)?;
    AuthenticatedClient::new(service, &email, &flags.client)
        .await
        .context("failed to authenticate")
}

/// Build an OutputConfig from global flags.
pub fn make_output(flags: &GlobalFlags) -> Result<OutputConfig> {
    crate::output::make_output_config(
        flags.json,
        flags.plain,
        flags.results_only,
        flags.select.as_deref(),
    )
    .map_err(|e| anyhow::anyhow!(e))
}

/// Write a serializable value to stdout as JSON, applying any configured transforms.
pub fn output_json<T: Serialize>(flags: &GlobalFlags, value: &T) -> Result<()> {
    let cfg = make_output(flags)?;
    let mut stdout = std::io::stdout().lock();
    gog_core::output::write_json(&mut stdout, value, &cfg)?;
    Ok(())
}

/// Output data with mode-aware formatting.
///
/// - `--json` → pretty JSON
/// - `--plain` → raw TSV (tab-separated, no alignment)
/// - default (text) → aligned table with column headers
///
/// `text_rows` is called only when not in JSON mode. It should return rows
/// where the first row is the header (e.g., `["ID", "NAME", "EMAIL"]`).
pub fn output<T: Serialize, F>(flags: &GlobalFlags, value: &T, text_rows: F) -> Result<()>
where
    F: FnOnce() -> Vec<Vec<String>>,
{
    if flags.json {
        return output_json(flags, value);
    }

    let rows = text_rows();
    if rows.is_empty() {
        return Ok(());
    }

    let mut stdout = std::io::stdout().lock();
    if flags.plain {
        gog_core::output::write_tsv(&mut stdout, &rows)?;
    } else {
        gog_core::output::write_table(&mut stdout, &rows)?;
    }
    Ok(())
}
