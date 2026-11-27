use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024; // 5 MiB

struct FolioLogger {
    file: Mutex<Option<std::fs::File>>,
    path: PathBuf,
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
        if let Ok(mut guard) = self.file.lock() {
            let _ = maybe_rotate(&mut guard, &self.path);
            if let Some(file) = guard.as_mut() {
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut guard) = self.file.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = file.flush();
            }
        }
    }
}

/// Cierra el handle antes de renombrar (necesario en Windows).
fn maybe_rotate(file: &mut Option<std::fs::File>, path: &Path) -> std::io::Result<()> {
    let Some(handle) = file.as_mut() else {
        return Ok(());
    };
    let len = handle.metadata()?.len();
    if len < MAX_LOG_BYTES {
        return Ok(());
    }
    let _ = handle.flush();
    *file = None;
    let rotated = path.with_extension("log.1");
    let _ = fs::remove_file(&rotated);
    let _ = fs::rename(path, &rotated);
    *file = Some(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?,
    );
    Ok(())
}

pub fn init(data_dir: &Path) -> PathBuf {
    let log_path = data_dir.join("folio.log");
    if let Ok(meta) = fs::metadata(&log_path) {
        if meta.len() >= MAX_LOG_BYTES {
            let rotated = data_dir.join("folio.log.1");
            let _ = fs::remove_file(&rotated);
            let _ = fs::rename(&log_path, &rotated);
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    if file.is_some() {
        let logger = FolioLogger {
            file: Mutex::new(file),
            path: log_path.clone(),
        };
        let _ = log::set_boxed_logger(Box::new(logger));
        log::set_max_level(LevelFilter::Info);
        log::info!("Log de Folio inicializado");
    } else {
        let _ = env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .try_init();
        log::warn!("No se pudo abrir folio.log; logging solo a consola");
    }

    log_path
}
