use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::Local;

const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;
const DEFAULT_READ_BYTES: u64 = 512 * 1024;

struct LogState {
    path: PathBuf,
}

static LOG_STATE: OnceLock<Mutex<LogState>> = OnceLock::new();

fn state() -> Option<&'static Mutex<LogState>> {
    LOG_STATE.get()
}

pub fn init(app_data_dir: &Path) -> Result<PathBuf, String> {
    let logs_dir = app_data_dir.join("logs");
    fs::create_dir_all(&logs_dir).map_err(|e| format!("创建日志目录失败: {}", e))?;
    let path = logs_dir.join("app.log");
    if !path.exists() {
        fs::File::create(&path).map_err(|e| format!("创建日志文件失败: {}", e))?;
    }
    let _ = LOG_STATE.set(Mutex::new(LogState { path: path.clone() }));
    Ok(path)
}

pub fn log_path() -> Option<PathBuf> {
    state()
        .and_then(|s| s.lock().ok())
        .map(|g| g.path.clone())
}

pub fn logs_dir() -> Option<PathBuf> {
    log_path().and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

fn write_line(level: &str, message: &str) {
    let Some(lock) = state() else {
        return;
    };
    let Ok(guard) = lock.lock() else {
        return;
    };
    let path = &guard.path;
    if let Ok(meta) = fs::metadata(path) {
        if meta.len() >= MAX_LOG_BYTES {
            let backup = path.with_extension("log.1");
            let _ = fs::remove_file(&backup);
            let _ = fs::rename(path, &backup);
        }
    }
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("[{}] [{}] {}\n", ts, level, message);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

pub fn info(message: impl AsRef<str>) {
    write_line("INFO", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    write_line("WARN", message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    write_line("ERROR", message.as_ref());
}

pub fn read_logs(max_bytes: Option<u64>) -> Result<String, String> {
    let path = log_path().ok_or_else(|| "日志尚未初始化".to_string())?;
    if !path.exists() {
        return Ok(String::new());
    }
    let limit = max_bytes.unwrap_or(DEFAULT_READ_BYTES).max(1024);
    let mut file = fs::File::open(&path).map_err(|e| format!("打开日志失败: {}", e))?;
    let len = file
        .metadata()
        .map(|m| m.len())
        .unwrap_or(0);
    if len > limit {
        file.seek(SeekFrom::End(-(limit as i64)))
            .map_err(|e| format!("定位日志失败: {}", e))?;
    }
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| format!("读取日志失败: {}", e))?;
    if len > limit {
        if let Some(pos) = buf.find('\n') {
            buf = format!("…（仅显示末尾约 {} KB）\n{}", limit / 1024, &buf[pos + 1..]);
        }
    }
    Ok(buf)
}

pub fn clear_logs() -> Result<(), String> {
    let path = log_path().ok_or_else(|| "日志尚未初始化".to_string())?;
    fs::write(&path, "").map_err(|e| format!("清空日志失败: {}", e))?;
    let backup = path.with_extension("log.1");
    let _ = fs::remove_file(backup);
    info("日志已清空");
    Ok(())
}

pub fn export_logs(dest: &str) -> Result<(), String> {
    let path = log_path().ok_or_else(|| "日志尚未初始化".to_string())?;
    if !path.exists() {
        return Err("日志文件不存在".to_string());
    }
    let dest_path = PathBuf::from(dest);
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建导出目录失败: {}", e))?;
    }
    fs::copy(&path, &dest_path).map_err(|e| format!("导出日志失败: {}", e))?;
    info(format!("日志已导出到 {}", dest));
    Ok(())
}
