use std::fs;
use std::net::{IpAddr, TcpListener};
use std::process::{Command, Stdio};
use crate::models::YtDlpStatus;
use crate::{log_error, log_info, YTDLP_STATUS};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::ptr::null;
use std::sync::RwLock;
use once_cell::sync::Lazy;
//tail -f /var/packages/vda_serwer/var/vda_serwer.log

// Globalna zmienna FFmpeg
static FFMPEG_GO: Lazy<RwLock<String>> = Lazy::new(|| {
    RwLock::new("ffmpeg".to_string()) // domyślny ffmpeg w PATH
});

// Funkcja do pobrania ścieżki ffmpeg
pub fn get_ffmpeg() -> String {
    FFMPEG_GO.read().unwrap().clone()
}

// Funkcja do zmiany ścieżki ffmpeg
pub fn set_ffmpeg(path: &str) {
    let mut ffmpeg = FFMPEG_GO.write().unwrap();
    *ffmpeg = path.to_string();
}


pub(crate) static YTDLP_PATH: Lazy<String> = Lazy::new(|| {
    if cfg!(target_os = "linux") && !Command::new("yt-dlp").output().is_ok() {
        if(is_synology()) {"/var/packages/vda_serwer/var/bin/yt-dlp".into()}
        else { "./bin/yt-dlp".into()}
    } else {
        "yt-dlp".into()
    }
});

pub(crate) fn check_ytdlp_once() -> &'static YtDlpStatus {
    YTDLP_STATUS.get_or_init(|| {
        log_info("Sprawdzam yt-dlp w PATH i ./bin...");

        // Lista możliwych lokalizacji
        let candidates = [
            "yt-dlp",
            "./bin/yt-dlp",
            "/var/packages/vda_serwer/var/bin/yt-dlp",
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


pub(crate) fn port_is_available(ip: IpAddr, port: u16) -> bool {
    TcpListener::bind((ip, port)).is_ok()
}


pub(crate) fn ffmpeg_available() -> bool {
    let ffmpeg_path = get_ffmpeg();
    Command::new(ffmpeg_path)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}



pub(crate) fn ffmpeg_versia() {
    let ffmpeg_path = std::env::var("FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".to_string());
    set_ffmpeg(ffmpeg_path.as_str());

    let ffmpeg_path = get_ffmpeg();
    let ffmpeg_version = Command::new(ffmpeg_path)
        .arg("-version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)       // Cow<str>
                .lines()                             // iterator po liniach
                .next()                              // pierwsza linia Option<&str>
                .unwrap_or("Brak wersji")            // &str
                .to_string()                         // zamiana w String
        })
        .unwrap_or_else(|_| "Nie udało się uruchomić ffmpeg".to_string());

    println!("ℹ Wersja ffmpeg: {}", ffmpeg_version);
}



pub(crate) fn is_synology() -> bool {
    std::path::Path::new("/etc/synoinfo.conf").exists()
}

pub(crate) fn check_dependencies() {
    #[cfg(target_os = "windows")]
    {


        println!("🔍 Sprawdzam dostępność yt-dlp i ffmpeg na Windows...");

        let bin_dir = Path::new("./bin");
        fs::create_dir_all(bin_dir).expect("TODO: panic message");

        let candidates = ["yt-dlp.exe", "./bin/yt-dlp.exe"];

        let mut yt_found = false;
        for cmd in candidates.iter() {
            if let Ok(out) = Command::new(cmd).arg("--version").output() {
                if out.status.success() {
                    println!("✅ yt-dlp jest dostępny: {} ({})", String::from_utf8_lossy(&out.stdout), cmd);
                    yt_found = true;
                    break;
                }
            }
            println!("⚠️ Nie znaleziono yt-dlp w: {}", cmd);
        }



        if !yt_found {

            #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
            let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
            #[cfg(all(target_os = "windows", target_arch = "x86"))]
            let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_x86.exe";
            #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
            let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_arm64.exe";

            println!("📥 Pobieram yt-dlp dla Windows...");
            let out_path = bin_dir.join("yt-dlp.exe");
            let status = Command::new("powershell")
                .arg("-Command")
                .arg(format!("Invoke-WebRequest -Uri {} -OutFile {}", url, out_path.display()))
                .status()
                .expect("Nie udało się uruchomić PowerShell");

            if status.success() {
                println!("✅ yt-dlp został pobrany do ./bin/yt-dlp.exe");
            } else {
                println!("❌ Nie udało się pobrać yt-dlp.exe");
            }
        }

        // ffmpeg
        if Command::new("ffmpeg").arg("-version").output().is_err() {
            println!("❌ ffmpeg nie jest zainstalowany lub nie jest w PATH");
        } else {
            println!("✅ ffmpeg jest dostępny");
        }
    }

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
            let mut url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";
            //url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";
            if is_synology(){
                url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";
                log_info("📥 Pobieram pytchon dla synology yt-dlp...");
            }else {
                #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
                {
                    url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_musllinux";
                    log_info("📥 Pobieram pytchon dla musl-linux yt-dlp...");
                }
                #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]{
                    url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";
                    log_info("📥 Pobieram pytchon dla linux yt-dlp...");
                }
            }


            let mut out_path = "./bin/yt-dlp";

            if is_synology() {
                out_path = "/var/packages/vda_serwer/var/bin/yt-dlp";
                log_info(&format!("📥 Ustawiam ścieżki dla synology: {}", out_path));
                setup_random_for_synology();
            }

            // Tworzymy katalog nadrzędny pliku, jeśli nie istnieje
            let out_dir = std::path::Path::new(out_path)
                .parent()
                .expect("Nie można uzyskać katalogu nadrzędnego");
            fs::create_dir_all(out_dir)
                .expect(&format!("Nie udało się utworzyć katalogu: {:?}", out_dir));

            // Pobranie pliku
            log_info(&format!("📥 Rozpoczynam pobieranie {}", out_path));
            let resp = reqwest::blocking::get(url).expect("Nie udało się pobrać yt-dlp");
            let bytes = resp.bytes().expect("Błąd odczytu pobranego pliku");

            // Zapis do pliku
            fs::write(&out_path, &bytes)
                .expect(&format!("Nie udało się zapisać yt-dlp: {}", out_path));

            // Nadaj prawa wykonywalne (tylko dla Unix / Synology)
            #[cfg(unix)]
            fs::set_permissions(out_path, fs::Permissions::from_mode(0o755))
                .expect("Nie udało się nadać uprawnień wykonywalnych");

            log_info(&format!("✅ yt-dlp został pobrany i zapisany w {:?}", out_path));
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
    ffmpeg_versia();
}
#[cfg(target_os = "linux")]
fn setup_random_for_synology() {
    if is_synology() {
        std::env::set_var("RUST_RANDOM_SEED", "urandom");
        std::env::set_var("RNG_SEED_DEVICE", "/dev/urandom");
        log_info("✅ Wymuszono użycie /dev/urandom dla Synology");
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