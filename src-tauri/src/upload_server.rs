use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

fn server_port() -> &'static Arc<Mutex<Option<u16>>> {
    static INSTANCE: OnceLock<Arc<Mutex<Option<u16>>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Arc::new(Mutex::new(None)))
}

const UPLOAD_PAGE: &str = r#"<!DOCTYPE html>
<html lang="zh">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>上传文件</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, sans-serif; background: #f5f5f5; display: flex; justify-content: center; align-items: center; min-height: 100vh; }
.card { background: #fff; border-radius: 12px; padding: 40px; width: 90%; max-width: 500px; box-shadow: 0 2px 12px rgba(0,0,0,0.08); }
h1 { font-size: 20px; margin-bottom: 20px; color: #333; }
.drop-zone { border: 2px dashed #ccc; border-radius: 8px; padding: 40px; text-align: center; cursor: pointer; transition: all 0.2s; margin-bottom: 16px; }
.drop-zone:hover, .drop-zone.dragover { border-color: #007bff; background: #f0f7ff; }
.drop-zone p { color: #666; font-size: 14px; }
.drop-zone .icon { font-size: 36px; margin-bottom: 8px; }
input[type="file"] { display: none; }
.btn { display: inline-block; padding: 10px 24px; background: #007bff; color: #fff; border: none; border-radius: 6px; font-size: 14px; cursor: pointer; transition: background 0.2s; }
.btn:hover { background: #0056b3; }
.btn:disabled { background: #ccc; cursor: not-allowed; }
.status { margin-top: 16px; font-size: 13px; color: #666; }
.status.success { color: #28a745; }
.status.error { color: #dc3545; }
.file-list { margin-top: 12px; font-size: 13px; color: #555; }
.file-list div { padding: 4px 0; }
</style>
</head>
<body>
<div class="card">
  <h1>上传文件</h1>
  <div class="drop-zone" id="dropZone">
    <div class="icon">📁</div>
    <p>拖拽文件到此处，或点击选择</p>
  </div>
  <input type="file" id="fileInput" multiple>
  <div style="text-align:center">
    <button class="btn" id="uploadBtn" disabled>上传</button>
  </div>
  <div class="status" id="status"></div>
  <div class="file-list" id="fileList"></div>
</div>
<script>
const dropZone = document.getElementById('dropZone');
const fileInput = document.getElementById('fileInput');
const uploadBtn = document.getElementById('uploadBtn');
const status = document.getElementById('status');
const fileList = document.getElementById('fileList');
let files = [];

dropZone.addEventListener('click', () => fileInput.click());
dropZone.addEventListener('dragover', e => { e.preventDefault(); dropZone.classList.add('dragover'); });
dropZone.addEventListener('dragleave', () => dropZone.classList.remove('dragover'));
dropZone.addEventListener('drop', e => { e.preventDefault(); dropZone.classList.remove('dragover'); addFiles(e.dataTransfer.files); });
fileInput.addEventListener('change', e => addFiles(e.target.files));

function addFiles(f) {
  files = [...files, ...Array.from(f)];
  updateList();
}

function updateList() {
  uploadBtn.disabled = files.length === 0;
  fileList.innerHTML = files.map(f => `<div>${f.name} (${(f.size/1024).toFixed(1)} KB)</div>`).join('');
}

uploadBtn.addEventListener('click', async () => {
  if (files.length === 0) return;
  uploadBtn.disabled = true;
  status.textContent = '上传中...';
  status.className = 'status';
  let ok = 0, fail = 0;
  for (const file of files) {
    try {
      const form = new FormData();
      form.append('file', file);
      const res = await fetch('/upload', { method: 'POST', body: form });
      if (res.ok) ok++; else fail++;
    } catch { fail++; }
  }
  status.textContent = fail > 0 ? `完成: ${ok} 成功, ${fail} 失败` : `全部上传成功: ${ok} 个文件`;
  status.className = fail > 0 ? 'status error' : 'status success';
  files = [];
  updateList();
  uploadBtn.disabled = false;
});
</script>
</body>
</html>"#;

fn get_local_ip() -> String {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect("8.8.8.8:80").unwrap();
    socket.local_addr().unwrap().ip().to_string()
}

pub fn start_server(input_dir: String) -> Result<String, String> {
    let store = server_port();
    {
        let guard = store.lock().unwrap();
        if let Some(port) = *guard {
            let ip = get_local_ip();
            return Ok(format!("http://{}:{}", ip, port));
        }
    }

    let server = tiny_http::Server::http("0.0.0.0:0").map_err(|e| format!("启动服务器失败: {}", e))?;
    let port = server.server_addr().to_ip().map(|a| a.port()).unwrap_or(8080);

    {
        let mut guard = store.lock().unwrap();
        *guard = Some(port);
    }

    let ip = get_local_ip();
    let url = format!("http://{}:{}", ip, port);
    let input_path = PathBuf::from(&input_dir);

    thread::spawn(move || {
        for mut request in server.incoming_requests() {
            match request.method() {
                tiny_http::Method::Get => {
                    let response = tiny_http::Response::from_string(UPLOAD_PAGE)
                        .with_header(
                            tiny_http::Header::from_bytes(
                                &b"Content-Type"[..],
                                &b"text/html; charset=utf-8"[..],
                            )
                            .unwrap(),
                        );
                    let _ = request.respond(response);
                }
                tiny_http::Method::Post if request.url() == "/upload" => {
                    let content_type = request
                        .headers()
                        .iter()
                        .find(|h| h.field.equiv("Content-Type"))
                        .map(|h| h.value.to_string())
                        .unwrap_or_default();

                    let boundary = if content_type.contains("boundary=") {
                        content_type
                            .split("boundary=")
                            .nth(1)
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    } else {
                        let response = tiny_http::Response::from_string("Missing boundary")
                            .with_status_code(400);
                        let _ = request.respond(response);
                        continue;
                    };

                    let mut body = Vec::new();
                    let _ = request.as_reader().read_to_end(&mut body);

                    match extract_file_from_multipart(&body, &boundary) {
                        Some((filename, data)) => {
                            let dest = input_path.join(&filename);
                            match std::fs::write(&dest, &data) {
                                Ok(_) => {
                                    let response =
                                        tiny_http::Response::from_string("OK").with_status_code(200);
                                    let _ = request.respond(response);
                                }
                                Err(e) => {
                                    let response =
                                        tiny_http::Response::from_string(format!("写入失败: {}", e))
                                            .with_status_code(500);
                                    let _ = request.respond(response);
                                }
                            }
                        }
                        None => {
                            let response =
                                tiny_http::Response::from_string("未找到文件").with_status_code(400);
                            let _ = request.respond(response);
                        }
                    }
                }
                _ => {
                    let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                    let _ = request.respond(response);
                }
            }
        }
    });

    Ok(url)
}

fn extract_file_from_multipart(body: &[u8], boundary: &str) -> Option<(String, Vec<u8>)> {
    let boundary_delim = format!("--{}", boundary).into_bytes();
    let header_sep = b"\r\n\r\n";

    // 按 boundary 分割
    let mut start = 0;
    while start < body.len() {
        // 查找下一个 boundary
        let boundary_pos = find_subsequence(&body[start..], &boundary_delim)?;
        let part_start = start + boundary_pos + boundary_delim.len();

        // 跳过 \r\n
        let part_start = if body[part_start..].starts_with(b"\r\n") {
            part_start + 2
        } else {
            part_start
        };

        // 查找 header 和 body 的分界 \r\n\r\n
        let header_end = find_subsequence(&body[part_start..], header_sep)?;
        let headers = &body[part_start..part_start + header_end];
        let content_start = part_start + header_end + 4;

        // 检查是否是文件字段
        let headers_str = String::from_utf8_lossy(headers);
        if headers_str.contains("filename=\"") {
            // 提取文件名
            let filename = if let Some(s) = headers_str.find("filename=\"") {
                let rest = &headers_str[s + 10..];
                if let Some(e) = rest.find('"') {
                    rest[..e].to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            if !filename.is_empty() {
                // 查找下一个 boundary 作为内容结束
                let content_end = find_subsequence(&body[content_start..], &boundary_delim)
                    .map(|p| content_start + p)
                    .unwrap_or(body.len());

                // 去掉末尾的 \r\n
                let data = if content_end > content_start + 2
                    && &body[content_end - 2..content_end] == b"\r\n"
                {
                    &body[content_start..content_end - 2]
                } else {
                    &body[content_start..content_end]
                };

                return Some((filename, data.to_vec()));
            }
        }

        // 移动到下一个 part
        let next_boundary = find_subsequence(&body[content_start..], &boundary_delim);
        match next_boundary {
            Some(p) => start = content_start + p,
            None => break,
        }
    }

    None
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
