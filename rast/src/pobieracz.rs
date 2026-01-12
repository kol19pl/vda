use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use actix_web::web;
use tokio::sync::mpsc;
use crate::models::{DownloadParams, JobResult};
use crate::{log_error, log_info, pobieracz, AppState, DownloadJob};
use crate::dodatkowe_funkcje::{save_queue_to_file,parse_ytdlp_output_line};
use crate::setup::{ffmpeg_available, get_ffmpeg, YTDLP_PATH};





pub(crate) async fn download_worker_loop(
    mut rx: mpsc::Receiver<DownloadJob>,
    app_state: web::Data<AppState>,
) {
    while let Some(job) = rx.recv().await {
        let id = job.id;
        let params = job.params.clone();
        let res = pobieracz::run_download_and_convert(&params, id);

        // if job.resp_tx.send(res).is_err() {
        //     log_error(&format!(
        //         "❌ Błąd wątku pobierania #{id}: nie udało się odesłać wyniku"
        //     ));
        // }

        // Usuwanie z kolejki po zakończeniu
        let mut queue = app_state.queue.lock().unwrap();
        // queue.retain(|item| item.url != job.params.url);
        queue.retain(|item| item.id != id);
        save_queue_to_file(&queue);
    }
}



pub(crate) fn run_download_and_convert(params: &DownloadParams, job_id: u64) -> JobResult {


    let has_premium = params.username.is_some() && params.password.is_some();
    let output_path = params.output_path.clone();

    let mut cmd: Vec<String> = vec![YTDLP_PATH.clone()];

    // dodajemy --newline, żeby postęp był wypisywany od razu
    cmd.push("--newline".into());

    let quality_arg = match params.quality.as_str() {
        "best" => "bestvideo+bestaudio/best",
        "worst" => "worstvideo+bestaudio/worst",
        "bestaudio" => "bestvideo+bestaudio/best",
        "best[height<=720]" => "bestvideo[height<=720]+bestaudio/best[height<=720]",
        "best[height<=480]" => "bestvideo[height<=480]+bestaudio/best[height<=480]",
        other => other,
    }
        .to_string();
    cmd.push("-f".into());
    cmd.push(quality_arg);

    cmd.push("--merge-output-format".into());
    cmd.push("mp4".into());

    let mut needs_conversion = false;
    let target_format = params.format_selector.to_lowercase();
    if matches!(target_format.as_str(), "mkv" | "webm" | "mp3") {
        needs_conversion = true;
        if target_format == "mp3" {
            log_info("🔄 Po pobraniu zostanie wykonana konwersja do MP3 (audio)");
        } else {
            log_info(&format!(
                "🔄 Po pobraniu zostanie wykonana konwersja do {}",
                target_format.to_uppercase()
            ));
        }
    }

    cmd.extend([
        "--no-part".into(),
        "--remux-video".into(),
        "mp4".into(),
        "--no-keep-fragments".into(),
        "--fixup".into(),
        "detect_or_warn".into(),
        "--postprocessor-args".into(),
        "ffmpeg:-movflags +faststart".into(),
        "--concurrent-fragments".into(),
        "10".into(),
        "--retries".into(),
        "10".into(),
        "--fragment-retries".into(),
        "10".into(),
    ]);

    cmd.extend([
        "--no-playlist".into(),
        "--no-write-info-json".into(),
        "--no-write-thumbnail".into(),
        "--no-write-description".into(),
        "--no-write-auto-sub".into(),
        "--no-write-sub".into(),
        "--no-embed-thumbnail".into(),
        "--add-metadata".into(),
        "--no-warnings".into(),
    ]);

    let output_template = if let Some(title) = &params.custom_title {
        let clean = clean_filename(title);
        log_info(&format!("📋 Używam własnego tytułu: {clean}"));
        output_path.join(format!("{clean}.%(ext)s"))
    } else {
        output_path.join("%(title)s.%(ext)s")
    };
    let output_template_str = output_template.to_string_lossy().to_string();
    cmd.push("-o".into());
    cmd.push(output_template_str);

    if has_premium {
        if let (Some(u), Some(p)) = (&params.username, &params.password) {
            cmd.push("--username".into());
            cmd.push(u.clone());
            cmd.push("--password".into());
            cmd.push(p.clone());
            log_info("👑 Używam konta Premium do pobierania");
        }
    }

    cmd.push(params.url.clone());

    log_info(&format!("🚀 Start pobierania #{job_id}"));
    log_info(&format!("🚀 Rozpoczynam pobieranie z URL: {}", params.url));

    let mut child = match Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Nie udało się uruchomić yt-dlp: {e}");
            log_error(&msg);
            return JobResult {
                success: false,
                http_status: 500,
                message: None,
                error: Some(msg),
                output_path: None,
            };
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let downloaded_file = Arc::new(Mutex::new(None));
    let df_clone1 = downloaded_file.clone();
    let df_clone2 = downloaded_file.clone();


    /////duplikacja to nie błąd!!!!!!
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let mut df = df_clone1.lock().unwrap();
            parse_ytdlp_output_line(&line, &mut df);
        }
    });

    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let mut df = df_clone2.lock().unwrap();
            parse_ytdlp_output_line(&line, &mut df);
        }
    });

    // Czekamy na zakończenie yt-dlp
    let _status = child.wait().expect("Błąd oczekiwania na yt-dlp");

    // Czekamy na wątki stdout/stderr
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    // Pobranie finalnej wartości pobranego pliku
    let downloaded_file = downloaded_file.lock().unwrap().clone();


    ////
    let mut actual_downloaded_file: Option<PathBuf> = None;

    if needs_conversion {
        if let Some(mut p) = downloaded_file.clone() {
            if !p.is_absolute() {
                p = output_path.join(p);
            }
            if p.exists() {
                actual_downloaded_file = Some(p.clone());
                log_info(&format!(
                    "✅ Używam pliku z yt-dlp: {}",
                    p.to_string_lossy()
                ));
            }
        }

        if actual_downloaded_file.is_none() {
            log_info("🔍 Wyszukiwanie pobranego pliku MP4 do konwersji...");
            if let Some(p) = find_latest_mp4_in_dir(&output_path) {
                log_info(&format!(
                    "Znaleziono plik do konwersji: {}",
                    p.to_string_lossy()
                ));
                actual_downloaded_file = Some(p);
            } else {
                log_error("⚠️ Nie znaleziono pliku MP4 do konwersji");
            }
        }

        if let Some(src) = &actual_downloaded_file {
            if !src.exists() {
                log_error(&format!("⚠️ Błąd konwersji - plik nie istnieje"));
            } else if !ffmpeg_available() {
                log_error("⚠️ FFmpeg nie jest dostępny - pomijam konwersję");
            } else {
                let base = src.with_extension("");
                let output_file = base.with_extension(&target_format);

                log_info(&format!(
                    "🔄 Rozpoczynam konwersję {} do {}",
                    src.to_string_lossy(),
                    target_format
                ));

                let ffmpeg_path = get_ffmpeg();


                let ffmpeg_cmd: Vec<String> = if target_format == "mp3" {
                    vec![
                        ffmpeg_path.clone(),
                        "-i".into(),
                        src.to_string_lossy().into(),
                        "-vn".into(),
                        "-acodec".into(),
                        "libmp3lame".into(),
                        "-q:a".into(),
                        "2".into(),
                        "-y".into(),
                        output_file.to_string_lossy().into(),
                    ]
                } else {
                    vec![
                        ffmpeg_path.clone(),
                        "-i".into(),
                        src.to_string_lossy().into(),
                        "-c".into(),
                        "copy".into(),
                        "-movflags".into(),
                        "+faststart".into(),
                        "-y".into(),
                        output_file.to_string_lossy().into(),
                    ]
                };

                log_info(&format!("ffmpeg cmd: {:?}", ffmpeg_cmd));

                let status = Command::new(&ffmpeg_cmd[0])
                    .args(&ffmpeg_cmd[1..])
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        log_info("✅ Konwersja zakończona pomyślnie!");
                        if let Err(e) = fs::remove_file(src) {
                            log_error(&format!(
                                "Nie udało się usunąć oryginalnego pliku: {e}"
                            ));
                        } else {
                            log_info("🗑️ Usunięto oryginalny plik MP4");
                        }
                        if let Some(name) = output_file.file_name() {
                            log_info(&format!(
                                "📁 Zapisano jako: {}",
                                name.to_string_lossy()
                            ));
                        }
                    }
                    Ok(_) => {
                        log_error("⚠️ Konwersja nie powiodła się");
                        log_info("ℹ️ Plik pozostał w formacie MP4");
                    }
                    Err(e) => {
                        log_error(&format!("Błąd konwersji ffmpeg: {e}"));
                        log_info("ℹ️ Plik pozostał w formacie MP4");
                    }
                }
            }
        } else {
            log_error("⚠️ Nie można wykonać konwersji - nie znaleziono pliku");
        }
    }

    log_info(&format!(
        "📁 Zapisano do: {}",
        output_path.to_string_lossy()
    ));

    JobResult {
        success: true,
        http_status: 200,
        message: Some("Pobieranie zakończone pomyślnie".into()),
        error: None,
        output_path: Some(output_path.to_string_lossy().to_string()),
    }
}



fn find_latest_mp4_in_dir(dir: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let path = e.path();
            if path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("mp4"))
                .unwrap_or(false)
            {
                if let Ok(meta) = e.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        candidates.push((path, mtime));
                    }
                }
            }
        }
    }
    candidates.sort_by_key(|(_, t)| *t);
    candidates.pop().map(|(p, _)| p)
}

fn clean_filename(title: &str) -> String {
    // mapowanie polskich znaków i innych na ASCII
    let translit = |c: char| match c {
        'ą' | 'Ą' => 'a',
        'ć' | 'Ć' => 'c',
        'ę' | 'Ę' => 'e',
        'ł' | 'Ł' => 'l',
        'ń' | 'Ń' => 'n',
        'ó' | 'Ó' => 'o',
        'ś' | 'Ś' => 's',
        'ż' | 'Ż' | 'ź' | 'Ź' => 'z',
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        _ => c,
    };

    let mut s: String = title
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c => translit(c),
        })
        .collect();

    if s.len() > 100 {
        s.truncate(100);
    }

    s = s.trim_matches(&['_', ' ', '.'][..]).to_string();

    if s.is_empty() {
        "Unknown_Video".into()
    } else {
        s
    }
}
