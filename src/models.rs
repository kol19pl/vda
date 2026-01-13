use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize)]
pub struct StatusResponse {
    pub(crate) status: &'static str,
    pub(crate) version: &'static str,
    pub(crate) timestamp: f64,
    pub(crate) downloads_folder: String,}

#[derive(Serialize, Clone)]
pub struct YtDlpStatus {
    pub(crate) installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
    pub(crate) message: String,}

#[derive(Deserialize)]
pub struct DownloadRequest {
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) quality: Option<String>,
    #[serde(default)]
    pub(crate) format: Option<String>,
    #[serde(default)]
    pub(crate) subfolder: Option<String>,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) username: Option<String>,
    #[serde(default)]
    pub(crate) password: Option<String>,
}

#[derive(Serialize)]
pub struct DownloadResponse {
    pub(crate) success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_path: Option<String>,
    pub(crate) id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadQueueItem {     pub(crate) url: String,
    pub(crate) quality: String,
    pub(crate) format_selector: String,
    pub(crate) subfolder: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) id: u64,
}

pub struct JobResult {
    pub(crate) success: bool,
    pub(crate) http_status: u16,
    pub(crate) message: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) output_path: Option<String>, }



#[derive(Clone)]
pub struct DownloadParams {
    pub(crate) url: String,
    pub(crate) quality: String,
    pub(crate) format_selector: String,
    pub(crate) output_path: PathBuf,
    pub(crate) custom_title: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
}


