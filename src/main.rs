#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod launcher;

use serde::Serialize;
use serde_json::Value;
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const CLAUDE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const CODEX_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
const MAX_JSON_BYTES: u64 = 1024 * 1024;
const REFRESH_INTERVAL: Duration = Duration::from_secs(300);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Limit {
    label: &'static str,
    used: f64,
    reset_ms: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Usage {
    status: &'static str,
    limits: Vec<Limit>,
    last_updated_ms: Option<u64>,
}

impl Usage {
    fn unavailable(status: &'static str) -> Self {
        Self { status, limits: Vec::new(), last_updated_ms: None }
    }
}

#[derive(Clone, Serialize)]
struct UsagePair {
    claude: Usage,
    codex: Usage,
}

#[derive(Default)]
struct CachedUsage {
    claude: Option<(Instant, Usage)>,
    codex: Option<(Instant, Usage)>,
    claude_stamp: Option<(SystemTime, u64)>,
}

type Cache = Mutex<CachedUsage>;

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn session_stamp(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

fn refresh_due(cached: &Option<(Instant, Usage)>, requested: bool) -> bool {
    cached.as_ref().is_none_or(|(checked, usage)| {
        checked.elapsed() >= REFRESH_INTERVAL || (requested && usage.status != "rate-limited")
    })
}

fn keep_last_reading(mut current: Usage, previous: Option<&Usage>) -> Usage {
    if current.status != "ok" {
        if let Some(previous) = previous {
            current.limits = previous.limits.clone();
            current.last_updated_ms = previous.last_updated_ms;
        }
    }
    current
}

fn read_json(reader: impl Read) -> Option<Value> {
    let mut bytes = Vec::new();
    reader.take(MAX_JSON_BYTES + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > MAX_JSON_BYTES {
        return None;
    }
    serde_json::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes)).ok()
}

fn credential_json(path: &Path) -> Option<Value> {
    read_json(std::fs::File::open(path).ok()?)
}

fn header_value(value: &Value) -> Option<&str> {
    let value = value.as_str()?.trim();
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_graphic())).then_some(value)
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(0)
        .try_proxy_from_env(false)
        .timeout(Duration::from_secs(15))
        .build()
}

fn request_json(request: ureq::Request) -> Result<Value, &'static str> {
    match request
        .set("Accept", "application/json")
        .set("Cache-Control", "no-store")
        .set("User-Agent", concat!("quota-sentinel/", env!("CARGO_PKG_VERSION"), " (Windows)"))
        .call()
    {
        Ok(response) if response.status() == 200 => {
            read_json(response.into_reader()).ok_or("unsupported")
        }
        Ok(response) if (300..400).contains(&response.status()) => Err("redirect"),
        Err(ureq::Error::Status(401 | 403, _)) => Err("rejected"),
        Err(ureq::Error::Status(429, _)) => Err("rate-limited"),
        _ => Err("unavailable"),
    }
}

fn percentage(value: &Value) -> Option<f64> {
    let number = value.as_f64().or_else(|| value.as_str()?.parse().ok())?;
    number.is_finite().then(|| number.clamp(0.0, 100.0))
}

fn claude_limits(value: &Value) -> Vec<Limit> {
    [
        ("Session", "five_hour", &["session", "five_hour"][..]),
        ("Weekly", "seven_day", &["seven_day", "weekly_all", "weekly"][..]),
    ]
    .into_iter()
    .filter_map(|(label, key, aliases)| {
        let modern = value["limits"].as_array().and_then(|limits| {
            limits.iter().find(|entry| {
                entry["kind"].as_str().is_some_and(|kind| aliases.contains(&kind))
                    && percentage(&entry["percent"]).is_some()
            })
        });
        let (entry, field) = modern.map_or((&value[key], "utilization"), |entry| (entry, "percent"));
        Some(Limit {
            label,
            used: percentage(&entry[field])?,
            reset_ms: entry["resets_at"].as_str()
                .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
                .and_then(|time| time.timestamp_millis().try_into().ok()),
        })
    })
    .collect()
}

fn codex_limits(value: &Value) -> Vec<Limit> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    [("Primary", "primary_window"), ("Secondary", "secondary_window")]
        .into_iter()
        .filter_map(|(fallback, key)| {
            let entry = &value["rate_limit"][key];
            let label = match entry["limit_window_seconds"].as_u64() {
                Some(18_000) => "5 hours",
                Some(604_800) => "Weekly",
                _ => fallback,
            };
            let reset_ms = entry["reset_at"].as_u64()
                .or_else(|| now.checked_add(entry["reset_after_seconds"].as_u64()?))
                .and_then(|seconds| seconds.checked_mul(1000));
            Some(Limit { label, used: percentage(&entry["used_percent"])?, reset_ms })
        })
        .collect()
}

fn result(response: Result<Value, &'static str>, parse: fn(&Value) -> Vec<Limit>) -> Usage {
    match response {
        Ok(response) => {
            let limits = parse(&response);
            let status = if limits.is_empty() { "unsupported" } else { "ok" };
            Usage { status, limits, last_updated_ms: (status == "ok").then(now_ms) }
        }
        Err(status) => Usage::unavailable(status),
    }
}

fn claude_session_path(home: &Path) -> PathBuf {
    home.join(".claude").join(".credentials.json")
}

fn claude(agent: &ureq::Agent, path: &Path) -> Usage {
    let Some(json) = credential_json(path) else {
        return Usage::unavailable("claude-session-missing");
    };
    let credential = json.get("claudeAiOauth").unwrap_or(&json);
    let Some(token) = header_value(&credential["accessToken"]) else {
        return Usage::unavailable("claude-session-missing");
    };
    if let Some(expiry) = credential["expiresAt"].as_u64() {
        if expiry <= now_ms() {
            return Usage::unavailable("claude-session-expired");
        }
    }
    result(request_json(agent.get(CLAUDE_URL)
        .set("Authorization", &format!("Bearer {token}"))
        .set("anthropic-beta", "oauth-2025-04-20")), claude_limits)
}

fn codex(agent: &ureq::Agent, home: &Path) -> Usage {
    let Some(json) = credential_json(&home.join(".codex").join("auth.json")) else {
        return Usage::unavailable("sign-in");
    };
    let tokens = &json["tokens"];
    let (Some(token), Some(account)) = (
        header_value(&tokens["access_token"]),
        header_value(&tokens["account_id"]),
    ) else {
        return Usage::unavailable("sign-in");
    };
    let response = request_json(agent.get(CODEX_URL)
        .set("Authorization", &format!("Bearer {token}"))
        .set("ChatGPT-Account-Id", account));
    result(response.map_err(|status| if status == "rejected" { "sign-in" } else { status }), codex_limits)
}

#[tauri::command]
async fn refresh(app: tauri::AppHandle, force: bool) -> Result<UsagePair, &'static str> {
    use tauri::Manager;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Cache>();
        let mut cache = state.lock().map_err(|_| "unavailable")?;
        let home = dirs::home_dir().ok_or("unavailable")?;
        let path = claude_session_path(&home);
        let stamp = session_stamp(&path);
        if refresh_due(&cache.claude, force || stamp != cache.claude_stamp) {
            let usage = keep_last_reading(claude(&agent(), &path), cache.claude.as_ref().map(|(_, usage)| usage));
            cache.claude = Some((Instant::now(), usage));
            cache.claude_stamp = stamp;
        }
        if refresh_due(&cache.codex, force) {
            cache.codex = Some((Instant::now(), codex(&agent(), &home)));
        }
        let (_, claude) = cache.claude.as_ref().ok_or("unavailable")?;
        let (_, codex) = cache.codex.as_ref().ok_or("unavailable")?;
        Ok(UsagePair { claude: claude.clone(), codex: codex.clone() })
    })
    .await
    .map_err(|_| "unavailable")?
}

fn main() {
    use tauri::Manager;
    tauri::Builder::default()
        .manage(Cache::default())
        .manage(Mutex::new(Dock::default()))
        .setup(|app| {
            let window = app.get_webview_window("main").expect("Missing notch window");
            if let Some(monitor) = window.primary_monitor()? {
                let area = monitor.work_area();
                let scale = monitor.scale_factor();
                let inset = (12.0 * scale).round() as i32;
                let anchor = tauri::PhysicalPosition::new(
                    area.position.x + area.size.width as i32 - inset,
                    area.position.y + (area.size.height / 2) as i32,
                );
                place_dock(&window, &monitor, anchor, Edge::Right, DockView::Grip, true)?;
                app.state::<Mutex<Dock>>().lock().expect("Dock state poisoned").anchor = Some(anchor);
            }
            window.show()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![refresh, set_dock_view, drag_dock, quit, launch_desktop])
        .run(tauri::generate_context!())
        .expect("Could not start quota-sentinel");
}

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum DesktopApp {
    Claude,
    Codex,
}

#[tauri::command]
async fn launch_desktop(provider: DesktopApp) -> Result<&'static str, &'static str> {
    #[cfg(target_os = "windows")]
    return tauri::async_runtime::spawn_blocking(move || launcher::launch(provider))
        .await
        .map_err(|_| "Could not open the desktop app.")?;

    #[cfg(not(target_os = "windows"))]
    {
        let _ = provider;
        Err("Desktop launching is available on Windows.")
    }
}

type Point = tauri::PhysicalPosition<i32>;

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DockView {
    Grip,
    Rings,
    Details,
}

#[derive(Default)]
struct Dock {
    anchor: Option<Point>,
    drag_offset: Option<Point>,
    edge: Edge,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Edge {
    Top,
    Bottom,
    Left,
    #[default]
    Right,
}

impl Edge {
    fn name(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

fn drop_edge(point: Point, origin: Point, width: u32, height: u32) -> Edge {
    let x = f64::from(point.x - origin.x) / f64::from(width.max(1));
    let y = f64::from(point.y - origin.y) / f64::from(height.max(1));
    if (0.35..=0.65).contains(&x) && (0.35..=0.65).contains(&y) {
        return Edge::Top;
    }
    [(y, Edge::Top), (1.0 - y, Edge::Bottom), (x, Edge::Left), (1.0 - x, Edge::Right)]
        .into_iter()
        .min_by(|(a, _), (b, _)| a.total_cmp(b))
        .map_or(Edge::Top, |(_, edge)| edge)
}

fn pixels(logical: f64, scale: f64) -> u32 {
    (logical * scale).round() as u32
}

fn fit_axis(value: i32, start: i32, span: u32, size: u32) -> i32 {
    value.clamp(start, start + span.saturating_sub(size) as i32)
}

fn place_dock(window: &tauri::WebviewWindow, monitor: &tauri::Monitor, anchor: Point, edge: Edge, view: DockView, snap: bool) -> tauri::Result<()> {
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let (logical_width, logical_height) = match (edge.horizontal(), view) {
        (true, DockView::Grip) => (52.0, 22.0),
        (true, DockView::Rings) => (176.0, 60.0),
        (true, DockView::Details) => (282.0, 250.0),
        (false, DockView::Grip) => (22.0, 52.0),
        (false, DockView::Rings) => (60.0, 176.0),
        (false, DockView::Details) => (350.0, 206.0),
    };
    let width = pixels(logical_width, scale).min(area.size.width);
    let height = pixels(logical_height, scale).min(area.size.height);
    let (x, y) = if snap {
        match edge {
            Edge::Left => (area.position.x, anchor.y - height as i32 / 2),
            Edge::Right => (area.position.x + (area.size.width - width) as i32, anchor.y - height as i32 / 2),
            Edge::Top => (anchor.x - width as i32 / 2, area.position.y),
            Edge::Bottom => (anchor.x - width as i32 / 2, area.position.y + (area.size.height - height) as i32),
        }
    } else {
        (anchor.x, anchor.y)
    };
    let position = Point::new(
        fit_axis(x, area.position.x, area.size.width, width),
        fit_axis(y, area.position.y, area.size.height, height),
    );
    let size = tauri::PhysicalSize::new(width, height);
    if window.outer_size()? != size {
        window.set_size(size)?;
    }
    window.set_position(position)?;
    Ok(())
}

#[tauri::command]
fn set_dock_view(window: tauri::WebviewWindow, state: tauri::State<'_, Mutex<Dock>>, view: DockView) -> Result<&'static str, String> {
    let mut dock = state.lock().map_err(|err| err.to_string())?;
    let anchor = match dock.anchor {
        Some(anchor) => anchor,
        None => {
            let origin = window.outer_position().map_err(|err| err.to_string())?;
            let size = window.outer_size().map_err(|err| err.to_string())?;
            Point::new(origin.x + size.width as i32 / 2, origin.y + size.height as i32 / 2)
        }
    };
    let monitor = window.monitor_from_point(anchor.x as f64, anchor.y as f64)
        .map_err(|err| err.to_string())?
        .or(window.current_monitor().map_err(|err| err.to_string())?)
        .ok_or("Display unavailable")?;
    place_dock(&window, &monitor, anchor, dock.edge, view, true).map_err(|err| err.to_string())?;
    dock.anchor = Some(anchor);
    Ok(dock.edge.name())
}

#[tauri::command]
fn drag_dock(window: tauri::WebviewWindow, state: tauri::State<'_, Mutex<Dock>>, phase: &str) -> Result<&'static str, String> {
    let mut dock = state.lock().map_err(|err| err.to_string())?;
    if phase == "end" {
        dock.drag_offset = None;
        return Ok(dock.edge.name());
    }
    let cursor = window.cursor_position().map_err(|err| err.to_string())?;
    let point = Point::new(cursor.x.round() as i32, cursor.y.round() as i32);
    if phase == "start" {
        let origin = window.outer_position().map_err(|err| err.to_string())?;
        dock.drag_offset = Some(Point::new(point.x - origin.x, point.y - origin.y));
    } else if let Some(offset) = dock.drag_offset {
        if let Some(monitor) = window.monitor_from_point(cursor.x, cursor.y).map_err(|err| err.to_string())? {
            if phase == "drop" {
                let area = monitor.work_area();
                let edge = drop_edge(point, area.position, area.size.width, area.size.height);
                place_dock(&window, &monitor, point, edge, DockView::Grip, true).map_err(|err| err.to_string())?;
                dock.edge = edge;
                dock.anchor = Some(point);
            } else {
                place_dock(&window, &monitor, Point::new(point.x - offset.x, point.y - offset.y), dock.edge, DockView::Grip, false)
                    .map_err(|err| err.to_string())?;
            }
        }
    }
    Ok(dock.edge.name())
}

#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod tests {
    #[test]
    fn claude_failures_preserve_usage_and_manual_refresh_respects_rate_limits() {
        let reading = Usage {
            status: "ok",
            limits: vec![Limit { label: "Session", used: 42.0, reset_ms: Some(1000) }],
            last_updated_ms: Some(500),
        };
        let stale = keep_last_reading(Usage::unavailable("rejected"), Some(&reading));
        assert_eq!(stale.status, "rejected");
        assert_eq!(stale.limits[0].used, 42.0);
        assert_eq!(stale.last_updated_ms, Some(500));
        let cache = Some((Instant::now(), stale));
        assert!(!refresh_due(&cache, false));
        assert!(refresh_due(&cache, true));
        let limited = Some((Instant::now(), Usage::unavailable("rate-limited")));
        assert!(!refresh_due(&limited, true));
    }

    use super::*;
    use std::{io::Write, net::TcpListener};

    #[test]
    fn drops_choose_edges_on_the_destination_display() {
        let origin = Point::new(-1920, -100);
        let edge = |x, y| drop_edge(Point::new(x, y), origin, 1920, 1080);
        assert_eq!(edge(-960, 440), Edge::Top);
        assert_eq!(edge(-960, 960), Edge::Bottom);
        assert_eq!(edge(-1900, 440), Edge::Left);
        assert_eq!(edge(-20, 440), Edge::Right);
    }

    #[test]
    fn clamps_entire_dock_inside_negative_and_positive_display_bounds() {
        assert_eq!(fit_axis(-25, -1920, 1920, 50), -50);
        assert_eq!(fit_axis(-25, 0, 1920, 50), 0);
        assert_eq!(fit_axis(1900, 0, 1920, 304), 1616);
        assert_eq!(fit_axis(1050, 0, 1040, 156), 884);
    }

    #[test]
    fn refuses_redirects_without_contacting_the_destination() {
        let endpoint = TcpListener::bind("127.0.0.1:0").unwrap();
        let destination = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", endpoint.local_addr().unwrap());
        let location = destination.local_addr().unwrap();
        destination.set_nonblocking(true).unwrap();
        let server = std::thread::spawn(move || {
            let (mut connection, _) = endpoint.accept().unwrap();
            connection.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                connection.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            assert!(String::from_utf8(request).unwrap().contains("Bearer dummy-token"));
            write!(connection, "HTTP/1.1 302 Found\r\nLocation: http://{location}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        let request = agent().get(&url).set("Authorization", "Bearer dummy-token");
        assert_eq!(request_json(request).unwrap_err(), "redirect");
        server.join().unwrap();
        assert_eq!(destination.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    }

    #[test]
    fn projects_only_usage_fields_and_rejects_unsafe_header_values() {
        let response = serde_json::json!({
            "limits": [{ "kind": "session", "percent": 21, "resets_at": "2026-11-24T10:00:00Z" }],
            "secret": "synthetic-secret"
        });
        let limits = claude_limits(&response);
        assert_eq!(limits[0].used, 21.0);
        assert!(limits[0].reset_ms.is_some());
        assert!(!serde_json::to_string(&limits).unwrap().contains("synthetic-secret"));
        assert!(header_value(&serde_json::json!("test\r\nInjected: 1")).is_none());
        let response = serde_json::json!({
            "rate_limit": { "secondary_window": { "used_percent": 52, "limit_window_seconds": 604800 } }
        });
        let limits = codex_limits(&response);
        assert_eq!(limits[0].label, "Weekly");
        assert_eq!(limits[0].used, 52.0);
    }
}
