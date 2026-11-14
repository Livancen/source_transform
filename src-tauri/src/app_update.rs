//! 可选的 GitHub 更新加速。
//!
//! 用户只保存加速站的 https 前缀。检查 `latest.json` 和下载安装包时，
//! 都把该前缀拼在 GitHub 原链接前面；失败再回退到原链接。
//! 安装包仍由 updater 插件用公钥验签。

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};
use url::Url;

/// 与 `tauri.conf.json` 里 `plugins.updater.endpoints` 保持一致。
const UPDATE_ENDPOINT: &str =
    "https://github.com/Livancen/source_transform/releases/latest/download/latest.json";

const CHECK_TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// 两次读到数据之间允许的最长空闲。慢速传输不会触发，卡住才会回退。
const READ_STALL_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
struct Pending {
    update: Update,
    original_url: Url,
    busy: bool,
}

#[derive(Default)]
pub struct PendingUpdate {
    inner: Mutex<Option<Pending>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub version: String,
    pub body: Option<String>,
    pub mirrored: bool,
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum AppDownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        chunk_length: usize,
    },
    Finished,
    Fallback,
}

fn normalize_mirror(input: &str) -> Result<Option<String>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let url = Url::parse(&with_scheme).map_err(|_| "加速域名不是合法的网址".to_string())?;
    if url.scheme() != "https" {
        return Err("加速域名必须使用 https".into());
    }
    let host = url.host_str().ok_or("加速域名缺少主机名")?;
    let host_l = host.to_ascii_lowercase();
    if host_l == "github.com"
        || host_l.ends_with(".github.com")
        || host_l.ends_with("githubusercontent.com")
    {
        return Err("请填写加速站域名，不是 GitHub 的地址".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("加速域名不能包含账号密码".into());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("加速域名不能带参数或锚点".into());
    }
    let host_display = match url.host() {
        Some(url::Host::Ipv6(ip)) => format!("[{ip}]"),
        Some(_) => host.to_string(),
        None => return Err("加速域名缺少主机名".into()),
    };
    let port = url.port().map(|p| format!(":{p}")).unwrap_or_default();
    let path = url.path().trim_end_matches('/');
    let path = if path == "/" { "" } else { path };
    Ok(Some(format!("https://{host_display}{port}{path}")))
}

fn prefix_url(mirror_base: &str, original: &Url) -> Result<Url, String> {
    let joined = format!("{mirror_base}/{original}");
    Url::parse(&joined).map_err(|_| "无法拼接加速地址".to_string())
}

fn strip_mirror(mirror_base: &str, original: &Url) -> Option<Url> {
    let rest = original
        .as_str()
        .strip_prefix(mirror_base)?
        .strip_prefix('/')?;
    Url::parse(rest).ok()
}

fn official_endpoint() -> Result<Url, String> {
    Url::parse(UPDATE_ENDPOINT).map_err(|_| "更新地址配置无效".to_string())
}

async fn check_one(app: &AppHandle, endpoint: Url) -> Result<Option<Update>, String> {
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| e.to_string())?
        .timeout(CHECK_TIMEOUT)
        .configure_client(|client| {
            client
                .connect_timeout(CONNECT_TIMEOUT)
                .read_timeout(READ_STALL_TIMEOUT)
        })
        .build()
        .map_err(|e| e.to_string())?;
    updater.check().await.map_err(|e| e.to_string())
}

fn resolve_download_urls(mirror: Option<&str>, raw: &Url) -> Result<(Url, Url), String> {
    let Some(base) = mirror else {
        return Ok((raw.clone(), raw.clone()));
    };
    if let Some(inner) = strip_mirror(base, raw) {
        return Ok((raw.clone(), inner));
    }
    Ok((prefix_url(base, raw)?, raw.clone()))
}

#[tauri::command]
pub async fn check_app_update(
    app: AppHandle,
    mirror: Option<String>,
) -> Result<Option<UpdateInfo>, String> {
    let mirror = normalize_mirror(mirror.as_deref().unwrap_or(""))?;
    let state = app.state::<PendingUpdate>();
    {
        let guard = state
            .inner
            .lock()
            .map_err(|_| "更新状态不可用".to_string())?;
        if guard.as_ref().is_some_and(|pending| pending.busy) {
            return Err("更新正在下载，请稍候".into());
        }
    }

    let official = official_endpoint()?;
    let mut found: Option<Update> = None;
    let mut mirror_said_none = false;

    if let Some(ref base) = mirror {
        let endpoint = prefix_url(base, &official)?;
        crate::logger::info(format!("通过加速站检查更新: {base}"));
        match check_one(&app, endpoint).await {
            Ok(Some(update)) => found = Some(update),
            Ok(None) => {
                mirror_said_none = true;
                crate::logger::info("加速站检查完成，没有新版本");
            }
            Err(err) => {
                crate::logger::warn(format!("加速站检查失败，改用 GitHub 直连: {err}"));
            }
        }
    }

    if found.is_none() && !mirror_said_none {
        match check_one(&app, official).await {
            Ok(update) => found = update,
            Err(err) => {
                return Err(if mirror.is_some() {
                    format!("检查更新失败（加速站与 GitHub 都不可用）: {err}")
                } else {
                    format!("检查更新失败: {err}")
                });
            }
        }
    }

    let Some(mut update) = found else {
        let mut guard = state
            .inner
            .lock()
            .map_err(|_| "更新状态不可用".to_string())?;
        *guard = None;
        return Ok(None);
    };

    let (download_url, original_url) =
        resolve_download_urls(mirror.as_deref(), &update.download_url)?;
    let mirrored = download_url != original_url;
    if mirrored {
        crate::logger::info(format!("更新包将优先走加速站: {download_url}"));
    }
    update.download_url = download_url;

    let info = UpdateInfo {
        current_version: update.current_version.clone(),
        version: update.version.clone(),
        body: update.body.clone(),
        mirrored,
    };
    let mut guard = state
        .inner
        .lock()
        .map_err(|_| "更新状态不可用".to_string())?;
    *guard = Some(Pending {
        update,
        original_url,
        busy: false,
    });
    Ok(Some(info))
}

async fn download_bytes(
    update: &Update,
    on_event: &Channel<AppDownloadEvent>,
) -> Result<Vec<u8>, String> {
    let mut started = false;
    update
        .download(
            |chunk_length, content_length| {
                if !started {
                    started = true;
                    let _ = on_event.send(AppDownloadEvent::Started { content_length });
                }
                let _ = on_event.send(AppDownloadEvent::Progress { chunk_length });
            },
            || {
                let _ = on_event.send(AppDownloadEvent::Finished);
            },
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_and_install_app_update(
    app: AppHandle,
    on_event: Channel<AppDownloadEvent>,
) -> Result<(), String> {
    let state = app.state::<PendingUpdate>();
    let mut pending = {
        let mut guard = state
            .inner
            .lock()
            .map_err(|_| "更新状态不可用".to_string())?;
        let pending = guard.as_mut().ok_or("没有待安装的更新，请重新检查")?;
        if pending.busy {
            return Err("更新已在进行".into());
        }
        pending.busy = true;
        pending.clone()
    };

    let result = async {
        let bytes = if pending.update.download_url != pending.original_url {
            match download_bytes(&pending.update, &on_event).await {
                Ok(bytes) => bytes,
                Err(err) => {
                    crate::logger::warn(format!("加速站下载失败，改用 GitHub 直连: {err}"));
                    let _ = on_event.send(AppDownloadEvent::Fallback);
                    pending.update.download_url = pending.original_url.clone();
                    download_bytes(&pending.update, &on_event)
                        .await
                        .map_err(|direct| {
                            format!("更新包下载失败。加速站：{err}；GitHub 直连：{direct}")
                        })?
                }
            }
        } else {
            download_bytes(&pending.update, &on_event).await?
        };
        pending.update.install(&bytes).map_err(|e| e.to_string())
    }
    .await;

    if result.is_err() {
        let mut guard = state
            .inner
            .lock()
            .map_err(|_| "更新状态不可用".to_string())?;
        if let Some(current) = guard.as_mut() {
            current.busy = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_matches_tauri_conf() {
        let conf = std::fs::read_to_string("tauri.conf.json").expect("read tauri.conf.json");
        assert!(
            conf.contains(UPDATE_ENDPOINT),
            "UPDATE_ENDPOINT 与 tauri.conf.json 不一致"
        );
    }

    #[test]
    fn normalizes_mirror_and_prefixes_github_url() {
        let base = normalize_mirror(" https://ghfast.top/ ").unwrap().unwrap();
        assert_eq!(base, "https://ghfast.top");
        let raw = Url::parse(UPDATE_ENDPOINT).unwrap();
        let prefixed = prefix_url(&base, &raw).unwrap();
        assert_eq!(
            prefixed.as_str(),
            "https://ghfast.top/https://github.com/Livancen/source_transform/releases/latest/download/latest.json"
        );
        assert_eq!(strip_mirror(&base, &prefixed).unwrap(), raw);
    }

    #[test]
    fn adds_https_and_rejects_github_host() {
        assert_eq!(
            normalize_mirror("mirror.example/proxy").unwrap().unwrap(),
            "https://mirror.example/proxy"
        );
        assert!(normalize_mirror("").unwrap().is_none());
        assert!(normalize_mirror("http://mirror.example").is_err());
        assert!(normalize_mirror("https://github.com/foo").is_err());
        assert!(normalize_mirror("https://mirror.example/a?b=1").is_err());
    }

    #[test]
    fn already_prefixed_url_is_not_doubled() {
        let base = "https://ghfast.top";
        let raw = Url::parse(UPDATE_ENDPOINT).unwrap();
        let prefixed = prefix_url(base, &raw).unwrap();
        let (download, original) = resolve_download_urls(Some(base), &prefixed).unwrap();
        assert_eq!(download, prefixed);
        assert_eq!(original, raw);
    }
}
