use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::NamingOptions;

pub fn build_output_name(original_name: &str, naming: &NamingOptions, force_ext: Option<&str>) -> String {
    let path = Path::new(original_name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| original_name.to_string());
    let ext = force_ext
        .map(|e| e.trim_start_matches('.').to_string())
        .or_else(|| {
            path.extension()
                .map(|e| e.to_string_lossy().to_string())
        })
        .unwrap_or_default();

    let mut parts: Vec<String> = Vec::new();

    if naming.use_original_name || (!naming.use_timestamp && !naming.use_datetime && naming.custom_text.trim().is_empty()) {
        parts.push(stem.clone());
    }

    if naming.use_timestamp {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        parts.push(ms.to_string());
    }

    if naming.use_datetime {
        let now = chrono::Local::now();
        parts.push(now.format("%Y%m%d_%H%M%S").to_string());
    }

    let custom = naming.custom_text.trim();
    if !custom.is_empty() {
        parts.push(custom.to_string());
    }

    if parts.is_empty() {
        parts.push(stem);
    }

    let base = parts.join("-");
    if ext.is_empty() {
        base
    } else {
        format!("{}.{}", base, ext)
    }
}

pub fn join_output_path(output_dir: &str, file_name: &str) -> PathBuf {
    PathBuf::from(output_dir).join(file_name)
}

pub fn ratio_output_name(file_stem: &str, ratio: &str, ratio_value: f64, ext: &str) -> String {
    let ratio_dash = ratio.replace(':', "-");
    let ratio_value_str = format!("{:.2}", ratio_value);
    let date_str = chrono::Local::now().format("%y%m%d").to_string();

    let range_enum: &[(&str, f64, f64)] = &[
        ("0.94～1.05", 0.94, 1.05),
        ("1.29～1.41", 1.29, 1.41),
        ("1.11～1.25", 1.11, 1.25),
        ("1.67～1.83", 1.67, 1.83),
        ("2.00～3.00", 2.00, 3.00),
        ("0.44～0.50", 0.44, 0.50),
        ("0.30～0.43", 0.30, 0.43),
        ("0.63～0.71", 0.63, 0.71),
        ("0.75～0.83", 0.75, 0.83),
        ("0.84～0.94", 0.84, 0.94),
        ("0.53～0.60", 0.53, 0.60),
    ];

    let matched_range = range_enum
        .iter()
        .find(|(_, lo, hi)| ratio_value >= *lo && ratio_value <= *hi);

    if let Some((range_name, _, _)) = matched_range {
        format!(
            "{}-{}({})-({})-({}).{}",
            file_stem, ratio_dash, ratio_value_str, range_name, date_str, ext
        )
    } else {
        format!(
            "{}-{}({})-({}).{}",
            file_stem, ratio_dash, ratio_value_str, date_str, ext
        )
    }
}
