// Forms command.
// Mirrors: internal/cmd/forms.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Google Forms operations.
#[derive(Debug, Parser)]
#[command(about = "Google Forms")]
pub struct FormsCmd {
    #[command(subcommand)]
    pub subcommand: FormsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum FormsSubcommand {
    /// Get a form by ID.
    Get(FormsGetArgs),

    /// List responses for a form.
    #[command(aliases = ["response"])]
    Responses(FormsResponsesArgs),
}

#[derive(Debug, Parser)]
pub struct FormsGetArgs {
    /// Form ID.
    pub form_id: String,
}

#[derive(Debug, Parser)]
pub struct FormsResponsesArgs {
    /// Form ID.
    pub form_id: String,

    /// Maximum number of responses.
    #[arg(long, default_value = "20")]
    pub max_results: u32,

    /// Filter by response ID.
    #[arg(long)]
    pub response_id: Option<String>,
}

/// Execute the forms command.
pub async fn execute(cmd: &FormsCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        FormsSubcommand::Get(args) => execute_get(args, flags).await,
        FormsSubcommand::Responses(args) => execute_responses(args, flags).await,
    }
}

async fn execute_get(args: &FormsGetArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(gog_auth::Service::Forms, flags).await?;
    let form = gog_forms::get::get_form(&auth.client, &auth.access_token, &args.form_id).await?;
    crate::client::output_json(flags, &form)
}

async fn execute_responses(args: &FormsResponsesArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(gog_auth::Service::Forms, flags).await?;
    if let Some(ref response_id) = args.response_id {
        let response = gog_forms::get::get_response(
            &auth.client,
            &auth.access_token,
            &args.form_id,
            response_id,
        )
        .await?;
        crate::client::output_json(flags, &response)
    } else {
        let params = gog_forms::responses::ListResponsesParams {
            page_size: Some(args.max_results),
            page_token: None,
            filter: None,
        };
        let list = gog_forms::responses::list_responses(
            &auth.client,
            &auth.access_token,
            &args.form_id,
            &params,
        )
        .await?;
        crate::client::output_json(flags, &list)
    }
}
