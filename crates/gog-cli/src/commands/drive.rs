// Drive command.
// Mirrors: internal/cmd/drive.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Google Drive operations.
#[derive(Debug, Parser)]
#[command(about = "Google Drive")]
pub struct DriveCmd {
    #[command(subcommand)]
    pub subcommand: DriveSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DriveSubcommand {
    /// List files in a folder.
    #[command(aliases = ["list"])]
    Ls(DriveLsArgs),

    /// Search for files.
    #[command(aliases = ["find"])]
    Search(DriveSearchArgs),

    /// Upload a file.
    #[command(aliases = ["up", "put"])]
    Upload(DriveUploadArgs),

    /// Download a file.
    #[command(aliases = ["dl"])]
    Download(DriveDownloadArgs),

    /// Get metadata for a file by ID.
    Get(DriveGetArgs),
}

#[derive(Debug, Parser)]
pub struct DriveLsArgs {
    /// Folder ID or path (defaults to root).
    pub folder: Option<String>,

    /// Maximum number of files to return.
    #[arg(long, default_value = "20")]
    pub max_results: u32,

    /// Include trashed files.
    #[arg(long)]
    pub include_trashed: bool,
}

#[derive(Debug, Parser)]
pub struct DriveSearchArgs {
    /// Search query.
    pub query: String,

    /// Maximum number of results.
    #[arg(long, default_value = "20")]
    pub max_results: u32,
}

#[derive(Debug, Parser)]
pub struct DriveUploadArgs {
    /// Local file path to upload.
    pub file: String,

    /// Destination folder ID.
    #[arg(long, short = 'f')]
    pub folder: Option<String>,

    /// Override the filename on Drive.
    #[arg(long)]
    pub name: Option<String>,

    /// MIME type override.
    #[arg(long)]
    pub mime_type: Option<String>,

    /// Replace existing file with this ID.
    #[arg(long)]
    pub replace: Option<String>,
}

#[derive(Debug, Parser)]
pub struct DriveDownloadArgs {
    /// File ID to download.
    pub file_id: String,

    /// Destination path (defaults to current directory).
    #[arg(long, short = 'o')]
    pub output: Option<String>,

    /// Export MIME type (for Google Docs/Sheets/Slides).
    #[arg(long)]
    pub export_mime: Option<String>,
}

#[derive(Debug, Parser)]
pub struct DriveGetArgs {
    /// File ID.
    pub file_id: String,

    /// Fields to include.
    #[arg(long)]
    pub fields: Option<String>,
}

/// Execute the drive command.
pub async fn execute(cmd: &DriveCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        DriveSubcommand::Ls(args) => execute_ls(args, flags).await,
        DriveSubcommand::Search(args) => execute_search(args, flags).await,
        DriveSubcommand::Upload(args) => execute_upload(args, flags).await,
        DriveSubcommand::Download(args) => execute_download(args, flags).await,
        DriveSubcommand::Get(args) => execute_get(args, flags).await,
    }
}

async fn execute_ls(_args: &DriveLsArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement drive ls")
}

async fn execute_search(_args: &DriveSearchArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement drive search")
}

async fn execute_upload(_args: &DriveUploadArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement drive upload")
}

async fn execute_download(_args: &DriveDownloadArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement drive download")
}

async fn execute_get(_args: &DriveGetArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement drive get")
}
