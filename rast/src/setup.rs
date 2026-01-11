use std::process::{Command, Stdio};
use crate::models::YtDlpStatus;
use crate::{log_error, log_info, YTDLP_STATUS};









pub(crate) fn check_ytdlp_once() -> &'static YtDlpStatus {
    YTDLP_STATUS.get_or_init(|| {
        log_info("Sprawdzam yt-dlp w PATH i ./bin...");

        // Lista możliwych lokalizacji
        let candidates = [
            "yt-dlp",
            "./bin/yt-dlp",
            #[cfg(target_os = "windows")]
            "yt-dlp.exe",
            #[cfg(target_os = "windows")]
            ".\\bin\\yt-dlp.exe",
        ];

        for cmd in candidates.iter() {
            match Command::new(cmd)
                .arg("--version")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(out) if out.status.success() => {
                    let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    log_info(&format!("✅ yt-dlp jest zainstalowany: {} (komenda: {})", ver, cmd));
                    return YtDlpStatus {
                        installed: true,
                        version: Some(ver.clone()),
                        error: None,
                        message: format!("yt-dlp wersja {} jest zainstalowany ({})", ver, cmd),
                    };
                }
                Ok(_) => {
                    log_error(&format!("⚠️ yt-dlp istnieje, ale nie działa poprawnie: {}", cmd));
                }
                Err(_) => {
                    log_info(&format!("⚠️ nie znaleziono yt-dlp w: {}", cmd));
                }
            }
        }

        log_error("⚠️ yt-dlp nie jest zainstalowany w PATH ani ./bin");
        YtDlpStatus {
            installed: false,
            version: None,
            error: Some("not_found".into()),
            message: "yt-dlp nie jest zainstalowany".into(),
        }
    })
}

pub(crate) fn check_dependencies() {
    #[cfg(target_os = "linux")]
    {
        log_info("🔍 Sprawdzam dostępność yt-dlp i ffmpeg na Linuxie...");

        // Sprawdzenie yt-dlp
        let yt_installed = Command::new("yt-dlp").arg("--version").output().is_ok();
        if !yt_installed {
            log_error("❌ yt-dlp nie jest zainstalowany lub nie jest w PATH");

            // print!("Chcesz pobrać yt-dlp automatycznie? (y/n): ");
            // io::stdout().flush().unwrap();

            //  let mut input = String::new();
            //  io::stdin().read_line(&mut input).unwrap();

            //  if input.trim().eq_ignore_ascii_case("y") {
            log_info("📥 Pobieram yt-dlp...");
            let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";
            let out_path = "./bin/yt-dlp";
            std::fs::create_dir_all("./bin").expect("Nie udało się utworzyć katalogu ./bin");


            let resp = reqwest::blocking::get(url).expect("Nie udało się pobrać yt-dlp");
            let bytes = resp.bytes().expect("Błąd odczytu pobranego pliku");
            std::fs::write(out_path, &bytes).expect("Nie udało się zapisać yt-dlp");
            std::fs::set_permissions(out_path, std::fs::Permissions::from_mode(0o755))
                .expect("Nie udało się nadać uprawnień wykonywalnych");

            log_info("✅ yt-dlp został pobrany i zapisany w ./bin/yt-dlp");
            //  } else {
            //    log_error("❌ yt-dlp nie został zainstalowany. Pobieranie nie będzie działać.");
            // }
        } else {
            log_info("✅ yt-dlp jest dostępny");
        }

        // Sprawdzenie ffmpeg
        if Command::new("ffmpeg").arg("-version").output().is_err() {
            log_error("❌ ffmpeg nie jest zainstalowany lub nie jest w PATH");
        } else {
            log_info("✅ ffmpeg jest dostępny");
        }
    }
}


#[cfg(target_family = "unix")]

fn set_executable(path: &str) {
    #[cfg(target_family = "unix")]
    {
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755); // chmod +x
        std::fs::set_permissions(path, perms).unwrap();
    }
    #[cfg(not(target_family = "unix"))]
    {
        // Windows: nie trzeba nic robić, albo użyć atrybutów Windows jeśli konieczne
    }
}