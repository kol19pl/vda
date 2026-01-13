use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use time::OffsetDateTime;
use crate::models::DownloadQueueItem;
use crate::{GLOBAL_DOWNLOAD_DIR};
use crate::setup::is_synology;


static QUEUE_FILE: &str = "download_queue.json";

static QUEUE_FILE_PATH: Lazy<Mutex<PathBuf>> = Lazy::new(|| {
    let mut path = std::path::PathBuf::from("/var/packages/vda_serwer/var");
    path.push("download_queue.json");
    Mutex::new(path)
});

pub(crate) fn current_unix_time_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs() as f64 + f64::from(now.subsec_nanos()) / 1_000_000_000.0
}

pub(crate) fn set_global_download_dir(dir: String) {
    let mut g = GLOBAL_DOWNLOAD_DIR.lock().unwrap();
    *g = Some(dir);
}
pub(crate) fn downloads_folder() -> String {
    if let Some(dir) = GLOBAL_DOWNLOAD_DIR.lock().unwrap().as_ref() {
        if !dir.is_empty() {
            return dir.clone();
        }
    }

    if let Ok(v) = env::var("VDA_DOWNLOADS_FOLDER") {
        if !v.is_empty() {
            return v;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join("Downloads").to_string_lossy().to_string();
    }

    "Downloads".to_string()
}


pub(crate) fn parse_ytdlp_output_line(line: &str, downloaded_file: &mut Option<PathBuf>) {
    use std::ffi::OsStr;

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }

    // Główne logowanie postępu
    if trimmed.contains("[download]") {
        if let Some(idx) = trimmed.find("Destination:") {
            // Znaleziono nazwę pliku
            let fname = trimmed[idx + "Destination:".len()..].trim();
            *downloaded_file = Some(PathBuf::from(fname));
            let basename = Path::new(fname)
                .file_name()
                .unwrap_or_else(|| OsStr::new(fname))
                .to_string_lossy()
                .to_string();
            log_info(&format!("📄 Plik: {basename}"));
        } else if trimmed.contains('%') && trimmed.contains("ETA") {
            // Klasyczny progress
            let part = trimmed
                .split_whitespace()
                .find(|p| p.contains('%'))
                .unwrap_or(trimmed);
            log_info(&format!("⏳ Postęp: {part}"));
        } else {
            // Inne komunikaty download
            log_info(&format!("⏳ Pobieranie: {trimmed}"));
        }
    } else if trimmed.contains("[Merger]") && trimmed.contains("Merging formats into") {
        if let Some(start) = trimmed.find('"') {
            if let Some(end_rel) = trimmed[start + 1..].find('"') {
                let end = start + 1 + end_rel;
                let fname = &trimmed[start + 1..end];
                *downloaded_file = Some(PathBuf::from(fname));
            }
        }
        log_info(&format!("🔄 Łączenie formatów: {trimmed}"));
    } else if trimmed.contains("[ExtractAudio]") {
        log_info(&format!("🎵 Konwersja audio: {trimmed}"));
    } else if trimmed.to_uppercase().contains("ERROR") {
        log_error(&format!("❌ Błąd: {trimmed}"));
    } else {
        // Wszystkie pozostałe linie wypisujemy, żeby nic się nie zgubiło
        log_info(&format!("ℹ️ {trimmed}"));
    }
}







      //  if let Ok(json) = serde_json::to_string_pretty(queue) {
      //      if let Err(e) = std::fs::write(QUEUE_FILE, json) {
      //          log_error(&format!("Nie udało się zapisać kolejki: {e}"));
      //      }
      //  } else {
      //      log_error("Błąd serializacji kolejki do JSON");
      //  }




///funkcje zapisu json
pub(crate) fn save_queue_to_file(queue: &[DownloadQueueItem]) {
    // Serializacja JSON
    let json = match serde_json::to_string_pretty(queue) {
        Ok(j) => j,
        Err(e) => {
            log_error(&format!("Błąd serializacji kolejki do JSON: {e}"));
            return;
        }
    };

    if is_synology() {
        let path = QUEUE_FILE_PATH.lock().unwrap();
        if let Err(e) = std::fs::write(&*path, &json) {
            log_error(&format!("Nie udało się zapisać kolejki w DSM: {e}"));
        }
    } else {
        if let Err(e) = std::fs::write(QUEUE_FILE, &json) {
            log_error(&format!("Nie udało się zapisać kolejki: {e}"));
        }
    }
}




pub(crate) fn load_queue_from_file() -> Vec<DownloadQueueItem> {
    let path = if is_synology() {
        QUEUE_FILE_PATH.lock().unwrap().clone()
    } else {
        QUEUE_FILE.to_string().parse().unwrap()
    };

    match std::fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str::<Vec<DownloadQueueItem>>(&data) {
            Ok(queue) => queue,
            Err(e) => {
                log_error(&format!("Nie udało się odczytać kolejki JSON z {:?}: {e}", path));
                vec![]
            }
        },
        Err(e) => {
            log_error(&format!("Nie udało się wczytać pliku {:?}: {e}", path));
            vec![]
        }
    }
}







pub(crate) fn log_info(msg: &str) {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    eprintln!("{now} - INFO - {msg}");
}

pub(crate) fn log_error(msg: &str) {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    eprintln!("{now} - ERROR - {msg}");
}