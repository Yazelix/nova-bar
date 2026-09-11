use std::env;
use std::path::Path;
use std::process;

const SESSION_TERMINAL_ENV: &str = "YAZELIX_SESSION_TERMINAL";
const UNKNOWN_SESSION_TERMINAL: &str = "unknown";
const CODEX_QUOTA_MAX_AGE_SECONDS: u64 = 60;
const CODEX_QUOTA_ERROR_BACKOFF_SECONDS: u64 = 120;
const CODEX_HISTORY_MAX_AGE_SECONDS: u64 = 600;
const CODEX_HISTORY_ERROR_BACKOFF_SECONDS: u64 = 1_800;

fn main() {
    match run(env::args().skip(1).collect()) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    let Some((command, rest)) = args.split_first() else {
        return Err(
            "expected command: claude, codex, cpu, opencode_go, ram, term, version, or render-nova-runtime"
                .to_string(),
        );
    };
    match command.as_str() {
        "claude" => run_claude_usage(rest),
        "codex" => run_codex_usage(rest),
        "opencode_go" => run_opencode_go_usage(rest),
        "render-nova-runtime" => run_render_nova_runtime(rest),
        "version" => run_version(rest),
        "cpu" => run_cpu_usage(rest),
        "ram" => run_ram_usage(rest),
        "term" => run_term(rest),
        _ => Err(format!("unknown widget command: {command}")),
    }
}

fn run_term(args: &[String]) -> Result<String, String> {
    if !args.is_empty() {
        return Err("term usage: nova_bar_widget term".to_string());
    }
    Ok(render_term_from_env(|key| env::var(key).ok()))
}

fn render_term_from_env<F>(get_env: F) -> String
where
    F: FnMut(&str) -> Option<String>,
{
    format!(" {}", session_terminal_label_from_env(get_env))
}

fn session_terminal_label_from_env<F>(mut get_env: F) -> String
where
    F: FnMut(&str) -> Option<String>,
{
    get_env(SESSION_TERMINAL_ENV)
        .and_then(|value| normalize_session_terminal_label(&value))
        .unwrap_or_else(|| UNKNOWN_SESSION_TERMINAL.to_string())
}

fn normalize_session_terminal_label(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn run_version(args: &[String]) -> Result<String, String> {
    let [flag, runtime_dir] = args else {
        return Err(
            "version usage: nova_bar_widget version --runtime-dir <runtime-dir>".to_string(),
        );
    };
    if flag != "--runtime-dir" {
        return Err("version expects --runtime-dir <runtime-dir>".to_string());
    }
    runtime_badge(Path::new(runtime_dir))
}

fn runtime_badge(runtime_dir: &Path) -> Result<String, String> {
    let identity_path = runtime_dir.join("runtime_identity.json");
    let raw = std::fs::read_to_string(&identity_path).map_err(|error| {
        format!(
            "failed to read Nova runtime identity at {}: {error}",
            identity_path.display()
        )
    })?;
    let identity: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "failed to parse Nova runtime identity at {}: {error}",
            identity_path.display()
        )
    })?;
    let field = |name| {
        identity
            .get(name)
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                format!(
                    "Nova runtime identity at {} is missing string field `{name}`",
                    identity_path.display()
                )
            })
    };
    runtime_badge_for_status_bar(field("version")?, field("channel")?)
}

fn runtime_badge_for_status_bar(version: &str, channel: &str) -> Result<String, String> {
    if !matches!(channel, "stable" | "main" | "edge") {
        return Err(format!("unsupported Nova channel: {channel}"));
    }
    let channel = channel.to_ascii_uppercase();
    let compact_version = if version == "dev" {
        "NOVA DEV".to_string()
    } else {
        let (core, beta) = version
            .split_once("-beta.")
            .map_or((version, None), |(core, beta)| (core, Some(beta)));
        let components = core.split('.').collect::<Vec<_>>();
        let valid_component = |value: &&str| {
            !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
        };
        if components.len() != 3
            || !components.iter().all(valid_component)
            || beta.is_some_and(|value| {
                value.is_empty() || !value.chars().all(|character| character.is_ascii_digit())
            })
        {
            return Err(format!("unsupported Nova version: {version}"));
        }
        match beta {
            Some(beta) => format!("NOVA β{beta}"),
            None => format!("NOVA {}.{}", components[0], components[1]),
        }
    };
    Ok(format!("{compact_version} {channel}"))
}

fn run_render_nova_runtime(args: &[String]) -> Result<String, String> {
    let [flag, raw_request] = args else {
        return Err(
            "render-nova-runtime usage: nova_bar_widget render-nova-runtime --json <request-json>"
                .to_string(),
        );
    };
    if flag != "--json" {
        return Err("render-nova-runtime expects --json <request-json>".to_string());
    }

    let config: nova_bar::NovaRuntimeBarConfig = serde_json::from_str(raw_request)
        .map_err(|error| format!("invalid runtime bar config json: {error}"))?;
    let plugin_block = nova_bar::render_nova_runtime_plugin_block(&config)
        .map_err(|error| format!("failed to render Nova runtime bar: {}", error.code()))?;
    let background_plugin_block = nova_bar::render_nova_runtime_background_plugin_block(&config)
        .map_err(|error| format!("failed to render Nova runtime bar: {}", error.code()))?;

    serde_json::to_string(&nova_bar::NovaRuntimeBarRender {
        schema_version: nova_bar::NOVA_RUNTIME_BAR_RENDER_SCHEMA_VERSION,
        plugin_block,
        background_plugin_block,
    })
    .map_err(|error| format!("failed to encode Nova runtime bar render: {error}"))
}

#[derive(Debug, Clone)]
struct WidgetCommandChrome {
    frame: String,
    separator: String,
    first: bool,
    configured: bool,
}

impl Default for WidgetCommandChrome {
    fn default() -> Self {
        Self {
            frame: nova_bar::WIDGET_FRAME_NONE.to_string(),
            separator: nova_bar::WIDGET_SEPARATOR_DOT.to_string(),
            first: false,
            configured: false,
        }
    }
}

impl WidgetCommandChrome {
    fn configured(&self) -> bool {
        self.configured
    }
}

fn parse_widget_chrome_arg<'a>(
    arg: &str,
    iter: &mut std::slice::Iter<'a, String>,
    chrome: &mut WidgetCommandChrome,
) -> Result<bool, String> {
    match arg {
        "--widget-frame" => {
            chrome.configured = true;
            chrome.frame = iter
                .next()
                .ok_or_else(|| "--widget-frame requires a value".to_string())?
                .to_string();
            Ok(true)
        }
        "--widget-separator" => {
            chrome.configured = true;
            chrome.separator = iter
                .next()
                .ok_or_else(|| "--widget-separator requires a value".to_string())?
                .to_string();
            Ok(true)
        }
        "--widget-first" => {
            chrome.configured = true;
            chrome.first = parse_bool_arg("--widget-first", iter.next())?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn render_widget_command_segment(
    body: String,
    chrome: &WidgetCommandChrome,
) -> Result<String, String> {
    if !chrome.configured() {
        return Ok(body);
    }
    nova_bar::render_configured_widget_segment(
        &body,
        &chrome.frame,
        &chrome.separator,
        chrome.first,
    )
    .map_err(|error| format!("invalid widget chrome: {}", error.code()))
}

fn run_claude_usage(args: &[String]) -> Result<String, String> {
    let mut cache_path = None;
    let mut display = nova_bar::AgentUsageDisplay::Both;
    let mut periods = vec![
        nova_bar::AgentUsagePeriod::FiveHour,
        nova_bar::AgentUsagePeriod::Weekly,
    ];
    let mut max_age_seconds = 600;
    let mut error_backoff_seconds = 1_800;
    let mut timeout_ms = 5_000;
    let mut chrome = WidgetCommandChrome::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if parse_widget_chrome_arg(arg, &mut iter, &mut chrome)? {
            continue;
        }
        match arg.as_str() {
            "--cache" => {
                cache_path = Some(std::path::PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--cache requires a value".to_string())?,
                ));
            }
            "--display" => {
                display = parse_agent_usage_display(
                    iter.next()
                        .ok_or_else(|| "--display requires a value".to_string())?,
                )?;
            }
            "--periods" => {
                periods = parse_agent_usage_periods(
                    iter.next()
                        .ok_or_else(|| "--periods requires a value".to_string())?,
                )?;
            }
            "--max-age-seconds" => {
                max_age_seconds = parse_u64_arg("--max-age-seconds", iter.next())?;
            }
            "--error-backoff-seconds" => {
                error_backoff_seconds = parse_u64_arg("--error-backoff-seconds", iter.next())?;
            }
            "--timeout-ms" => {
                timeout_ms = parse_u64_arg("--timeout-ms", iter.next())?.max(1);
            }
            _ => {
                return Err(format!("unknown claude_usage argument: {arg}"));
            }
        }
    }
    let cache_path = cache_path
        .or_else(nova_bar::claude_usage_cache_path_from_env)
        .or_else(|| default_cache_path("claude_usage_cache_v1.json"))
        .ok_or_else(|| {
            "claude usage: nova_bar_widget claude [--cache <path>] [--display quota|token|both] [--periods 5h,week]".to_string()
        })?;
    let path_var = env::var_os("PATH");
    let options = nova_bar::ClaudeUsageWidgetOptions {
        cache_path: &cache_path,
        path_var: path_var.as_deref(),
        now_unix_seconds: unix_time_seconds(),
        max_age_seconds,
        error_backoff_seconds,
        timeout: std::time::Duration::from_millis(timeout_ms),
        display,
        periods: &periods,
    };
    let text = if chrome.configured() {
        nova_bar::claude_usage_widget_body_text(options)?
    } else {
        nova_bar::claude_usage_widget_text(options)?
    };
    render_widget_command_segment(text, &chrome)
}

fn run_codex_usage(args: &[String]) -> Result<String, String> {
    let mut cache_path = None;
    let mut display = nova_bar::AgentUsageDisplay::Quota;
    let mut periods = vec![
        nova_bar::AgentUsagePeriod::FiveHour,
        nova_bar::AgentUsagePeriod::Weekly,
    ];
    let mut max_age_seconds = None;
    let mut error_backoff_seconds = None;
    let mut timeout_ms = 5_000;
    let mut chrome = WidgetCommandChrome::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if parse_widget_chrome_arg(arg, &mut iter, &mut chrome)? {
            continue;
        }
        match arg.as_str() {
            "--cache" => {
                cache_path = Some(std::path::PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--cache requires a value".to_string())?,
                ));
            }
            "--display" => {
                display = parse_agent_usage_display(
                    iter.next()
                        .ok_or_else(|| "--display requires a value".to_string())?,
                )?;
            }
            "--periods" => {
                periods = parse_agent_usage_periods(
                    iter.next()
                        .ok_or_else(|| "--periods requires a value".to_string())?,
                )?;
            }
            "--max-age-seconds" => {
                max_age_seconds = Some(parse_u64_arg("--max-age-seconds", iter.next())?);
            }
            "--error-backoff-seconds" => {
                error_backoff_seconds =
                    Some(parse_u64_arg("--error-backoff-seconds", iter.next())?);
            }
            "--timeout-ms" => {
                timeout_ms = parse_u64_arg("--timeout-ms", iter.next())?.max(1);
            }
            _ => {
                return Err(format!("unknown codex_usage argument: {arg}"));
            }
        }
    }
    let cache_path = cache_path
        .or_else(nova_bar::codex_usage_cache_path_from_env)
        .or_else(|| {
            default_cache_path(&format!(
                "codex_usage_cache_v{}.json",
                nova_bar::CODEX_USAGE_CACHE_SCHEMA_VERSION
            ))
        })
        .ok_or_else(|| {
            "codex usage: nova_bar_widget codex [--cache <path>] [--display quota|token|both] [--periods 5h,week]".to_string()
        })?;
    let path_var = env::var_os("PATH");
    let (default_max_age, default_error_backoff) = match display {
        nova_bar::AgentUsageDisplay::Quota => (
            CODEX_QUOTA_MAX_AGE_SECONDS,
            CODEX_QUOTA_ERROR_BACKOFF_SECONDS,
        ),
        _ => (
            CODEX_HISTORY_MAX_AGE_SECONDS,
            CODEX_HISTORY_ERROR_BACKOFF_SECONDS,
        ),
    };
    let options = nova_bar::CodexUsageWidgetOptions {
        cache_path: &cache_path,
        path_var: path_var.as_deref(),
        now_unix_seconds: unix_time_seconds(),
        max_age_seconds: max_age_seconds.unwrap_or(default_max_age),
        error_backoff_seconds: error_backoff_seconds.unwrap_or(default_error_backoff),
        timeout: std::time::Duration::from_millis(timeout_ms),
        display,
        periods: &periods,
    };
    let text = if chrome.configured() {
        nova_bar::codex_usage_widget_body_text(options)?
    } else {
        nova_bar::codex_usage_widget_text(options)?
    };
    render_widget_command_segment(text, &chrome)
}

fn run_opencode_go_usage(args: &[String]) -> Result<String, String> {
    let mut cache_path = None;
    let mut db_paths = Vec::new();
    let mut display = nova_bar::AgentUsageDisplay::Both;
    let mut periods = vec![
        nova_bar::AgentUsagePeriod::FiveHour,
        nova_bar::AgentUsagePeriod::Weekly,
        nova_bar::AgentUsagePeriod::Monthly,
    ];
    let mut max_age_seconds = 600;
    let mut error_backoff_seconds = 1_800;
    let mut chrome = WidgetCommandChrome::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if parse_widget_chrome_arg(arg, &mut iter, &mut chrome)? {
            continue;
        }
        match arg.as_str() {
            "--cache" => {
                cache_path = Some(std::path::PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--cache requires a value".to_string())?,
                ));
            }
            "--db" => {
                db_paths.push(std::path::PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--db requires a value".to_string())?,
                ));
            }
            "--display" => {
                display = parse_agent_usage_display(
                    iter.next()
                        .ok_or_else(|| "--display requires a value".to_string())?,
                )?;
            }
            "--periods" => {
                periods = parse_agent_usage_periods(
                    iter.next()
                        .ok_or_else(|| "--periods requires a value".to_string())?,
                )?;
            }
            "--max-age-seconds" => {
                max_age_seconds = parse_u64_arg("--max-age-seconds", iter.next())?;
            }
            "--error-backoff-seconds" => {
                error_backoff_seconds = parse_u64_arg("--error-backoff-seconds", iter.next())?;
            }
            _ => {
                return Err(format!("unknown opencode_go_usage argument: {arg}"));
            }
        }
    }
    let cache_path = cache_path
        .or_else(nova_bar::opencode_go_usage_cache_path_from_env)
        .or_else(|| default_cache_path("opencode_go_usage_cache_v1.json"))
        .ok_or_else(|| {
            "opencode_go usage: nova_bar_widget opencode_go [--cache <path>] [--db <path>] [--display quota|token|both] [--periods 5h,week,month]".to_string()
        })?;
    if db_paths.is_empty() {
        db_paths = nova_bar::opencode_db_candidates_from_env();
    }
    let options = nova_bar::OpenCodeGoUsageWidgetOptions {
        cache_path: &cache_path,
        db_paths: &db_paths,
        now_unix_seconds: unix_time_seconds(),
        max_age_seconds,
        error_backoff_seconds,
        display,
        periods: &periods,
    };
    let text = if chrome.configured() {
        nova_bar::opencode_go_usage_widget_body_text(options)?
    } else {
        nova_bar::opencode_go_usage_widget_text(options)?
    };
    render_widget_command_segment(text, &chrome)
}

fn run_cpu_usage(args: &[String]) -> Result<String, String> {
    run_system_usage_widget(args, "cpu", nova_bar::current_cpu_usage_widget_text())
}

fn run_ram_usage(args: &[String]) -> Result<String, String> {
    run_system_usage_widget(args, "ram", nova_bar::current_ram_usage_widget_text())
}

fn run_system_usage_widget(args: &[String], label: &str, value: String) -> Result<String, String> {
    let mut chrome = WidgetCommandChrome::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if parse_widget_chrome_arg(arg, &mut iter, &mut chrome)? {
            continue;
        }
        return Err(format!("unknown {label} argument: {arg}"));
    }
    let body = if chrome.configured() {
        format!("{label} {value}")
    } else {
        value
    };
    render_widget_command_segment(body, &chrome)
}

fn parse_agent_usage_display(raw: &str) -> Result<nova_bar::AgentUsageDisplay, String> {
    match raw {
        "both" => Ok(nova_bar::AgentUsageDisplay::Both),
        "token" | "tokens" => Ok(nova_bar::AgentUsageDisplay::Token),
        "quota" => Ok(nova_bar::AgentUsageDisplay::Quota),
        _ => Err(format!("invalid display mode: {raw}")),
    }
}

fn parse_agent_usage_periods(raw: &str) -> Result<Vec<nova_bar::AgentUsagePeriod>, String> {
    let mut periods = Vec::new();
    for value in raw.split(',') {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let period = parse_agent_usage_period(value)?;
        if !periods.contains(&period) {
            periods.push(period);
        }
    }
    if periods.is_empty() {
        return Err("--periods must include at least one period".to_string());
    }
    Ok(periods)
}

fn parse_agent_usage_period(raw: &str) -> Result<nova_bar::AgentUsagePeriod, String> {
    match raw {
        "5h" | "five_hour" | "five-hour" => Ok(nova_bar::AgentUsagePeriod::FiveHour),
        "week" | "weekly" | "wk" => Ok(nova_bar::AgentUsagePeriod::Weekly),
        "month" | "monthly" | "mo" => Ok(nova_bar::AgentUsagePeriod::Monthly),
        _ => Err(format!("invalid usage period: {raw}")),
    }
}

fn parse_u64_arg(name: &str, value: Option<&String>) -> Result<u64, String> {
    value
        .ok_or_else(|| format!("{name} requires a value"))?
        .parse::<u64>()
        .map_err(|_| format!("{name} must be an integer"))
}

fn parse_bool_arg(name: &str, value: Option<&String>) -> Result<bool, String> {
    match value.map(String::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(value) => Err(format!("{name} must be true or false, got {value}")),
        None => Err(format!("{name} requires a value")),
    }
}

fn unix_time_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn default_cache_path(file_name: &str) -> Option<std::path::PathBuf> {
    xdg_base_path("XDG_CACHE_HOME", ".cache").map(|base| base.join("nova_bar").join(file_name))
}

fn xdg_base_path(env_name: &str, home_fallback: &str) -> Option<std::path::PathBuf> {
    env::var_os(env_name)
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| std::path::PathBuf::from(home).join(home_fallback))
        })
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: integrated Yazelix status-bar rendering is owned by this child command surface, not copied into the main repo.
    #[test]
    fn render_nova_runtime_outputs_layout_fragments() {
        let output = run(vec![
            "render-nova-runtime".into(),
            "--json".into(),
            serde_json::json!({
                "widget_tray": ["editor", "workspace", "cpu"],
                "zjstatus_plugin_url": "file:/runtime/configs/zellij/plugins/zjstatus.wasm",
                "editor_label": "hx",
                "shell_label": "nu",
                "terminal_label": "ghostty",
                "custom_text": "demo",
                "tab_label_mode": "compact",
                "nu_bin": "/runtime/libexec/nu",
                "yzx_control_bin": "/runtime/libexec/yzx_control",
                "nova_bar_widget_bin": "/runtime/libexec/nova_bar_widget",
                "runtime_dir": "/runtime",
                "claude_usage_display": "both",
                "claude_usage_periods": ["5h", "week"],
                "codex_usage_display": "quota",
                "codex_usage_periods": ["5h", "week"],
                "opencode_go_usage_display": "both",
                "opencode_go_usage_periods": ["5h", "week", "month"]
            })
            .to_string(),
        ])
        .unwrap();
        let rendered: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(rendered["schema_version"], 4);
        let plugin_block = rendered["plugin_block"].as_str().unwrap();
        assert!(
            plugin_block.contains(
                r#"plugin location="file:/runtime/configs/zellij/plugins/zjstatus.wasm" {"#
            )
        );
        assert!(plugin_block.contains(r#"role "view""#));
        assert!(!plugin_block.contains("format_"));
        let background = rendered["background_plugin_block"].as_str().unwrap();
        assert!(
            background.starts_with(r#""file:/runtime/configs/zellij/plugins/zjstatus.wasm" {"#)
        );
        assert!(background.contains(r#"role "controller""#));
        assert!(background.contains(
            "format_right \"#[fg=#00ff88,bold] hx{segment}{pipe_workspace}{segment}#[fg=#ff6600]{command_cpu}{segment}#[fg=#ffff00,bold][demo]{segment}#[fg=#00ccff,bold]{command_version}\""
        ));
        assert!(background.contains(r##"tab_normal "#[fg=#ffff00] [{index}] ""##));
        assert!(background.contains(
            r##"tab_normal_bell "#[fg=#ff0088,bold] [{index}] {sync_indicator}{fullscreen_indicator}""##
        ));
        assert!(background.contains(r##"tab_bell_indicator       """##));
        assert!(!background.contains("tab_activity"));
        assert!(background.contains(r##"pipe_workspace_format "#[fg=#00ff88,bold]{output}""##));
        assert!(background.contains(r#"format_left   "{tabs}""#));
        assert!(!background.contains("{mode}"));
        assert!(!background.contains("command_yazelix_tabs_command"));
        assert!(!background.contains("command_workspace_command"));
        assert!(background.contains(
            "/runtime/libexec/nova_bar_widget codex --display quota --periods 5h,week --widget-frame none --widget-separator empty --widget-first false"
        ));
        assert!(background.contains(
            "/runtime/libexec/nova_bar_widget cpu --widget-frame none --widget-separator empty --widget-first false"
        ));
    }

    // Regression: any widget chrome flag activates configured rendering instead of being a silent no-op.
    #[test]
    fn system_usage_command_honors_partial_widget_chrome_flags() {
        let output = run(vec![
            "cpu".into(),
            "--widget-separator".into(),
            "pipe".into(),
        ])
        .unwrap();

        assert!(output.starts_with(" | cpu "));
    }

    // Defends: command widgets render their own frame and first-position spacing for the integrated bar.
    #[test]
    fn system_usage_command_renders_configured_widget_segment() {
        let output = run(vec![
            "ram".into(),
            "--widget-frame".into(),
            "square".into(),
            "--widget-separator".into(),
            "dot".into(),
            "--widget-first".into(),
            "true".into(),
        ])
        .unwrap();

        assert!(output.starts_with(" [ram "));
        assert!(output.ends_with(']'));
    }

    // Defends: invalid generated widget chrome is visible to zjstatus instead of producing malformed bar text.
    #[test]
    fn system_usage_command_rejects_invalid_widget_chrome() {
        let error = run(vec![
            "cpu".into(),
            "--widget-frame".into(),
            "none".into(),
            "--widget-separator".into(),
            "comma".into(),
        ])
        .unwrap_err();

        assert!(error.contains("invalid_widget_separator"));
    }

    // Regression: the terminal widget follows the running Zellij session env instead of a
    // terminal label baked into a shared generated layout.
    #[test]
    fn term_command_renders_session_terminal_env() {
        let output = render_term_from_env(|key| {
            (key == SESSION_TERMINAL_ENV).then_some(" Mars ".to_string())
        });

        assert_eq!(output, " mars");
    }

    // Defends: missing or blank terminal ids render visibly instead of leaking a label from
    // another Yazelix window's generated config.
    #[test]
    fn term_command_renders_unknown_when_session_terminal_env_is_empty() {
        let blank =
            render_term_from_env(|key| (key == SESSION_TERMINAL_ENV).then_some("  ".to_string()));
        let missing = render_term_from_env(|_| None);

        assert_eq!(blank, " unknown");
        assert_eq!(missing, " unknown");
    }

    // Defends: the CLI keeps `term` as a no-argument command for zjstatus command-widget wiring.
    #[test]
    fn term_command_rejects_extra_args() {
        let error = run_term(&["extra".to_string()]).unwrap_err();

        assert_eq!(error, "term usage: nova_bar_widget term");
    }

    // Defends: one packaged version/channel identity produces the complete, compact Nova badge.
    #[test]
    fn version_command_renders_packaged_channel_badges() {
        for (version, channel, expected) in [
            ("dev", "stable", "NOVA DEV STABLE"),
            ("1.0.0-beta.4", "stable", "NOVA β4 STABLE"),
            ("1.0.0-beta.12", "main", "NOVA β12 MAIN"),
            ("1.0.0", "edge", "NOVA 1.0 EDGE"),
            ("1.2.0", "edge", "NOVA 1.2 EDGE"),
        ] {
            assert_eq!(
                run_version_fixture(&format!(
                    r#"{{"version":"{version}","channel":"{channel}"}}"#
                ))
                .unwrap(),
                expected
            );
        }
    }

    // Regression: malformed identities must fail visibly instead of rendering an ambiguous badge.
    #[test]
    fn version_command_rejects_invalid_runtime_identity() {
        for (identity, expected) in [
            (r#"{"channel":"stable"}"#, "missing string field `version`"),
            (r#"{"version":"1.0.0"}"#, "missing string field `channel`"),
            (
                r#"{"version":"1.0.0","channel":"preview"}"#,
                "unsupported Nova channel: preview",
            ),
            (
                r#"{"version":"v-test","channel":"stable"}"#,
                "unsupported Nova version: v-test",
            ),
        ] {
            assert!(
                run_version_fixture(identity)
                    .unwrap_err()
                    .contains(expected)
            );
        }
    }

    fn run_version_fixture(identity: &str) -> Result<String, String> {
        let runtime_dir = unique_test_dir("nova-bar-version");
        std::fs::create_dir_all(&runtime_dir).unwrap();
        std::fs::write(runtime_dir.join("runtime_identity.json"), identity).unwrap();
        let result = run(vec![
            "version".into(),
            "--runtime-dir".into(),
            runtime_dir.to_string_lossy().into_owned(),
        ]);
        let _ = std::fs::remove_dir_all(runtime_dir);
        result
    }

    fn unique_test_dir(stem: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{}-{nanos}", std::process::id()))
    }
}
