// Drive command.
// Mirrors: internal/cmd/drive.go

use anyhow::Result;
use clap::{Parser, Subcommand};

use gog_auth::Service;

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

async fn execute_ls(args: &DriveLsArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Drive, flags).await?;

    let opts = gog_drive::list::ListOptions {
        parent_id: args.folder.clone(),
        page_size: Some(args.max_results),
        include_trashed: args.include_trashed,
        ..Default::default()
    };

    let file_list = gog_drive::list::list_files(&auth.client, &auth.access_token, &opts).await?;
    crate::client::output(flags, &file_list, || drive_file_rows(&file_list.files))
}

async fn execute_search(args: &DriveSearchArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Drive, flags).await?;

    let files =
        gog_drive::search::search_by_name(&auth.client, &auth.access_token, &args.query, None)
            .await?;

    crate::client::output(flags, &files, || drive_file_rows(&files))
}

async fn execute_upload(args: &DriveUploadArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Drive, flags).await?;

    let path = std::path::Path::new(&args.file);
    let content = std::fs::read(path)?;

    // Determine the filename: explicit --name flag, or the local file's stem.
    let name = args.name.clone().unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload")
            .to_string()
    });

    // Determine the MIME type: explicit flag, or sniff from file extension.
    let mime_type = args.mime_type.clone().unwrap_or_else(|| {
        mime_from_extension(path.extension().and_then(|e| e.to_str()).unwrap_or(""))
    });

    let parents = match &args.folder {
        Some(folder_id) => vec![folder_id.clone()],
        None => vec![],
    };

    let opts = gog_drive::upload::UploadOptions {
        name,
        mime_type,
        parents,
    };

    let file =
        gog_drive::upload::upload_file(&auth.client, &auth.access_token, &opts, content).await?;
    crate::client::output_json(flags, &file)
}

async fn execute_download(args: &DriveDownloadArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Drive, flags).await?;

    let bytes = if let Some(export_mime) = &args.export_mime {
        gog_drive::download::export_file(
            &auth.client,
            &auth.access_token,
            &args.file_id,
            export_mime,
        )
        .await?
    } else {
        gog_drive::download::download_file(&auth.client, &auth.access_token, &args.file_id).await?
    };

    // Determine the output path: explicit --output flag, or the file_id as filename.
    let output_path = args.output.clone().unwrap_or_else(|| args.file_id.clone());

    std::fs::write(&output_path, &bytes)?;

    // Report what was written as a small JSON object.
    let result = serde_json::json!({
        "file_id": args.file_id,
        "output": output_path,
        "bytes": bytes.len(),
    });
    crate::client::output_json(flags, &result)
}

async fn execute_get(args: &DriveGetArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Drive, flags).await?;

    let file = gog_drive::list::get_file(&auth.client, &auth.access_token, &args.file_id).await?;

    crate::client::output_json(flags, &file)
}

fn drive_file_rows(files: &[gog_drive::types::DriveFile]) -> Vec<Vec<String>> {
    let mut rows = vec![vec![
        "ID".into(),
        "NAME".into(),
        "TYPE".into(),
        "SIZE".into(),
        "MODIFIED".into(),
    ]];
    for f in files {
        rows.push(vec![
            f.id.clone(),
            f.name.clone(),
            drive_type(&f.mime_type),
            f.size.clone().unwrap_or("-".into()),
            f.modified_time
                .map_or("-".into(), |t| t.format("%Y-%m-%d %H:%M").to_string()),
        ]);
    }
    rows
}

fn drive_type(mime: &str) -> String {
    match mime {
        "application/vnd.google-apps.folder" => "folder",
        "application/vnd.google-apps.document" => "gdoc",
        "application/vnd.google-apps.spreadsheet" => "gsheet",
        "application/vnd.google-apps.presentation" => "gslides",
        "application/vnd.google-apps.form" => "gform",
        _ => mime.rsplit('/').next().unwrap_or(mime),
    }
    .to_string()
}

/// Infer a MIME type from a file extension. Falls back to
/// `application/octet-stream` for unknown extensions.
fn mime_from_extension(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "csv" => "text/csv",
        "md" => "text/markdown",
        "js" | "mjs" => "text/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
    .to_string()
}
