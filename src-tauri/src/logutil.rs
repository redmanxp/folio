use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

struct FolioLogger {
    file: Mutex<std::fs::File>,
}

impl Log for FolioLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} [{}] {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        );
        eprint!("{line}");
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

pub fn init(data_dir: &Path) -> PathBuf {
    let log_path = data_dir.join("folio.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    if let Some(file) = file {
        let logger = FolioLogger {
            file: Mutex::new(file),
        };
        let _ = log::set_boxed_logger(Box::new(logger));
        log::set_max_level(LevelFilter::Info);
        log::info!("Log de Folio: {}", log_path.display());
    } else {
        let _ = env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .try_init();
        log::warn!("No se pudo abrir folio.log; logging solo a consola");
    }

    log_path
}
