mod appearance;

pub use appearance::{
    APPEARANCE_MODE_AUTO, APPEARANCE_MODE_DARK, APPEARANCE_MODE_LIGHT, default_appearance_mode,
    normalize_appearance_arg,
};
use appearance::{
    BarStyle, DARK_BAR_STYLE, LIGHT_BAR_STYLE, bar_style_for_appearance, runtime_bar_appearance,
};
use serde::{Deserialize, Serialize};

pub const WIDGET_EDITOR: &str = "editor";
pub const WIDGET_SESSION: &str = "session";
pub const WIDGET_SHELL: &str = "shell";
pub const WIDGET_TERM: &str = "term";
pub const WIDGET_WORKSPACE: &str = "workspace";
pub const WIDGET_CLAUDE_USAGE: &str = "claude_usage";
pub const WIDGET_CODEX_USAGE: &str = "codex_usage";
pub const WIDGET_OPENCODE_GO_USAGE: &str = "opencode_go_usage";
pub const WIDGET_CPU: &str = "cpu";
pub const WIDGET_RAM: &str = "ram";

pub const PIPE_WORKSPACE: &str = "{pipe_workspace}";
pub const COMMAND_CLAUDE_USAGE: &str = "{command_claude_usage}";
pub const COMMAND_CODEX_USAGE: &str = "{command_codex_usage}";
pub const COMMAND_OPENCODE_GO_USAGE: &str = "{command_opencode_go_usage}";
pub const COMMAND_CPU: &str = "{command_cpu}";
pub const COMMAND_RAM: &str = "{command_ram}";
pub const COMMAND_TERM: &str = "{command_term}";
pub const COMMAND_VERSION: &str = "{command_version}";
pub const TAB_LABEL_MODE_FULL: &str = "full";
pub const TAB_LABEL_MODE_COMPACT: &str = "compact";
pub const WIDGET_FRAME_NONE: &str = "none";
pub const WIDGET_FRAME_SQUARE: &str = "square";
pub const WIDGET_FRAME_ROUND: &str = "round";
pub const WIDGET_SEPARATOR_DOT: &str = "dot";
pub const WIDGET_SEPARATOR_EMPTY: &str = "empty";
pub const WIDGET_SEPARATOR_PIPE: &str = "pipe";
pub const WIDGET_SEPARATOR_SPACE: &str = "space";

pub const DEFAULT_WIDGET_TRAY: &[&str] = &[
    WIDGET_SESSION,
    WIDGET_EDITOR,
    WIDGET_SHELL,
    WIDGET_TERM,
    WIDGET_CODEX_USAGE,
    WIDGET_CPU,
    WIDGET_RAM,
];

pub fn default_widget_frame() -> String {
    WIDGET_FRAME_NONE.to_string()
}

pub fn default_widget_separator() -> String {
    WIDGET_SEPARATOR_DOT.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WidgetFrame {
    None,
    Square,
    Round,
}

impl WidgetFrame {
    fn parse(raw: &str) -> Result<Self, BarRenderError> {
        match raw.trim() {
            WIDGET_FRAME_NONE => Ok(Self::None),
            WIDGET_FRAME_SQUARE => Ok(Self::Square),
            WIDGET_FRAME_ROUND => Ok(Self::Round),
            value => Err(BarRenderError::InvalidWidgetFrame {
                value: value.to_string(),
            }),
        }
    }

    fn frame(self, text: &str) -> String {
        match self {
            Self::None => text.to_string(),
            Self::Square => format!("[{text}]"),
            Self::Round => format!("({text})"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::None => WIDGET_FRAME_NONE,
            Self::Square => WIDGET_FRAME_SQUARE,
            Self::Round => WIDGET_FRAME_ROUND,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WidgetSeparator {
    Dot,
    Empty,
    Pipe,
    Space,
}

impl WidgetSeparator {
    fn parse(raw: &str) -> Result<Self, BarRenderError> {
        match raw.trim() {
            WIDGET_SEPARATOR_DOT => Ok(Self::Dot),
            WIDGET_SEPARATOR_EMPTY => Ok(Self::Empty),
            WIDGET_SEPARATOR_PIPE => Ok(Self::Pipe),
            WIDGET_SEPARATOR_SPACE => Ok(Self::Space),
            value => Err(BarRenderError::InvalidWidgetSeparator {
                value: value.to_string(),
            }),
        }
    }

    fn marker(self) -> &'static str {
        match self {
            Self::Dot => "•",
            Self::Empty => "",
            Self::Pipe => "|",
            Self::Space => " ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WidgetChrome {
    frame: WidgetFrame,
    separator: WidgetSeparator,
}

impl WidgetChrome {
    fn parse(frame: &str, separator: &str) -> Result<Self, BarRenderError> {
        Ok(Self {
            frame: WidgetFrame::parse(frame)?,
            separator: WidgetSeparator::parse(separator)?,
        })
    }

    fn default_runtime() -> Self {
        Self {
            frame: WidgetFrame::None,
            separator: WidgetSeparator::Dot,
        }
    }

    fn plain_segment_prefix(self, first: bool) -> String {
        if first {
            " ".to_string()
        } else {
            match self.separator {
                WidgetSeparator::Dot => " • ".to_string(),
                WidgetSeparator::Pipe => " | ".to_string(),
                WidgetSeparator::Space => " ".to_string(),
                WidgetSeparator::Empty => String::new(),
            }
        }
    }

    fn styled_segment_prefix(self, first: bool, style: &BarStyle) -> String {
        if first {
            " ".to_string()
        } else {
            match self.separator {
                WidgetSeparator::Dot | WidgetSeparator::Pipe => {
                    format!(" {}{} ", style.separator, self.separator.marker())
                }
                WidgetSeparator::Space => " ".to_string(),
                WidgetSeparator::Empty => String::new(),
            }
        }
    }

    fn segment(self, text: &str, first: bool) -> String {
        if text.is_empty() {
            String::new()
        } else {
            format!(
                "{}{}",
                self.plain_segment_prefix(first),
                self.frame.frame(text)
            )
        }
    }
}

pub fn render_configured_widget_segment(
    text: &str,
    frame: &str,
    separator: &str,
    first: bool,
) -> Result<String, BarRenderError> {
    Ok(WidgetChrome::parse(frame, separator)?.segment(text, first))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRenderRequest {
    pub widget_tray: Vec<String>,
    pub editor_label: String,
    pub shell_label: String,
    pub terminal_label: String,
    pub custom_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRenderData {
    pub widget_tray_segment: String,
    pub custom_text_segment: String,
}

pub const NOVA_RUNTIME_BAR_RENDER_SCHEMA_VERSION: u64 = 4;
const NOVA_RUNTIME_BAR_TEMPLATE: &str = include_str!("../presets/nova_runtime_bar.template.kdl");
const RUNTIME_PLACEHOLDER_PREFIX: &str = "__YAZELIX_RUNTIME_";
const RUNTIME_ZJSTATUS_PLUGIN_URL_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_ZJSTATUS_PLUGIN_URL__";
const RUNTIME_ACTIVE_THEME_FIELDS_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_ACTIVE_THEME_FIELDS__";
const RUNTIME_APPEARANCE_MODE_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_APPEARANCE_MODE__";
const RUNTIME_HOST_THEME_PALETTES_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_HOST_THEME_PALETTES__";
const RUNTIME_FLOATING_INDICATOR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_FLOATING_INDICATOR__";
const RUNTIME_NU_BIN_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_NU_BIN__";
const RUNTIME_YZX_CONTROL_BIN_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_YZX_CONTROL_BIN__";
const RUNTIME_WIDGET_BIN_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_WIDGET_BIN__";
const RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";
const RUNTIME_CLAUDE_USAGE_DISPLAY_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_CLAUDE_USAGE_DISPLAY__";
const RUNTIME_CLAUDE_USAGE_PERIODS_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_CLAUDE_USAGE_PERIODS__";
const RUNTIME_CLAUDE_USAGE_WIDGET_ARGS_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_CLAUDE_USAGE_WIDGET_ARGS__";
const RUNTIME_CODEX_USAGE_DISPLAY_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_CODEX_USAGE_DISPLAY__";
const RUNTIME_CODEX_USAGE_PERIODS_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_CODEX_USAGE_PERIODS__";
const RUNTIME_CODEX_USAGE_WIDGET_ARGS_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_CODEX_USAGE_WIDGET_ARGS__";
const RUNTIME_OPENCODE_GO_USAGE_DISPLAY_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_OPENCODE_GO_USAGE_DISPLAY__";
const RUNTIME_OPENCODE_GO_USAGE_PERIODS_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_OPENCODE_GO_USAGE_PERIODS__";
const RUNTIME_OPENCODE_GO_USAGE_WIDGET_ARGS_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_OPENCODE_GO_USAGE_WIDGET_ARGS__";
const RUNTIME_CPU_WIDGET_ARGS_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_CPU_WIDGET_ARGS__";
const RUNTIME_RAM_WIDGET_ARGS_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_RAM_WIDGET_ARGS__";
const RUNTIME_COMMAND_CLAUDE_USAGE_FORMAT_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_COMMAND_CLAUDE_USAGE_FORMAT__";
const RUNTIME_COMMAND_CLAUDE_USAGE_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_COMMAND_CLAUDE_USAGE_PLACEHOLDER__";
const RUNTIME_COMMAND_CODEX_USAGE_FORMAT_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_COMMAND_CODEX_USAGE_FORMAT__";
const RUNTIME_COMMAND_CODEX_USAGE_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_COMMAND_CODEX_USAGE_PLACEHOLDER__";
const RUNTIME_COMMAND_OPENCODE_GO_USAGE_FORMAT_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_COMMAND_OPENCODE_GO_USAGE_FORMAT__";
const RUNTIME_COMMAND_OPENCODE_GO_USAGE_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_COMMAND_OPENCODE_GO_USAGE_PLACEHOLDER__";
const RUNTIME_COMMAND_CPU_FORMAT_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_COMMAND_CPU_FORMAT__";
const RUNTIME_COMMAND_CPU_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_COMMAND_CPU_PLACEHOLDER__";
const RUNTIME_COMMAND_RAM_FORMAT_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_COMMAND_RAM_FORMAT__";
const RUNTIME_COMMAND_RAM_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_COMMAND_RAM_PLACEHOLDER__";
const SYSTEM_USAGE_CACHE_SCHEMA_VERSION: u64 = 1;
const SYSTEM_USAGE_CACHE_MAX_AGE_MILLIS: u64 = 5_000;
const SYSTEM_USAGE_CACHE_REFRESH_GRACE_MILLIS: u64 = 30_000;
const SYSTEM_USAGE_CACHE_LOCK_WAIT_PADDING_MILLIS: u64 = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovaRuntimeBarConfig {
    pub zjstatus_plugin_url: String,
    pub widget_tray: Vec<String>,
    #[serde(default = "default_widget_frame")]
    pub widget_frame: String,
    #[serde(default = "default_widget_separator")]
    pub widget_separator: String,
    pub editor_label: String,
    pub shell_label: String,
    pub terminal_label: String,
    pub custom_text: String,
    #[serde(default = "default_appearance_mode")]
    pub appearance_mode: String,
    pub tab_label_mode: String,
    pub nu_bin: String,
    pub yzx_control_bin: String,
    pub nova_bar_widget_bin: String,
    pub runtime_dir: String,
    pub claude_usage_display: String,
    #[serde(default = "default_claude_usage_periods")]
    pub claude_usage_periods: Vec<String>,
    pub codex_usage_display: String,
    #[serde(default = "default_codex_usage_periods")]
    pub codex_usage_periods: Vec<String>,
    pub opencode_go_usage_display: String,
    #[serde(default = "default_opencode_go_usage_periods")]
    pub opencode_go_usage_periods: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NovaRuntimeBarRender {
    pub schema_version: u64,
    pub plugin_block: String,
    pub background_plugin_block: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabLabelFormats {
    pub tab_normal: String,
    pub tab_normal_fullscreen: String,
    pub tab_normal_sync: String,
    pub tab_normal_bell: String,
    pub tab_normal_flashing_bell: String,
    pub tab_active: String,
    pub tab_active_fullscreen: String,
    pub tab_active_sync: String,
    pub tab_rename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarRenderError {
    InvalidWidgetFrame { value: String },
    InvalidWidgetSeparator { value: String },
    InvalidWidgetTrayEntry { entry: String },
    InvalidTabLabelMode { mode: String },
    UnresolvedRuntimePresetPlaceholder { placeholder: String },
}

impl BarRenderError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidWidgetFrame { .. } => "invalid_widget_frame",
            Self::InvalidWidgetSeparator { .. } => "invalid_widget_separator",
            Self::InvalidWidgetTrayEntry { .. } => "invalid_widget_tray_entry",
            Self::InvalidTabLabelMode { .. } => "invalid_tab_label_mode",
            Self::UnresolvedRuntimePresetPlaceholder { .. } => {
                "unresolved_runtime_preset_placeholder"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentUsageDisplay {
    Both,
    Token,
    Quota,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentUsagePeriod {
    FiveHour,
    Weekly,
    Monthly,
}

impl AgentUsagePeriod {
    pub fn short_label(self) -> &'static str {
        match self {
            Self::FiveHour => "5h",
            Self::Weekly => "wk",
            Self::Monthly => "mo",
        }
    }
}

pub fn default_claude_usage_periods() -> Vec<String> {
    vec!["5h".to_string(), "week".to_string()]
}

pub fn default_codex_usage_periods() -> Vec<String> {
    vec!["5h".to_string(), "week".to_string()]
}

pub fn default_opencode_go_usage_periods() -> Vec<String> {
    vec!["5h".to_string(), "week".to_string(), "month".to_string()]
}

pub fn agent_usage_periods_arg(periods: &[String]) -> String {
    periods.join(",")
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowedAgentUsageFacts {
    pub updated_at_unix_seconds: Option<u64>,
    pub five_hour_tokens: Option<u64>,
    pub weekly_tokens: Option<u64>,
    pub monthly_tokens: Option<u64>,
    pub five_hour_remaining_percent: Option<u64>,
    pub weekly_remaining_percent: Option<u64>,
    pub monthly_remaining_percent: Option<u64>,
    pub five_hour_reset_at_unix_seconds: Option<u64>,
    pub weekly_reset_at_unix_seconds: Option<u64>,
    pub five_hour_window_seconds: Option<u64>,
    pub weekly_window_seconds: Option<u64>,
    pub error: Option<String>,
}

impl WindowedAgentUsageFacts {
    fn has_tokens(&self) -> bool {
        self.five_hour_tokens.is_some()
            || self.weekly_tokens.is_some()
            || self.monthly_tokens.is_some()
    }

    fn has_quota(&self) -> bool {
        self.five_hour_remaining_percent.is_some()
            || self.weekly_remaining_percent.is_some()
            || self.monthly_remaining_percent.is_some()
    }

    fn is_empty(&self) -> bool {
        !self.has_tokens() && !self.has_quota()
    }
}

pub const CODEX_USAGE_CACHE_SCHEMA_VERSION: i64 = 4;
pub const CLAUDE_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
pub const OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
pub const CODEX_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub const CLAUDE_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub const OPENCODE_GO_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub const OPENCODE_GO_PROVIDER_ID: &str = "opencode-go";
pub const CODEX_FIVE_HOUR_WINDOW_SECONDS: u64 = 5 * 60 * 60;
pub const CODEX_WEEKLY_WINDOW_SECONDS: u64 = 7 * 24 * 60 * 60;
pub const OPENCODE_GO_FIVE_HOUR_SECONDS: u64 = 5 * 60 * 60;
pub const OPENCODE_GO_WEEK_SECONDS: u64 = 7 * 24 * 60 * 60;
pub const OPENCODE_GO_MONTH_SECONDS: u64 = 30 * 24 * 60 * 60;
pub const OPENCODE_GO_FIVE_HOUR_LIMIT_USD: f64 = 12.0;
pub const OPENCODE_GO_WEEKLY_LIMIT_USD: f64 = 30.0;
pub const OPENCODE_GO_MONTHLY_LIMIT_USD: f64 = 60.0;
const AGENT_USAGE_MINUTE_SECONDS: u64 = 60;

#[derive(Debug, Clone, Copy)]
pub struct CodexUsageWidgetOptions<'a> {
    pub cache_path: &'a std::path::Path,
    pub path_var: Option<&'a std::ffi::OsStr>,
    pub now_unix_seconds: u64,
    pub max_age_seconds: u64,
    pub error_backoff_seconds: u64,
    pub timeout: std::time::Duration,
    pub display: AgentUsageDisplay,
    pub periods: &'a [AgentUsagePeriod],
}

#[derive(Debug, Clone, Copy)]
pub struct ClaudeUsageWidgetOptions<'a> {
    pub cache_path: &'a std::path::Path,
    pub path_var: Option<&'a std::ffi::OsStr>,
    pub now_unix_seconds: u64,
    pub max_age_seconds: u64,
    pub error_backoff_seconds: u64,
    pub timeout: std::time::Duration,
    pub display: AgentUsageDisplay,
    pub periods: &'a [AgentUsagePeriod],
}

#[derive(Debug, Clone, Copy)]
pub struct OpenCodeGoUsageWidgetOptions<'a> {
    pub cache_path: &'a std::path::Path,
    pub db_paths: &'a [std::path::PathBuf],
    pub now_unix_seconds: u64,
    pub max_age_seconds: u64,
    pub error_backoff_seconds: u64,
    pub display: AgentUsageDisplay,
    pub periods: &'a [AgentUsagePeriod],
}

pub fn codex_usage_widget_text(options: CodexUsageWidgetOptions<'_>) -> Result<String, String> {
    refresh_codex_usage_shared_cache(
        options.cache_path,
        options.path_var,
        options.now_unix_seconds,
        options.max_age_seconds,
        options.error_backoff_seconds,
        options.timeout,
        options.display,
    )?;
    Ok(render_codex_usage_widget_from_cache(
        options.cache_path,
        options.display,
        options.periods,
    ))
}

pub fn codex_usage_widget_body_text(
    options: CodexUsageWidgetOptions<'_>,
) -> Result<String, String> {
    refresh_codex_usage_shared_cache(
        options.cache_path,
        options.path_var,
        options.now_unix_seconds,
        options.max_age_seconds,
        options.error_backoff_seconds,
        options.timeout,
        options.display,
    )?;
    Ok(render_codex_usage_widget_body_from_cache(
        options.cache_path,
        options.display,
        options.periods,
    ))
}

pub fn claude_usage_widget_text(options: ClaudeUsageWidgetOptions<'_>) -> Result<String, String> {
    refresh_claude_usage_shared_cache(
        options.cache_path,
        options.path_var,
        options.now_unix_seconds,
        options.max_age_seconds,
        options.error_backoff_seconds,
        options.timeout,
    )?;
    Ok(render_claude_usage_widget_from_cache(
        options.cache_path,
        options.display,
        options.periods,
    ))
}

pub fn claude_usage_widget_body_text(
    options: ClaudeUsageWidgetOptions<'_>,
) -> Result<String, String> {
    refresh_claude_usage_shared_cache(
        options.cache_path,
        options.path_var,
        options.now_unix_seconds,
        options.max_age_seconds,
        options.error_backoff_seconds,
        options.timeout,
    )?;
    Ok(render_claude_usage_widget_body_from_cache(
        options.cache_path,
        options.display,
        options.periods,
    ))
}

#[cfg(feature = "providers")]
pub fn opencode_go_usage_widget_text(
    options: OpenCodeGoUsageWidgetOptions<'_>,
) -> Result<String, String> {
    refresh_opencode_go_usage_shared_cache(
        options.cache_path,
        options.db_paths,
        options.now_unix_seconds,
        options.max_age_seconds,
        options.error_backoff_seconds,
    )?;
    Ok(render_opencode_go_usage_widget_from_cache(
        options.cache_path,
        options.display,
        options.periods,
    ))
}

#[cfg(feature = "providers")]
pub fn opencode_go_usage_widget_body_text(
    options: OpenCodeGoUsageWidgetOptions<'_>,
) -> Result<String, String> {
    refresh_opencode_go_usage_shared_cache(
        options.cache_path,
        options.db_paths,
        options.now_unix_seconds,
        options.max_age_seconds,
        options.error_backoff_seconds,
    )?;
    Ok(render_opencode_go_usage_widget_body_from_cache(
        options.cache_path,
        options.display,
        options.periods,
    ))
}

pub fn render_opencode_go_usage_widget_from_cache(
    path: &std::path::Path,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    read_opencode_go_usage_shared_cache_value(path)
        .and_then(|cache| cache.get("opencode_go").cloned())
        .map(|entry| {
            render_windowed_agent_usage_status_widget(
                "go",
                &windowed_usage_facts_from_cache_entry(&entry),
                periods,
                display,
            )
        })
        .unwrap_or_default()
}

pub fn render_opencode_go_usage_widget_body_from_cache(
    path: &std::path::Path,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    read_opencode_go_usage_shared_cache_value(path)
        .and_then(|cache| cache.get("opencode_go").cloned())
        .map(|entry| {
            render_windowed_agent_usage_status_body(
                "go",
                &windowed_usage_facts_from_cache_entry(&entry),
                periods,
                display,
            )
        })
        .unwrap_or_default()
}

pub fn render_claude_usage_widget_from_cache(
    path: &std::path::Path,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    read_claude_usage_shared_cache_value(path)
        .and_then(|cache| cache.get("claude").cloned())
        .map(|entry| {
            render_windowed_agent_usage_status_widget(
                "claude",
                &windowed_usage_facts_from_cache_entry(&entry),
                periods,
                display,
            )
        })
        .unwrap_or_default()
}

pub fn render_claude_usage_widget_body_from_cache(
    path: &std::path::Path,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    read_claude_usage_shared_cache_value(path)
        .and_then(|cache| cache.get("claude").cloned())
        .map(|entry| {
            render_windowed_agent_usage_status_body(
                "claude",
                &windowed_usage_facts_from_cache_entry(&entry),
                periods,
                display,
            )
        })
        .unwrap_or_default()
}

pub fn render_codex_usage_widget_from_cache(
    path: &std::path::Path,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    read_codex_usage_shared_cache_value(path)
        .and_then(|cache| cache.get("codex").cloned())
        .map(|entry| {
            render_agent_usage_status_widget(
                codex_cache_status_label(&entry),
                &render_codex_usage_summary_for_periods(
                    &windowed_usage_facts_from_cache_entry(&entry),
                    display,
                    periods,
                ),
            )
        })
        .unwrap_or_default()
}

pub fn render_codex_usage_widget_body_from_cache(
    path: &std::path::Path,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    read_codex_usage_shared_cache_value(path)
        .and_then(|cache| cache.get("codex").cloned())
        .map(|entry| {
            render_agent_usage_status_body(
                codex_cache_status_label(&entry),
                &render_codex_usage_summary_for_periods(
                    &windowed_usage_facts_from_cache_entry(&entry),
                    display,
                    periods,
                ),
            )
        })
        .unwrap_or_default()
}

fn codex_cache_status_label(entry: &serde_json::Value) -> &'static str {
    match entry.get("status").and_then(serde_json::Value::as_str) {
        Some("ok") => "codex",
        _ => "codex~",
    }
}

pub fn read_codex_usage_shared_cache_value(path: &std::path::Path) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&raw).ok()?;
    if cache
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        != Some(CODEX_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub fn read_claude_usage_shared_cache_value(path: &std::path::Path) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&raw).ok()?;
    if cache
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        != Some(CLAUDE_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub fn read_opencode_go_usage_shared_cache_value(
    path: &std::path::Path,
) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&raw).ok()?;
    if cache
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        != Some(OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub fn claude_usage_cache_path_from_env() -> Option<std::path::PathBuf> {
    status_bar_cache_path_from_env().and_then(|path| {
        agent_usage_cache_path_from_status_cache_path(
            &path,
            "claude_usage_cache",
            CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
        )
    })
}

pub fn codex_usage_cache_path_from_env() -> Option<std::path::PathBuf> {
    status_bar_cache_path_from_env().and_then(|path| {
        agent_usage_cache_path_from_status_cache_path(
            &path,
            "codex_usage_cache",
            CODEX_USAGE_CACHE_SCHEMA_VERSION,
        )
    })
}

pub fn opencode_go_usage_cache_path_from_env() -> Option<std::path::PathBuf> {
    status_bar_cache_path_from_env().and_then(|path| {
        agent_usage_cache_path_from_status_cache_path(
            &path,
            "opencode_go_usage_cache",
            OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
        )
    })
}

pub fn agent_usage_cache_path_from_status_cache_path(
    status_cache_path: &std::path::Path,
    file_stem: &str,
    schema_version: i64,
) -> Option<std::path::PathBuf> {
    let state_dir = status_cache_path.parent()?.parent()?.parent()?;
    Some(
        state_dir
            .join("agent_usage")
            .join(format!("{file_stem}_v{schema_version}.json")),
    )
}

fn status_bar_cache_path_from_env() -> Option<std::path::PathBuf> {
    std::env::var_os("YAZELIX_STATUS_BAR_CACHE_PATH")
        .map(std::path::PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

pub fn refresh_codex_usage_shared_cache(
    shared_path: &std::path::Path,
    path_var: Option<&std::ffi::OsStr>,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
    timeout: std::time::Duration,
    display: AgentUsageDisplay,
) -> Result<bool, String> {
    if codex_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds, display)
        || codex_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }
    let Some(lock) = try_acquire_codex_usage_cache_lock(shared_path, now)? else {
        return Ok(false);
    };
    if codex_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds, display)
        || codex_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }

    let previous_facts = read_codex_usage_shared_cache_value(shared_path)
        .and_then(|cache| cache.get("codex").cloned())
        .map(|entry| windowed_usage_facts_from_cache_entry(&entry));
    let mut facts = collect_codex_usage_facts(path_var, timeout, display, &lock.path);
    let refresh_incomplete = !codex_usage_facts_are_complete(&facts, display);
    if !facts.has_quota() {
        preserve_previous_codex_window_quota(&mut facts, previous_facts.as_ref(), now);
    }
    preserve_previous_codex_window_tokens(&mut facts, previous_facts.as_ref());

    let mut entry = serde_json::Map::new();
    entry.insert(
        "updated_at_unix_seconds".to_string(),
        serde_json::json!(now),
    );
    insert_u64(&mut entry, "five_hour_tokens", facts.five_hour_tokens);
    insert_u64(&mut entry, "weekly_tokens", facts.weekly_tokens);
    insert_u64(
        &mut entry,
        "five_hour_remaining_percent",
        facts.five_hour_remaining_percent,
    );
    insert_u64(
        &mut entry,
        "weekly_remaining_percent",
        facts.weekly_remaining_percent,
    );
    insert_u64(
        &mut entry,
        "five_hour_reset_at_unix_seconds",
        facts.five_hour_reset_at_unix_seconds,
    );
    insert_u64(
        &mut entry,
        "weekly_reset_at_unix_seconds",
        facts.weekly_reset_at_unix_seconds,
    );
    insert_u64(
        &mut entry,
        "five_hour_window_seconds",
        facts.five_hour_window_seconds,
    );
    insert_u64(
        &mut entry,
        "weekly_window_seconds",
        facts.weekly_window_seconds,
    );
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        entry.insert("error".to_string(), serde_json::json!(error));
    }
    if refresh_incomplete {
        entry.insert(
            "backoff_until_unix_seconds".to_string(),
            serde_json::json!(now.saturating_add(error_backoff_seconds)),
        );
    }
    let status = if facts.is_empty() {
        "error"
    } else if refresh_incomplete {
        "partial"
    } else {
        "ok"
    };
    entry.insert("status".to_string(), serde_json::json!(status));

    write_json_value_atomic(
        shared_path,
        &serde_json::json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": serde_json::Value::Object(entry),
        }),
    )?;
    Ok(true)
}

pub fn refresh_claude_usage_shared_cache(
    shared_path: &std::path::Path,
    path_var: Option<&std::ffi::OsStr>,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
    timeout: std::time::Duration,
) -> Result<bool, String> {
    if claude_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds)
        || claude_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_claude_usage_cache_lock(shared_path, now)? else {
        return Ok(false);
    };
    if claude_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds)
        || claude_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }

    let quota_backoff_until = claude_usage_quota_backoff_until(shared_path, now);
    let previous_facts = read_claude_usage_shared_cache_value(shared_path)
        .and_then(|cache| cache.get("claude").cloned())
        .map(|entry| windowed_usage_facts_from_cache_entry(&entry));
    let mut facts = collect_claude_usage_facts(path_var, timeout, quota_backoff_until.is_none());
    let quota_probe_failed = quota_backoff_until.is_none() && !facts.has_quota();
    preserve_previous_claude_window_quota(&mut facts, previous_facts.as_ref());

    let mut entry = serde_json::Map::new();
    entry.insert(
        "updated_at_unix_seconds".to_string(),
        serde_json::json!(now),
    );
    insert_u64(&mut entry, "five_hour_tokens", facts.five_hour_tokens);
    insert_u64(&mut entry, "weekly_tokens", facts.weekly_tokens);
    insert_u64(
        &mut entry,
        "five_hour_remaining_percent",
        facts.five_hour_remaining_percent,
    );
    insert_u64(
        &mut entry,
        "weekly_remaining_percent",
        facts.weekly_remaining_percent,
    );
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        entry.insert("error".to_string(), serde_json::json!(error));
        if facts.is_empty() {
            entry.insert(
                "backoff_until_unix_seconds".to_string(),
                serde_json::json!(now.saturating_add(error_backoff_seconds)),
            );
        }
    }
    if let Some(backoff_until) = quota_backoff_until {
        entry.insert(
            "quota_backoff_until_unix_seconds".to_string(),
            serde_json::json!(backoff_until),
        );
    } else if facts.has_tokens() && (quota_probe_failed || !facts.has_quota()) {
        entry.insert(
            "quota_backoff_until_unix_seconds".to_string(),
            serde_json::json!(now.saturating_add(error_backoff_seconds)),
        );
    }
    let status = if facts.is_empty() {
        "error"
    } else if facts.has_tokens()
        && facts.has_quota()
        && !quota_probe_failed
        && quota_backoff_until.is_none()
    {
        "ok"
    } else {
        "partial"
    };
    entry.insert("status".to_string(), serde_json::json!(status));

    write_json_value_atomic(
        shared_path,
        &serde_json::json!({
            "schema_version": CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
            "claude": serde_json::Value::Object(entry),
        }),
    )?;
    Ok(true)
}

#[cfg(feature = "providers")]
pub fn refresh_opencode_go_usage_shared_cache(
    shared_path: &std::path::Path,
    db_paths: &[std::path::PathBuf],
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
) -> Result<bool, String> {
    if opencode_go_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds)
        || opencode_go_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_opencode_go_usage_cache_lock(shared_path, now)? else {
        return Ok(false);
    };
    if opencode_go_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds)
        || opencode_go_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }

    let facts = collect_opencode_go_usage_facts_from_dbs(db_paths, now);
    let mut entry = serde_json::Map::new();
    entry.insert(
        "updated_at_unix_seconds".to_string(),
        serde_json::json!(now),
    );
    insert_u64(&mut entry, "five_hour_tokens", facts.five_hour_tokens);
    insert_u64(&mut entry, "weekly_tokens", facts.weekly_tokens);
    insert_u64(&mut entry, "monthly_tokens", facts.monthly_tokens);
    insert_u64(
        &mut entry,
        "five_hour_remaining_percent",
        facts.five_hour_remaining_percent,
    );
    insert_u64(
        &mut entry,
        "weekly_remaining_percent",
        facts.weekly_remaining_percent,
    );
    insert_u64(
        &mut entry,
        "monthly_remaining_percent",
        facts.monthly_remaining_percent,
    );
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        entry.insert("error".to_string(), serde_json::json!(error));
        entry.insert(
            "backoff_until_unix_seconds".to_string(),
            serde_json::json!(now.saturating_add(error_backoff_seconds)),
        );
    }
    let status = if facts.is_empty() {
        "error"
    } else if facts.has_tokens() && facts.has_quota() {
        "ok"
    } else {
        "partial"
    };
    entry.insert("status".to_string(), serde_json::json!(status));

    write_json_value_atomic(
        shared_path,
        &serde_json::json!({
            "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
            "opencode_go": serde_json::Value::Object(entry),
        }),
    )?;
    Ok(true)
}

fn escape_kdl_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn runtime_theme_fields(
    config: &NovaRuntimeBarConfig,
    style: &BarStyle,
    chrome: WidgetChrome,
) -> Result<Vec<(&'static str, String)>, BarRenderError> {
    let request = BarRenderRequest {
        widget_tray: config.widget_tray.clone(),
        editor_label: config.editor_label.clone(),
        shell_label: config.shell_label.clone(),
        terminal_label: config.terminal_label.clone(),
        custom_text: config.custom_text.clone(),
    };
    let segment_chrome = WidgetChrome {
        separator: WidgetSeparator::Empty,
        ..chrome
    };
    let mut segments = request
        .widget_tray
        .iter()
        .map(|widget| render_widget_with_style(widget, &request, style, segment_chrome, false))
        .collect::<Result<Vec<_>, _>>()?;
    if !config.custom_text.trim().is_empty() {
        segments.push(render_runtime_custom_text_segment_with_style(
            &config.custom_text,
            style,
            segment_chrome,
            false,
        ));
    }
    segments.push(format!("{}{}", style.brand, COMMAND_VERSION));
    let tabs = render_zjstatus_tab_label_formats_with_style(&config.tab_label_mode, style)?;
    Ok(vec![
        ("format_right", segments.join("{segment}")),
        (
            "format_right_separator",
            chrome.styled_segment_prefix(false, style),
        ),
        ("tab_normal", kdl_assignment_value(&tabs.tab_normal)),
        (
            "tab_normal_fullscreen",
            kdl_assignment_value(&tabs.tab_normal_fullscreen),
        ),
        (
            "tab_normal_sync",
            kdl_assignment_value(&tabs.tab_normal_sync),
        ),
        (
            "tab_normal_bell",
            kdl_assignment_value(&tabs.tab_normal_bell),
        ),
        (
            "tab_normal_flashing_bell",
            kdl_assignment_value(&tabs.tab_normal_flashing_bell),
        ),
        ("tab_active", kdl_assignment_value(&tabs.tab_active)),
        (
            "tab_active_fullscreen",
            kdl_assignment_value(&tabs.tab_active_fullscreen),
        ),
        (
            "tab_active_sync",
            kdl_assignment_value(&tabs.tab_active_sync),
        ),
        ("tab_rename", kdl_assignment_value(&tabs.tab_rename)),
        (
            "tab_truncate_start_format",
            format!("{}< +{{count}} ... ", style.tab_truncate),
        ),
        (
            "tab_truncate_end_format",
            format!("{}... +{{count}} > ", style.tab_truncate),
        ),
        ("datetime", format!("{} {{format}} ", style.datetime)),
        (
            "pipe_workspace_format",
            format!("{}{}", style.workspace, chrome.frame.frame("{output}")),
        ),
    ])
}

fn kdl_assignment_value(assignment: &str) -> String {
    let start = assignment
        .find('"')
        .expect("generated KDL assignment has an opening quote")
        + 1;
    assignment[start..assignment.len() - 1].to_string()
}

fn render_runtime_theme_fields(prefix: &str, fields: &[(&'static str, String)]) -> String {
    fields
        .iter()
        .map(|(key, value)| {
            let datetime_marker = if prefix.is_empty() && *key == "format_right" {
                " // {datetime}"
            } else {
                ""
            };
            format!(
                "    {prefix}{key} \"{}\"{datetime_marker}",
                escape_kdl_string(value)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_zjstatus_bar_segments(
    request: &BarRenderRequest,
) -> Result<BarRenderData, BarRenderError> {
    render_zjstatus_bar_segments_with_style(
        request,
        &DARK_BAR_STYLE,
        WidgetChrome::default_runtime(),
    )
}

fn render_zjstatus_bar_segments_with_style(
    request: &BarRenderRequest,
    style: &BarStyle,
    chrome: WidgetChrome,
) -> Result<BarRenderData, BarRenderError> {
    let has_tray = !request.widget_tray.is_empty();
    Ok(BarRenderData {
        widget_tray_segment: render_widget_tray_segment_with_style(request, style, chrome)?,
        custom_text_segment: render_runtime_custom_text_segment_with_style(
            &request.custom_text,
            style,
            chrome,
            !has_tray,
        ),
    })
}

pub fn render_nova_runtime_plugin_block(
    config: &NovaRuntimeBarConfig,
) -> Result<String, BarRenderError> {
    Ok(format!(
        "plugin location=\"{}\" {{\n    role \"view\"\n}}",
        escape_kdl_string(&config.zjstatus_plugin_url)
    ))
}

pub fn render_nova_runtime_background_plugin_block(
    config: &NovaRuntimeBarConfig,
) -> Result<String, BarRenderError> {
    render_nova_runtime_block(config)
}

fn render_nova_runtime_block(config: &NovaRuntimeBarConfig) -> Result<String, BarRenderError> {
    let appearance_mode = runtime_bar_appearance(&config.appearance_mode);
    let chrome = WidgetChrome::parse(&config.widget_frame, &config.widget_separator)?;
    let dark_theme = runtime_theme_fields(config, &DARK_BAR_STYLE, chrome)?;
    let light_theme = runtime_theme_fields(config, &LIGHT_BAR_STYLE, chrome)?;
    let active_theme = if appearance_mode == APPEARANCE_MODE_LIGHT {
        &light_theme
    } else {
        &dark_theme
    };
    let widget_args = || runtime_command_widget_args(chrome);
    let widget_placeholder = |label| escape_kdl_string(&chrome.frame.frame(label));
    let host_theme_palettes = [
        render_runtime_theme_fields("host_theme_dark_", &dark_theme),
        render_runtime_theme_fields("host_theme_light_", &light_theme),
    ]
    .join("\n");
    let replacements = [
        (
            RUNTIME_ZJSTATUS_PLUGIN_URL_PLACEHOLDER,
            escape_kdl_string(&config.zjstatus_plugin_url),
        ),
        (
            RUNTIME_ACTIVE_THEME_FIELDS_PLACEHOLDER,
            render_runtime_theme_fields("", active_theme),
        ),
        (
            RUNTIME_APPEARANCE_MODE_PLACEHOLDER,
            escape_kdl_string(appearance_mode),
        ),
        (RUNTIME_HOST_THEME_PALETTES_PLACEHOLDER, host_theme_palettes),
        (
            RUNTIME_FLOATING_INDICATOR_PLACEHOLDER,
            escape_kdl_string("\u{2b1a} "),
        ),
        (
            RUNTIME_NU_BIN_PLACEHOLDER,
            escape_kdl_string(&config.nu_bin),
        ),
        (
            RUNTIME_YZX_CONTROL_BIN_PLACEHOLDER,
            escape_kdl_string(&config.yzx_control_bin),
        ),
        (
            RUNTIME_WIDGET_BIN_PLACEHOLDER,
            escape_kdl_string(&config.nova_bar_widget_bin),
        ),
        (
            RUNTIME_DIR_PLACEHOLDER,
            escape_kdl_string(&config.runtime_dir),
        ),
        (
            RUNTIME_CLAUDE_USAGE_DISPLAY_PLACEHOLDER,
            escape_kdl_string(&config.claude_usage_display),
        ),
        (
            RUNTIME_CLAUDE_USAGE_PERIODS_PLACEHOLDER,
            escape_kdl_string(&agent_usage_periods_arg(&config.claude_usage_periods)),
        ),
        (RUNTIME_CLAUDE_USAGE_WIDGET_ARGS_PLACEHOLDER, widget_args()),
        (
            RUNTIME_CODEX_USAGE_DISPLAY_PLACEHOLDER,
            escape_kdl_string(&config.codex_usage_display),
        ),
        (
            RUNTIME_CODEX_USAGE_PERIODS_PLACEHOLDER,
            escape_kdl_string(&agent_usage_periods_arg(&config.codex_usage_periods)),
        ),
        (RUNTIME_CODEX_USAGE_WIDGET_ARGS_PLACEHOLDER, widget_args()),
        (
            RUNTIME_OPENCODE_GO_USAGE_DISPLAY_PLACEHOLDER,
            escape_kdl_string(&config.opencode_go_usage_display),
        ),
        (
            RUNTIME_OPENCODE_GO_USAGE_PERIODS_PLACEHOLDER,
            escape_kdl_string(&agent_usage_periods_arg(&config.opencode_go_usage_periods)),
        ),
        (
            RUNTIME_OPENCODE_GO_USAGE_WIDGET_ARGS_PLACEHOLDER,
            widget_args(),
        ),
        (RUNTIME_CPU_WIDGET_ARGS_PLACEHOLDER, widget_args()),
        (RUNTIME_RAM_WIDGET_ARGS_PLACEHOLDER, widget_args()),
        (
            RUNTIME_COMMAND_CLAUDE_USAGE_FORMAT_PLACEHOLDER,
            "{stdout}".to_string(),
        ),
        (
            RUNTIME_COMMAND_CLAUDE_USAGE_PLACEHOLDER,
            widget_placeholder("claude …"),
        ),
        (
            RUNTIME_COMMAND_CODEX_USAGE_FORMAT_PLACEHOLDER,
            "{stdout}".to_string(),
        ),
        (
            RUNTIME_COMMAND_CODEX_USAGE_PLACEHOLDER,
            widget_placeholder("codex …"),
        ),
        (
            RUNTIME_COMMAND_OPENCODE_GO_USAGE_FORMAT_PLACEHOLDER,
            "{stdout}".to_string(),
        ),
        (
            RUNTIME_COMMAND_OPENCODE_GO_USAGE_PLACEHOLDER,
            widget_placeholder("opencode …"),
        ),
        (
            RUNTIME_COMMAND_CPU_FORMAT_PLACEHOLDER,
            "{stdout}".to_string(),
        ),
        (RUNTIME_COMMAND_CPU_PLACEHOLDER, widget_placeholder("cpu …")),
        (
            RUNTIME_COMMAND_RAM_FORMAT_PLACEHOLDER,
            "{stdout}".to_string(),
        ),
        (RUNTIME_COMMAND_RAM_PLACEHOLDER, widget_placeholder("ram …")),
    ];
    let mut rendered = NOVA_RUNTIME_BAR_TEMPLATE.to_string();
    for (placeholder, value) in replacements {
        rendered = rendered.replace(placeholder, &value);
    }
    if let Some(placeholder) = unresolved_runtime_preset_placeholder(&rendered) {
        return Err(BarRenderError::UnresolvedRuntimePresetPlaceholder { placeholder });
    }
    Ok(rendered)
}

fn unresolved_runtime_preset_placeholder(rendered: &str) -> Option<String> {
    let start = rendered.find(RUNTIME_PLACEHOLDER_PREFIX)?;
    let suffix = &rendered[start + RUNTIME_PLACEHOLDER_PREFIX.len()..];
    let end = suffix.find("__")?;
    Some(rendered[start..start + RUNTIME_PLACEHOLDER_PREFIX.len() + end + 2].to_string())
}

fn runtime_command_widget_args(chrome: WidgetChrome) -> String {
    format!(
        " --widget-frame {} --widget-separator {} --widget-first {}",
        chrome.frame.as_str(),
        WIDGET_SEPARATOR_EMPTY,
        "false"
    )
}

pub fn render_widget_tray_segment(request: &BarRenderRequest) -> Result<String, BarRenderError> {
    render_widget_tray_segment_with_style(request, &DARK_BAR_STYLE, WidgetChrome::default_runtime())
}

fn render_widget_tray_segment_with_style(
    request: &BarRenderRequest,
    style: &BarStyle,
    chrome: WidgetChrome,
) -> Result<String, BarRenderError> {
    let mut rendered = String::new();
    for (index, widget) in request.widget_tray.iter().enumerate() {
        let part = render_widget_with_style(widget, request, style, chrome, index == 0)?;
        rendered.push_str(&part);
    }
    Ok(rendered)
}

pub fn render_custom_text_segment(custom_text: &str) -> String {
    render_custom_text_segment_with_style(custom_text, &DARK_BAR_STYLE)
}

fn render_custom_text_segment_with_style(custom_text: &str, style: &BarStyle) -> String {
    let trimmed = custom_text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{}[{trimmed}] ", style.custom_text)
    }
}

fn render_runtime_custom_text_segment_with_style(
    custom_text: &str,
    style: &BarStyle,
    chrome: WidgetChrome,
    first: bool,
) -> String {
    let trimmed = custom_text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!(
            "{}{}[{trimmed}]",
            chrome.styled_segment_prefix(first, style),
            style.custom_text
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SystemUsageWidget {
    Cpu,
    Ram,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SystemUsageCache {
    schema_version: u64,
    captured_unix_millis: u64,
    cpu_percent: Option<u64>,
    ram_percent: Option<u64>,
}

pub fn current_cpu_usage_widget_text() -> String {
    current_system_usage_widget_text(SystemUsageWidget::Cpu, current_unix_millis())
}

pub fn current_ram_usage_widget_text() -> String {
    current_system_usage_widget_text(SystemUsageWidget::Ram, current_unix_millis())
}

fn current_system_usage_widget_text(widget: SystemUsageWidget, now_unix_millis: u64) -> String {
    let Some(cache_path) = system_usage_cache_path() else {
        return system_usage_widget_text(&sample_system_usage_cache(now_unix_millis), widget);
    };
    refresh_system_usage_cache_if_needed(&cache_path, now_unix_millis)
        .as_ref()
        .map(|cache| system_usage_widget_text(cache, widget))
        .unwrap_or_else(|| "??%".to_string())
}

fn current_unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn refresh_system_usage_cache_if_needed(
    cache_path: &std::path::Path,
    now_unix_millis: u64,
) -> Option<SystemUsageCache> {
    if let Some(cache) = read_fresh_system_usage_cache(cache_path, now_unix_millis) {
        return Some(cache);
    }

    let Some(_lock) = acquire_system_usage_cache_lock(cache_path) else {
        if let Some(cache) = read_recent_system_usage_cache(cache_path, now_unix_millis) {
            return Some(cache);
        }
        std::thread::sleep(
            sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
                + std::time::Duration::from_millis(SYSTEM_USAGE_CACHE_LOCK_WAIT_PADDING_MILLIS),
        );
        return read_recent_system_usage_cache(cache_path, now_unix_millis);
    };
    if let Some(cache) = read_fresh_system_usage_cache(cache_path, now_unix_millis) {
        return Some(cache);
    }

    let cache = sample_system_usage_cache(now_unix_millis);
    let _ = write_system_usage_cache(cache_path, &cache);
    Some(cache)
}

fn acquire_system_usage_cache_lock(cache_path: &std::path::Path) -> Option<std::fs::File> {
    let lock_path = system_usage_cache_lock_path(cache_path);
    std::fs::create_dir_all(lock_path.parent()?).ok()?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .ok()?;
    file.try_lock().ok()?;
    Some(file)
}

fn system_usage_cache_lock_path(cache_path: &std::path::Path) -> std::path::PathBuf {
    let file_name = cache_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("system_usage_cache.json");
    cache_path.with_file_name(format!("{file_name}.lock"))
}

fn read_fresh_system_usage_cache(
    path: &std::path::Path,
    now_unix_millis: u64,
) -> Option<SystemUsageCache> {
    read_system_usage_cache_with_max_age(path, now_unix_millis, SYSTEM_USAGE_CACHE_MAX_AGE_MILLIS)
}

fn read_recent_system_usage_cache(
    path: &std::path::Path,
    now_unix_millis: u64,
) -> Option<SystemUsageCache> {
    read_system_usage_cache_with_max_age(
        path,
        now_unix_millis,
        SYSTEM_USAGE_CACHE_MAX_AGE_MILLIS + SYSTEM_USAGE_CACHE_REFRESH_GRACE_MILLIS,
    )
}

fn read_system_usage_cache_with_max_age(
    path: &std::path::Path,
    now_unix_millis: u64,
    max_age_millis: u64,
) -> Option<SystemUsageCache> {
    let cache = read_system_usage_cache(path)?;
    let age = now_unix_millis.checked_sub(cache.captured_unix_millis)?;
    (age <= max_age_millis).then_some(cache)
}

fn read_system_usage_cache(path: &std::path::Path) -> Option<SystemUsageCache> {
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: SystemUsageCache = serde_json::from_str(&raw).ok()?;
    (cache.schema_version == SYSTEM_USAGE_CACHE_SCHEMA_VERSION).then_some(cache)
}

fn write_system_usage_cache(
    path: &std::path::Path,
    cache: &SystemUsageCache,
) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("json.tmp");
    let encoded = serde_json::to_vec(cache).map_err(std::io::Error::other)?;
    std::fs::write(&tmp_path, encoded)?;
    std::fs::rename(tmp_path, path)
}

fn sample_system_usage_cache(now_unix_millis: u64) -> SystemUsageCache {
    let mut system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_cpu(sysinfo::CpuRefreshKind::everything())
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_cpu_all();
    system.refresh_memory_specifics(sysinfo::MemoryRefreshKind::nothing().with_ram());
    let cpu_percent = rounded_bounded_percent(system.global_cpu_usage() as f64);
    let total_memory = system.total_memory();
    let available_memory = system.available_memory();
    let ram_percent = if total_memory == 0 || available_memory > total_memory {
        None
    } else {
        Some(round_percent(total_memory - available_memory, total_memory))
    };

    SystemUsageCache {
        schema_version: SYSTEM_USAGE_CACHE_SCHEMA_VERSION,
        captured_unix_millis: now_unix_millis,
        cpu_percent,
        ram_percent,
    }
}

fn rounded_bounded_percent(percent: f64) -> Option<u64> {
    percent
        .is_finite()
        .then(|| percent.round().clamp(0.0, 100.0) as u64)
}

fn system_usage_widget_text(cache: &SystemUsageCache, widget: SystemUsageWidget) -> String {
    match widget {
        SystemUsageWidget::Cpu => format_percent(cache.cpu_percent),
        SystemUsageWidget::Ram => format_percent(cache.ram_percent),
    }
}

fn system_usage_cache_path() -> Option<std::path::PathBuf> {
    status_bar_cache_path_from_env()
        .and_then(|path| {
            let state_dir = path.parent()?.parent()?.parent()?;
            Some(state_dir.join("system_usage").join(format!(
                "system_usage_cache_v{SYSTEM_USAGE_CACHE_SCHEMA_VERSION}.json"
            )))
        })
        .or_else(default_system_usage_cache_path)
}

fn default_system_usage_cache_path() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .filter(|path| !path.as_os_str().is_empty())
                .map(|home| home.join(".cache"))
        })
        .map(|cache_dir| {
            cache_dir.join("nova_bar").join(format!(
                "system_usage_cache_v{SYSTEM_USAGE_CACHE_SCHEMA_VERSION}.json"
            ))
        })
}

pub fn cpu_usage_percent_from_proc_stat(before: &str, after: &str) -> Option<u64> {
    cpu_usage_percent_from_totals(
        parse_proc_stat_cpu_totals(before)?,
        parse_proc_stat_cpu_totals(after)?,
    )
}

pub fn ram_usage_percent_from_meminfo(raw: &str) -> Option<u64> {
    let total = meminfo_kib_value(raw, "MemTotal:")?;
    let available = meminfo_kib_value(raw, "MemAvailable:")?;
    if total == 0 || available > total {
        return None;
    }
    Some(round_percent(total - available, total))
}

fn parse_proc_stat_cpu_totals(raw: &str) -> Option<(u64, u64)> {
    let line = raw.lines().find(|line| line.starts_with("cpu "))?;
    let values = line
        .split_whitespace()
        .skip(1)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() < 7 {
        return None;
    }
    let idle = values[3].checked_add(*values.get(4).unwrap_or(&0))?;
    let total = values
        .iter()
        .try_fold(0u64, |sum, value| sum.checked_add(*value))?;
    Some((idle, total))
}

fn cpu_usage_percent_from_totals(before: (u64, u64), after: (u64, u64)) -> Option<u64> {
    let idle_delta = after.0.checked_sub(before.0)?;
    let total_delta = after.1.checked_sub(before.1)?;
    if total_delta == 0 || idle_delta > total_delta {
        return None;
    }
    Some(round_percent(total_delta - idle_delta, total_delta))
}

fn meminfo_kib_value(raw: &str, key: &str) -> Option<u64> {
    raw.lines()
        .find_map(|line| line.strip_prefix(key))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
}

fn round_percent(numerator: u64, denominator: u64) -> u64 {
    ((numerator.saturating_mul(100) + denominator / 2) / denominator).min(100)
}

fn format_percent(percent: Option<u64>) -> String {
    percent
        .map(|percent| format!("{percent}%"))
        .unwrap_or_else(|| "??%".to_string())
}

fn insert_u64(
    entry: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<u64>,
) {
    if let Some(value) = value {
        entry.insert(key.to_string(), serde_json::json!(value));
    }
}

fn windowed_usage_facts_from_cache_entry(entry: &serde_json::Value) -> WindowedAgentUsageFacts {
    WindowedAgentUsageFacts {
        updated_at_unix_seconds: entry
            .get("updated_at_unix_seconds")
            .and_then(serde_json::Value::as_u64),
        five_hour_tokens: entry
            .get("five_hour_tokens")
            .and_then(serde_json::Value::as_u64),
        weekly_tokens: entry
            .get("weekly_tokens")
            .and_then(serde_json::Value::as_u64),
        monthly_tokens: entry
            .get("monthly_tokens")
            .and_then(serde_json::Value::as_u64),
        five_hour_remaining_percent: entry
            .get("five_hour_remaining_percent")
            .and_then(serde_json::Value::as_u64),
        weekly_remaining_percent: entry
            .get("weekly_remaining_percent")
            .and_then(serde_json::Value::as_u64),
        monthly_remaining_percent: entry
            .get("monthly_remaining_percent")
            .and_then(serde_json::Value::as_u64),
        five_hour_reset_at_unix_seconds: entry
            .get("five_hour_reset_at_unix_seconds")
            .and_then(serde_json::Value::as_u64),
        weekly_reset_at_unix_seconds: entry
            .get("weekly_reset_at_unix_seconds")
            .and_then(serde_json::Value::as_u64),
        five_hour_window_seconds: entry
            .get("five_hour_window_seconds")
            .and_then(serde_json::Value::as_u64),
        weekly_window_seconds: entry
            .get("weekly_window_seconds")
            .and_then(serde_json::Value::as_u64),
        error: entry
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

pub fn codex_usage_shared_cache_is_fresh(
    path: &std::path::Path,
    now: u64,
    max_age_seconds: u64,
    display: AgentUsageDisplay,
) -> bool {
    let Some(cache) = read_codex_usage_shared_cache_value(path) else {
        return false;
    };
    cache
        .get("codex")
        .and_then(|entry| entry.get("updated_at_unix_seconds"))
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get("codex")
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| codex_usage_facts_are_complete(&facts, display))
        })
}

pub fn codex_usage_shared_cache_is_backing_off(path: &std::path::Path, now: u64) -> bool {
    read_codex_usage_shared_cache_value(path)
        .and_then(|cache| {
            cache
                .get("codex")?
                .get("backoff_until_unix_seconds")?
                .as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

pub fn claude_usage_shared_cache_is_fresh(
    path: &std::path::Path,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    let Some(cache) = read_claude_usage_shared_cache_value(path) else {
        return false;
    };
    cache
        .get("claude")
        .and_then(|entry| entry.get("updated_at_unix_seconds"))
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get("claude")
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| claude_usage_facts_are_complete(&facts))
        })
}

pub fn claude_usage_shared_cache_is_backing_off(path: &std::path::Path, now: u64) -> bool {
    read_claude_usage_shared_cache_value(path)
        .and_then(|cache| {
            let entry = cache.get("claude")?;
            let facts = windowed_usage_facts_from_cache_entry(entry);
            if !facts.is_empty() && !claude_usage_facts_are_complete(&facts) {
                return None;
            }
            entry.get("backoff_until_unix_seconds")?.as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

pub fn opencode_go_usage_shared_cache_is_fresh(
    path: &std::path::Path,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    let Some(cache) = read_opencode_go_usage_shared_cache_value(path) else {
        return false;
    };
    cache
        .get("opencode_go")
        .and_then(|entry| entry.get("updated_at_unix_seconds"))
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get("opencode_go")
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| opencode_go_usage_facts_are_complete(&facts))
        })
}

pub fn opencode_go_usage_shared_cache_is_backing_off(path: &std::path::Path, now: u64) -> bool {
    read_opencode_go_usage_shared_cache_value(path)
        .and_then(|cache| {
            cache
                .get("opencode_go")?
                .get("backoff_until_unix_seconds")?
                .as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

pub fn claude_usage_quota_backoff_until(path: &std::path::Path, now: u64) -> Option<u64> {
    read_claude_usage_shared_cache_value(path)
        .and_then(|cache| {
            cache
                .get("claude")?
                .get("quota_backoff_until_unix_seconds")?
                .as_u64()
        })
        .filter(|backoff_until| now < *backoff_until)
}

fn codex_usage_facts_are_complete(
    facts: &WindowedAgentUsageFacts,
    display: AgentUsageDisplay,
) -> bool {
    let tokens_complete = facts.five_hour_tokens.is_some() && facts.weekly_tokens.is_some();
    let quota_complete = facts.has_quota()
        && codex_quota_window_is_complete(
            facts.five_hour_remaining_percent,
            facts.five_hour_reset_at_unix_seconds,
            facts.five_hour_window_seconds,
        )
        && codex_quota_window_is_complete(
            facts.weekly_remaining_percent,
            facts.weekly_reset_at_unix_seconds,
            facts.weekly_window_seconds,
        );
    match display {
        AgentUsageDisplay::Both => tokens_complete && quota_complete,
        AgentUsageDisplay::Token => tokens_complete,
        AgentUsageDisplay::Quota => quota_complete,
    }
}

fn codex_quota_window_is_complete(
    remaining_percent: Option<u64>,
    reset_at_unix_seconds: Option<u64>,
    window_seconds: Option<u64>,
) -> bool {
    matches!(
        (remaining_percent, reset_at_unix_seconds, window_seconds),
        (Some(_), Some(_), Some(_)) | (None, None, None)
    )
}

fn claude_usage_facts_are_complete(facts: &WindowedAgentUsageFacts) -> bool {
    facts.five_hour_tokens.is_some()
        && facts.weekly_tokens.is_some()
        && facts.five_hour_remaining_percent.is_some()
        && facts.weekly_remaining_percent.is_some()
}

fn opencode_go_usage_facts_are_complete(facts: &WindowedAgentUsageFacts) -> bool {
    facts.five_hour_tokens.is_some()
        && facts.weekly_tokens.is_some()
        && facts.monthly_tokens.is_some()
        && facts.five_hour_remaining_percent.is_some()
        && facts.weekly_remaining_percent.is_some()
        && facts.monthly_remaining_percent.is_some()
}

struct AgentUsageCacheLock {
    path: std::path::PathBuf,
}

impl Drop for AgentUsageCacheLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.path);
    }
}

fn try_acquire_codex_usage_cache_lock(
    shared_path: &std::path::Path,
    now: u64,
) -> Result<Option<AgentUsageCacheLock>, String> {
    try_acquire_agent_usage_cache_lock(
        shared_path.with_file_name(format!(
            ".codex_usage_cache_v{CODEX_USAGE_CACHE_SCHEMA_VERSION}.lock"
        )),
        now,
        CODEX_USAGE_LOCK_STALE_AFTER_SECONDS,
    )
}

fn try_acquire_claude_usage_cache_lock(
    shared_path: &std::path::Path,
    now: u64,
) -> Result<Option<AgentUsageCacheLock>, String> {
    try_acquire_agent_usage_cache_lock(
        shared_path.with_file_name(format!(
            ".claude_usage_cache_v{CLAUDE_USAGE_CACHE_SCHEMA_VERSION}.lock"
        )),
        now,
        CLAUDE_USAGE_LOCK_STALE_AFTER_SECONDS,
    )
}

fn try_acquire_opencode_go_usage_cache_lock(
    shared_path: &std::path::Path,
    now: u64,
) -> Result<Option<AgentUsageCacheLock>, String> {
    try_acquire_agent_usage_cache_lock(
        shared_path.with_file_name(format!(
            ".opencode_go_usage_cache_v{OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION}.lock"
        )),
        now,
        OPENCODE_GO_USAGE_LOCK_STALE_AFTER_SECONDS,
    )
}

fn try_acquire_agent_usage_cache_lock(
    lock_path: std::path::PathBuf,
    now: u64,
    stale_after_seconds: u64,
) -> Result<Option<AgentUsageCacheLock>, String> {
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create usage cache lock parent: {error}"))?;
    }
    match std::fs::create_dir(&lock_path) {
        Ok(()) => Ok(Some(AgentUsageCacheLock { path: lock_path })),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if agent_usage_cache_lock_is_stale(&lock_path, now, stale_after_seconds) {
                let _ = std::fs::remove_dir(&lock_path);
                return match std::fs::create_dir(&lock_path) {
                    Ok(()) => Ok(Some(AgentUsageCacheLock { path: lock_path })),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
                    Err(error) => Err(format!("failed to acquire usage cache lock: {error}")),
                };
            }
            Ok(None)
        }
        Err(error) => Err(format!("failed to acquire usage cache lock: {error}")),
    }
}

fn agent_usage_cache_lock_is_stale(
    lock_path: &std::path::Path,
    now: u64,
    stale_after_seconds: u64,
) -> bool {
    std::fs::metadata(lock_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| now.saturating_sub(duration.as_secs()) > stale_after_seconds)
        .unwrap_or(false)
}

fn preserve_previous_codex_window_tokens(
    facts: &mut WindowedAgentUsageFacts,
    previous: Option<&WindowedAgentUsageFacts>,
) {
    let Some(previous) = previous else {
        return;
    };
    if !codex_usage_facts_are_complete(previous, AgentUsageDisplay::Token) {
        return;
    }
    if facts.five_hour_tokens.is_none()
        && tokenusage_window_identity_matches(
            facts.five_hour_reset_at_unix_seconds,
            facts.five_hour_window_seconds,
            previous.five_hour_reset_at_unix_seconds,
            previous.five_hour_window_seconds,
        )
    {
        facts.five_hour_tokens = previous.five_hour_tokens;
    }
    if facts.weekly_tokens.is_none()
        && tokenusage_window_identity_matches(
            facts.weekly_reset_at_unix_seconds,
            facts.weekly_window_seconds,
            previous.weekly_reset_at_unix_seconds,
            previous.weekly_window_seconds,
        )
    {
        facts.weekly_tokens = previous.weekly_tokens;
    }
}

fn preserve_previous_codex_window_quota(
    facts: &mut WindowedAgentUsageFacts,
    previous: Option<&WindowedAgentUsageFacts>,
    now: u64,
) {
    let Some(previous) = previous else {
        return;
    };
    if facts.five_hour_remaining_percent.is_none()
        && previous_codex_quota_window_is_still_valid(
            previous.five_hour_reset_at_unix_seconds,
            previous.five_hour_window_seconds,
            now,
        )
    {
        facts.five_hour_remaining_percent = previous.five_hour_remaining_percent;
        facts.five_hour_reset_at_unix_seconds = previous.five_hour_reset_at_unix_seconds;
        facts.five_hour_window_seconds = previous.five_hour_window_seconds;
    }
    if facts.weekly_remaining_percent.is_none()
        && previous_codex_quota_window_is_still_valid(
            previous.weekly_reset_at_unix_seconds,
            previous.weekly_window_seconds,
            now,
        )
    {
        facts.weekly_remaining_percent = previous.weekly_remaining_percent;
        facts.weekly_reset_at_unix_seconds = previous.weekly_reset_at_unix_seconds;
        facts.weekly_window_seconds = previous.weekly_window_seconds;
    }
}

fn preserve_previous_claude_window_quota(
    facts: &mut WindowedAgentUsageFacts,
    previous: Option<&WindowedAgentUsageFacts>,
) {
    let Some(previous) = previous else {
        return;
    };
    if facts.five_hour_remaining_percent.is_none() {
        facts.five_hour_remaining_percent = previous.five_hour_remaining_percent;
    }
    if facts.weekly_remaining_percent.is_none() {
        facts.weekly_remaining_percent = previous.weekly_remaining_percent;
    }
}

fn previous_codex_quota_window_is_still_valid(
    reset_at_unix_seconds: Option<u64>,
    window_seconds: Option<u64>,
    now: u64,
) -> bool {
    reset_at_unix_seconds.is_some_and(|reset_at| now < reset_at)
        && window_seconds.is_some_and(|seconds| seconds > 0)
}

fn tokenusage_window_identity_matches(
    reset_at: Option<u64>,
    window_seconds: Option<u64>,
    previous_reset_at: Option<u64>,
    previous_window_seconds: Option<u64>,
) -> bool {
    reset_at.is_some()
        && window_seconds.is_some()
        && reset_at == previous_reset_at
        && window_seconds == previous_window_seconds
}

fn collect_codex_usage_facts(
    path_var: Option<&std::ffi::OsStr>,
    timeout: std::time::Duration,
    display: AgentUsageDisplay,
    empty_source: &std::path::Path,
) -> WindowedAgentUsageFacts {
    let started = std::time::Instant::now();
    let Some(path_var) = path_var else {
        return WindowedAgentUsageFacts {
            error: Some("missing PATH".to_string()),
            ..WindowedAgentUsageFacts::default()
        };
    };
    let Some(binary_path) = find_command_in_path_var(path_var, "tu") else {
        return WindowedAgentUsageFacts {
            error: Some("missing tu".to_string()),
            ..WindowedAgentUsageFacts::default()
        };
    };

    let mut facts = WindowedAgentUsageFacts::default();
    if matches!(display, AgentUsageDisplay::Token | AgentUsageDisplay::Both) {
        match run_tokenusage_json_command_with_budget(
            &binary_path,
            CODEX_ACTIVE_BLOCK_ARGS,
            started,
            timeout,
        ) {
            Ok(Some(value)) => {
                facts.five_hour_tokens = tokenusage_active_block_tokens_from_json(&value)
            }
            Ok(None) => facts.error = Some("active block unavailable".to_string()),
            Err(error) => facts.error = Some(error),
        }
        match run_tokenusage_json_command_with_budget(
            &binary_path,
            CODEX_WEEKLY_ARGS,
            started,
            timeout,
        ) {
            Ok(Some(value)) => facts.weekly_tokens = tokenusage_weekly_tokens_from_json(&value),
            Ok(None) => {
                if facts.error.is_none() {
                    facts.error = Some("weekly usage unavailable".to_string());
                }
            }
            Err(error) => {
                if facts.error.is_none() {
                    facts.error = Some(error);
                }
            }
        }
    }
    if matches!(display, AgentUsageDisplay::Quota | AgentUsageDisplay::Both) {
        match run_codex_official_limits_json_command_with_budget(
            &binary_path,
            empty_source,
            started,
            timeout,
        ) {
            Ok(Some(value)) => {
                let quota = codex_quota_from_official_json(&value);
                facts.five_hour_remaining_percent = quota.five_hour_remaining_percent;
                facts.weekly_remaining_percent = quota.weekly_remaining_percent;
                facts.five_hour_reset_at_unix_seconds = quota.five_hour_reset_at_unix_seconds;
                facts.weekly_reset_at_unix_seconds = quota.weekly_reset_at_unix_seconds;
                facts.five_hour_window_seconds = quota.five_hour_window_seconds;
                facts.weekly_window_seconds = quota.weekly_window_seconds;
                if !quota.has_quota() && facts.error.is_none() {
                    facts.error = Some("quota unavailable".to_string());
                }
            }
            Ok(None) => {
                if facts.error.is_none() {
                    facts.error = Some("quota unavailable".to_string());
                }
            }
            Err(error) => {
                if facts.error.is_none() {
                    facts.error = Some(error);
                }
            }
        }
    }
    facts
}

fn collect_claude_usage_facts(
    path_var: Option<&std::ffi::OsStr>,
    timeout: std::time::Duration,
    include_quota: bool,
) -> WindowedAgentUsageFacts {
    let Some(path_var) = path_var else {
        return WindowedAgentUsageFacts {
            error: Some("missing PATH".to_string()),
            ..WindowedAgentUsageFacts::default()
        };
    };
    let Some(binary_path) = find_command_in_path_var(path_var, "tu") else {
        return WindowedAgentUsageFacts {
            error: Some("missing tu".to_string()),
            ..WindowedAgentUsageFacts::default()
        };
    };

    let mut facts = WindowedAgentUsageFacts::default();
    match run_tokenusage_json_command(&binary_path, CLAUDE_ACTIVE_BLOCK_ARGS, timeout) {
        Ok(Some(value)) => {
            facts.five_hour_tokens = tokenusage_active_block_tokens_from_json(&value)
        }
        Ok(None) => facts.error = Some("active block unavailable".to_string()),
        Err(error) => facts.error = Some(error),
    }
    match run_tokenusage_json_command(&binary_path, CLAUDE_WEEKLY_ARGS, timeout) {
        Ok(Some(value)) => facts.weekly_tokens = tokenusage_weekly_tokens_from_json(&value),
        Ok(None) => {
            if facts.error.is_none() {
                facts.error = Some("weekly usage unavailable".to_string());
            }
        }
        Err(error) => {
            if facts.error.is_none() {
                facts.error = Some(error);
            }
        }
    }
    if include_quota {
        match run_tokenusage_json_command(&binary_path, CLAUDE_OFFICIAL_LIMITS_ARGS, timeout) {
            Ok(Some(value)) => {
                let quota = claude_quota_from_official_json(&value);
                facts.five_hour_remaining_percent = quota.five_hour_remaining_percent;
                facts.weekly_remaining_percent = quota.weekly_remaining_percent;
                if !quota.has_quota() && facts.error.is_none() {
                    facts.error = Some("quota unavailable".to_string());
                }
            }
            Ok(None) => {
                if facts.error.is_none() {
                    facts.error = Some("quota unavailable".to_string());
                }
            }
            Err(error) => {
                if facts.error.is_none() {
                    facts.error = Some(error);
                }
            }
        }
    }
    facts
}

#[cfg(feature = "providers")]
pub fn collect_opencode_go_usage_facts_from_dbs(
    db_paths: &[std::path::PathBuf],
    now: u64,
) -> WindowedAgentUsageFacts {
    if db_paths.is_empty() {
        return WindowedAgentUsageFacts {
            error: Some("missing OpenCode DB".to_string()),
            ..WindowedAgentUsageFacts::default()
        };
    }

    let mut five_hour = OpenCodeGoUsageWindow::default();
    let mut weekly = OpenCodeGoUsageWindow::default();
    let mut monthly = OpenCodeGoUsageWindow::default();
    let mut opened_any = false;
    let mut first_error = None;

    for path in db_paths {
        match collect_opencode_go_usage_windows_from_db(path, now) {
            Ok(db_windows) => {
                opened_any = true;
                five_hour.tokens = five_hour.tokens.saturating_add(db_windows.five_hour.tokens);
                five_hour.cost_usd += db_windows.five_hour.cost_usd;
                weekly.tokens = weekly.tokens.saturating_add(db_windows.weekly.tokens);
                weekly.cost_usd += db_windows.weekly.cost_usd;
                monthly.tokens = monthly.tokens.saturating_add(db_windows.monthly.tokens);
                monthly.cost_usd += db_windows.monthly.cost_usd;
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(format!("{}: {error}", path.display()));
                }
            }
        }
    }

    if !opened_any {
        return WindowedAgentUsageFacts {
            error: first_error.or_else(|| Some("OpenCode DB unavailable".to_string())),
            ..WindowedAgentUsageFacts::default()
        };
    }

    WindowedAgentUsageFacts {
        five_hour_tokens: Some(five_hour.tokens),
        five_hour_remaining_percent: Some(remaining_percent_from_cost_limit(
            five_hour.cost_usd,
            OPENCODE_GO_FIVE_HOUR_LIMIT_USD,
        )),
        weekly_tokens: Some(weekly.tokens),
        weekly_remaining_percent: Some(remaining_percent_from_cost_limit(
            weekly.cost_usd,
            OPENCODE_GO_WEEKLY_LIMIT_USD,
        )),
        monthly_tokens: Some(monthly.tokens),
        monthly_remaining_percent: Some(remaining_percent_from_cost_limit(
            monthly.cost_usd,
            OPENCODE_GO_MONTHLY_LIMIT_USD,
        )),
        ..WindowedAgentUsageFacts::default()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg(feature = "providers")]
pub struct OpenCodeGoUsageWindow {
    pub tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg(feature = "providers")]
pub struct OpenCodeGoUsageWindows {
    pub five_hour: OpenCodeGoUsageWindow,
    pub weekly: OpenCodeGoUsageWindow,
    pub monthly: OpenCodeGoUsageWindow,
}

#[cfg(feature = "providers")]
pub fn collect_opencode_go_usage_windows_from_db(
    path: &std::path::Path,
    now: u64,
) -> Result<OpenCodeGoUsageWindows, String> {
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| format!("failed to open OpenCode DB read-only: {error}"))?;
    connection
        .busy_timeout(std::time::Duration::from_millis(250))
        .map_err(|error| format!("failed to configure OpenCode DB read timeout: {error}"))?;
    let five_hour = query_opencode_go_usage_window(
        &connection,
        now.saturating_sub(OPENCODE_GO_FIVE_HOUR_SECONDS),
    )?;
    let weekly =
        query_opencode_go_usage_window(&connection, now.saturating_sub(OPENCODE_GO_WEEK_SECONDS))?;
    let monthly =
        query_opencode_go_usage_window(&connection, now.saturating_sub(OPENCODE_GO_MONTH_SECONDS))?;
    Ok(OpenCodeGoUsageWindows {
        five_hour,
        weekly,
        monthly,
    })
}

#[cfg(feature = "providers")]
pub fn query_opencode_go_usage_window(
    connection: &rusqlite::Connection,
    since_unix_seconds: u64,
) -> Result<OpenCodeGoUsageWindow, String> {
    connection
        .query_row(
            r#"
            SELECT
              COALESCE(SUM(
                COALESCE(
                  json_extract(data, '$.tokens.total'),
                  COALESCE(json_extract(data, '$.tokens.input'), 0) +
                  COALESCE(json_extract(data, '$.tokens.output'), 0) +
                  COALESCE(json_extract(data, '$.tokens.reasoning'), 0) +
                  COALESCE(json_extract(data, '$.tokens.cache.read'), 0) +
                  COALESCE(json_extract(data, '$.tokens.cache.write'), 0)
                )
              ), 0),
              COALESCE(SUM(COALESCE(json_extract(data, '$.cost'), 0.0)), 0.0)
            FROM message
            WHERE time_created >= ?1
              AND json_extract(data, '$.role') = 'assistant'
              AND json_extract(data, '$.providerID') = ?2
            "#,
            rusqlite::params![
                unix_millis_from_seconds_saturating(since_unix_seconds),
                OPENCODE_GO_PROVIDER_ID
            ],
            |row| {
                Ok(OpenCodeGoUsageWindow {
                    tokens: row.get::<_, i64>(0)?.max(0) as u64,
                    cost_usd: row.get::<_, f64>(1)?.max(0.0),
                })
            },
        )
        .map_err(|error| format!("failed to read OpenCode Go usage window: {error}"))
}

#[cfg(feature = "providers")]
pub fn unix_millis_from_seconds_saturating(seconds: u64) -> i64 {
    seconds.saturating_mul(1000).min(i64::MAX as u64) as i64
}

#[cfg(feature = "providers")]
pub fn remaining_percent_from_cost_limit(cost_usd: f64, limit_usd: f64) -> u64 {
    if limit_usd <= 0.0 {
        return 0;
    }
    (100.0 - (cost_usd / limit_usd * 100.0))
        .clamp(0.0, 100.0)
        .round() as u64
}

pub fn opencode_db_candidates_from_env() -> Vec<std::path::PathBuf> {
    opencode_db_candidates_from_values(
        std::env::var_os("OPENCODE_DB").map(std::path::PathBuf::from),
        std::env::var_os("OPENCODE_DATA_DIR").map(std::path::PathBuf::from),
        std::env::var_os("XDG_DATA_HOME").map(std::path::PathBuf::from),
        std::env::var_os("HOME").map(std::path::PathBuf::from),
    )
}

pub fn opencode_db_candidates_from_values(
    opencode_db: Option<std::path::PathBuf>,
    opencode_data_dir: Option<std::path::PathBuf>,
    xdg_data_home: Option<std::path::PathBuf>,
    home: Option<std::path::PathBuf>,
) -> Vec<std::path::PathBuf> {
    let data_dir = opencode_data_dir
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| {
            xdg_data_home
                .filter(|path| !path.as_os_str().is_empty())
                .map(|path| path.join("opencode"))
        })
        .or_else(|| {
            home.filter(|path| !path.as_os_str().is_empty())
                .map(|path| path.join(".local").join("share").join("opencode"))
        });

    let mut candidates = Vec::new();
    if let Some(path) = opencode_db.filter(|path| !path.as_os_str().is_empty()) {
        if path.is_absolute() {
            push_unique_path(&mut candidates, path);
        } else if let Some(data_dir) = data_dir.as_ref() {
            push_unique_path(&mut candidates, data_dir.join(path));
        } else {
            push_unique_path(&mut candidates, path);
        }
    }

    if let Some(data_dir) = data_dir {
        push_unique_path(&mut candidates, data_dir.join("opencode.db"));
        if let Ok(entries) = std::fs::read_dir(data_dir) {
            let mut channel_dbs = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("opencode-") && name.ends_with(".db"))
                })
                .collect::<Vec<_>>();
            channel_dbs.sort();
            for path in channel_dbs {
                push_unique_path(&mut candidates, path);
            }
        }
    }
    candidates
}

fn push_unique_path(paths: &mut Vec<std::path::PathBuf>, path: std::path::PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

const CODEX_ACTIVE_BLOCK_ARGS: &[&str] = &[
    "blocks",
    "--active",
    "--json",
    "--offline",
    "--workers",
    "1",
    "--no-claude",
    "--no-antigravity",
];

const CODEX_WEEKLY_ARGS: &[&str] = &[
    "weekly",
    "--json",
    "--offline",
    "--workers",
    "1",
    "--no-claude",
    "--no-antigravity",
    "--order",
    "desc",
];

const CODEX_OFFICIAL_LIMITS_ARGS: &[&str] = &[
    "blocks",
    "--active",
    "--json",
    "--official-limits",
    "--workers",
    "1",
    "--no-claude",
    "--no-antigravity",
];

const CLAUDE_ACTIVE_BLOCK_ARGS: &[&str] = &[
    "blocks",
    "--active",
    "--json",
    "--offline",
    "--no-codex",
    "--no-antigravity",
];

const CLAUDE_WEEKLY_ARGS: &[&str] = &[
    "weekly",
    "--json",
    "--offline",
    "--no-codex",
    "--no-antigravity",
    "--order",
    "desc",
];

const CLAUDE_OFFICIAL_LIMITS_ARGS: &[&str] = &[
    "blocks",
    "--active",
    "--json",
    "--official-limits",
    "--no-codex",
    "--no-antigravity",
];

fn run_tokenusage_json_command(
    binary_path: &std::path::Path,
    args: &[&str],
    timeout: std::time::Duration,
) -> Result<Option<serde_json::Value>, String> {
    let output =
        run_tokenusage_command_with_timeout(binary_path, args, timeout).map_err(|error| {
            format!(
                "failed to run tokenusage command {}: {error}",
                binary_path.display()
            )
        })?;
    parse_tokenusage_json_output(output)
}

fn run_tokenusage_json_command_with_budget(
    binary_path: &std::path::Path,
    args: &[&str],
    started: std::time::Instant,
    timeout: std::time::Duration,
) -> Result<Option<serde_json::Value>, String> {
    let remaining = timeout.saturating_sub(started.elapsed());
    if remaining.is_zero() {
        return Ok(None);
    }
    run_tokenusage_json_command(binary_path, args, remaining)
}

fn run_codex_official_limits_json_command_with_budget(
    binary_path: &std::path::Path,
    empty_source: &std::path::Path,
    started: std::time::Instant,
    timeout: std::time::Duration,
) -> Result<Option<serde_json::Value>, String> {
    let remaining = timeout.saturating_sub(started.elapsed());
    if remaining.is_zero() {
        return Ok(None);
    }
    let mut command = std::process::Command::new(binary_path);
    command
        .args(CODEX_OFFICIAL_LIMITS_ARGS)
        .arg("--codex-sessions-dir")
        .arg(empty_source)
        .arg("--no-incremental-cache")
        .env("RAYON_NUM_THREADS", "1")
        .env("TOKIO_WORKER_THREADS", "1");
    let output = run_command_with_timeout(command, remaining).map_err(|error| {
        format!(
            "failed to run tokenusage command {}: {error}",
            binary_path.display()
        )
    })?;
    parse_tokenusage_json_output(output)
}

fn run_tokenusage_command_with_timeout(
    binary_path: &std::path::Path,
    args: &[&str],
    timeout: std::time::Duration,
) -> std::io::Result<Option<std::process::Output>> {
    let mut command = std::process::Command::new(binary_path);
    command.args(args);
    if let Some(worker_count) = args
        .windows(2)
        .find(|pair| pair[0] == "--workers")
        .map(|pair| pair[1])
    {
        command
            .env("RAYON_NUM_THREADS", worker_count)
            .env("TOKIO_WORKER_THREADS", worker_count);
    }
    run_command_with_timeout(command, timeout)
}

fn run_command_with_timeout(
    mut command: std::process::Command,
    timeout: std::time::Duration,
) -> std::io::Result<Option<std::process::Output>> {
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    let started = std::time::Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map(Some);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn parse_tokenusage_json_output(
    output: Option<std::process::Output>,
) -> Result<Option<serde_json::Value>, String> {
    let Some(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(json_raw) = extract_json_object(&stdout) else {
        return Ok(None);
    };
    serde_json::from_str::<serde_json::Value>(json_raw)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn find_command_in_path_var(
    path_var: &std::ffi::OsStr,
    command_name: &str,
) -> Option<std::path::PathBuf> {
    std::env::split_paths(path_var)
        .map(|entry| entry.join(command_name))
        .find(|candidate| candidate.is_file())
}

fn tokenusage_active_block_tokens_from_json(value: &serde_json::Value) -> Option<u64> {
    value
        .get("blocks")
        .and_then(serde_json::Value::as_array)
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|block| {
                    block
                        .get("isActive")
                        .or_else(|| block.get("is_active"))
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
                })
                .or_else(|| blocks.first())
        })
        .and_then(|block| {
            first_u64_at(
                block,
                &[
                    &["totals", "total_tokens"],
                    &["totals", "totalTokens"],
                    &["total_tokens"],
                    &["totalTokens"],
                ],
            )
        })
}

fn tokenusage_weekly_tokens_from_json(value: &serde_json::Value) -> Option<u64> {
    value
        .get("weekly")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| {
            first_u64_at(
                row,
                &[
                    &["totals", "total_tokens"],
                    &["totals", "totalTokens"],
                    &["total_tokens"],
                    &["totalTokens"],
                ],
            )
        })
}

fn codex_quota_from_official_json(value: &serde_json::Value) -> WindowedAgentUsageFacts {
    let Some(official) = value.get("official_codex") else {
        return WindowedAgentUsageFacts::default();
    };

    let mut facts = WindowedAgentUsageFacts::default();
    for name in ["primary", "secondary"] {
        let Some(remaining_percent) = official
            .get(format!("{name}_used_percent"))
            .and_then(serde_json::Value::as_f64)
            .map(remaining_percent_from_used)
        else {
            continue;
        };
        let Some(window_seconds) = official
            .get(format!("{name}_window_mins"))
            .and_then(serde_json::Value::as_u64)
            .and_then(window_minutes_to_seconds)
        else {
            continue;
        };
        let reset_at = official
            .get(format!("{name}_resets_at"))
            .and_then(serde_json::Value::as_u64);

        match window_seconds {
            CODEX_FIVE_HOUR_WINDOW_SECONDS if facts.five_hour_remaining_percent.is_none() => {
                facts.five_hour_remaining_percent = Some(remaining_percent);
                facts.five_hour_reset_at_unix_seconds = reset_at;
                facts.five_hour_window_seconds = Some(window_seconds);
            }
            CODEX_WEEKLY_WINDOW_SECONDS if facts.weekly_remaining_percent.is_none() => {
                facts.weekly_remaining_percent = Some(remaining_percent);
                facts.weekly_reset_at_unix_seconds = reset_at;
                facts.weekly_window_seconds = Some(window_seconds);
            }
            _ => {}
        }
    }
    facts
}

fn claude_quota_from_official_json(value: &serde_json::Value) -> WindowedAgentUsageFacts {
    let Some(official) = value.get("official_claude") else {
        return WindowedAgentUsageFacts::default();
    };
    WindowedAgentUsageFacts {
        five_hour_remaining_percent: official
            .get("primary_used_percent")
            .and_then(serde_json::Value::as_f64)
            .map(remaining_percent_from_used),
        weekly_remaining_percent: official
            .get("secondary_used_percent")
            .and_then(serde_json::Value::as_f64)
            .map(remaining_percent_from_used),
        ..WindowedAgentUsageFacts::default()
    }
}

fn window_minutes_to_seconds(minutes: u64) -> Option<u64> {
    minutes
        .checked_mul(AGENT_USAGE_MINUTE_SECONDS)
        .filter(|seconds| *seconds > 0)
}

fn remaining_percent_from_used(used_percent: f64) -> u64 {
    (100.0 - used_percent).clamp(0.0, 100.0).round() as u64
}

fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    (start <= end).then_some(&raw[start..=end])
}

fn first_u64_at(value: &serde_json::Value, paths: &[&[&str]]) -> Option<u64> {
    paths
        .iter()
        .find_map(|path| nested_value(value, path)?.as_u64())
}

fn nested_value<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn write_json_value_atomic(
    path: &std::path::Path,
    value: &serde_json::Value,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create usage cache parent: {error}"))?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("agent_usage_cache");
    let tmp_path = path.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()));
    let raw = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    std::fs::write(&tmp_path, raw)
        .map_err(|error| format!("failed to write temporary usage cache: {error}"))?;
    std::fs::rename(&tmp_path, path)
        .map_err(|error| format!("failed to install usage cache: {error}"))?;
    Ok(())
}

pub fn render_codex_usage_status_widget(
    facts: &WindowedAgentUsageFacts,
    display: AgentUsageDisplay,
) -> String {
    render_codex_usage_status_widget_for_periods(
        facts,
        display,
        &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
    )
}

pub fn render_codex_usage_status_widget_for_periods(
    facts: &WindowedAgentUsageFacts,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    render_agent_usage_status_widget(
        "codex",
        &render_codex_usage_summary_for_periods(facts, display, periods),
    )
}

pub fn render_codex_usage_status_body_for_periods(
    facts: &WindowedAgentUsageFacts,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    render_agent_usage_status_body(
        "codex",
        &render_codex_usage_summary_for_periods(facts, display, periods),
    )
}

pub fn render_codex_usage_summary(
    facts: &WindowedAgentUsageFacts,
    display: AgentUsageDisplay,
) -> String {
    render_codex_usage_summary_for_periods(
        facts,
        display,
        &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
    )
}

pub fn render_codex_usage_summary_for_periods(
    facts: &WindowedAgentUsageFacts,
    display: AgentUsageDisplay,
    periods: &[AgentUsagePeriod],
) -> String {
    let mut parts = Vec::new();
    for period in periods {
        let (tokens, remaining_percent) = usage_period_values(facts, *period);
        let label = codex_reset_window_label(facts, *period)
            .unwrap_or_else(|| period.short_label().to_string());
        if let Some(part) = render_codex_usage_window(&label, tokens, remaining_percent, display) {
            parts.push(part);
        }
    }
    parts.join(" ")
}

pub fn render_windowed_agent_usage_status_widget(
    label: &str,
    facts: &WindowedAgentUsageFacts,
    periods: &[AgentUsagePeriod],
    display: AgentUsageDisplay,
) -> String {
    render_agent_usage_status_widget(
        label,
        &render_windowed_agent_usage_summary(facts, periods, display),
    )
}

pub fn render_windowed_agent_usage_status_body(
    label: &str,
    facts: &WindowedAgentUsageFacts,
    periods: &[AgentUsagePeriod],
    display: AgentUsageDisplay,
) -> String {
    render_agent_usage_status_body(
        label,
        &render_windowed_agent_usage_summary(facts, periods, display),
    )
}

pub fn render_windowed_agent_usage_summary(
    facts: &WindowedAgentUsageFacts,
    periods: &[AgentUsagePeriod],
    display: AgentUsageDisplay,
) -> String {
    let mut parts = Vec::new();
    for period in periods {
        let (tokens, remaining_percent) = usage_period_values(facts, *period);
        if let Some(part) = render_windowed_agent_usage_window(
            period.short_label(),
            tokens,
            remaining_percent,
            display,
        ) {
            parts.push(part);
        }
    }
    parts.join(" ")
}

fn render_agent_usage_status_widget(label: &str, summary: &str) -> String {
    let body = render_agent_usage_status_body(label, summary);
    if body.is_empty() {
        String::new()
    } else {
        format!(" [{body}]")
    }
}

fn render_agent_usage_status_body(label: &str, summary: &str) -> String {
    if summary.is_empty() {
        String::new()
    } else {
        format!("{label} {summary}")
    }
}

fn render_codex_usage_window(
    label: &str,
    tokens: Option<u64>,
    remaining_percent: Option<u64>,
    display: AgentUsageDisplay,
) -> Option<String> {
    let mut pieces = vec![label.to_string()];
    match display {
        AgentUsageDisplay::Token => {
            pieces.push(format_agent_usage_token_count(tokens?));
        }
        AgentUsageDisplay::Quota => {
            pieces.push(format_quota_percent(remaining_percent?));
        }
        AgentUsageDisplay::Both => {
            if let Some(tokens) = tokens {
                pieces.push(format_agent_usage_token_count(tokens));
            }
            if let Some(remaining_percent) = remaining_percent {
                pieces.push(format_quota_percent(remaining_percent));
            }
            if pieces.len() == 1 {
                return None;
            }
        }
    }
    Some(pieces.join("|"))
}

fn render_windowed_agent_usage_window(
    label: &str,
    tokens: Option<u64>,
    remaining_percent: Option<u64>,
    display: AgentUsageDisplay,
) -> Option<String> {
    let mut pieces = vec![label.to_string()];
    match display {
        AgentUsageDisplay::Token => {
            pieces.push(format_agent_usage_token_count(tokens?));
        }
        AgentUsageDisplay::Quota => {
            pieces.push(match remaining_percent {
                Some(percent) => format_quota_percent(percent),
                None if tokens.is_some() => "n/a".to_string(),
                None => return None,
            });
        }
        AgentUsageDisplay::Both => {
            if let Some(tokens) = tokens {
                pieces.push(format_agent_usage_token_count(tokens));
            }
            if let Some(remaining_percent) = remaining_percent {
                pieces.push(format_quota_percent(remaining_percent));
            }
            if pieces.len() == 1 {
                return None;
            }
        }
    }
    Some(pieces.join("|"))
}

fn usage_period_values(
    facts: &WindowedAgentUsageFacts,
    period: AgentUsagePeriod,
) -> (Option<u64>, Option<u64>) {
    match period {
        AgentUsagePeriod::FiveHour => (facts.five_hour_tokens, facts.five_hour_remaining_percent),
        AgentUsagePeriod::Weekly => (facts.weekly_tokens, facts.weekly_remaining_percent),
        AgentUsagePeriod::Monthly => (facts.monthly_tokens, facts.monthly_remaining_percent),
    }
}

fn codex_reset_window_label(
    facts: &WindowedAgentUsageFacts,
    period: AgentUsagePeriod,
) -> Option<String> {
    let now = facts.updated_at_unix_seconds?;
    let (reset_at, window_seconds) = match period {
        AgentUsagePeriod::FiveHour => (
            facts.five_hour_reset_at_unix_seconds?,
            facts.five_hour_window_seconds?,
        ),
        AgentUsagePeriod::Weekly => (
            facts.weekly_reset_at_unix_seconds?,
            facts.weekly_window_seconds?,
        ),
        AgentUsagePeriod::Monthly => return None,
    };
    format_reset_window_label(reset_at, window_seconds, now)
}

fn format_agent_usage_token_count(tokens: u64) -> String {
    if tokens >= 1_000_000_000 {
        format_scaled_agent_usage_count(tokens as f64 / 1_000_000_000.0, "B")
    } else if tokens >= 1_000_000 {
        format_scaled_agent_usage_count(tokens as f64 / 1_000_000.0, "M")
    } else if tokens >= 1_000 {
        format!("{}k", tokens / 1_000)
    } else {
        tokens.to_string()
    }
}

fn format_scaled_agent_usage_count(value: f64, suffix: &str) -> String {
    let raw = if value >= 100.0 {
        format!("{value:.0}")
    } else if value >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    };
    let trimmed = if raw.contains('.') {
        raw.trim_end_matches('0').trim_end_matches('.')
    } else {
        raw.as_str()
    };
    format!("{trimmed}{suffix}")
}

fn format_reset_window_label(
    reset_at_unix_seconds: u64,
    window_seconds: u64,
    now_unix_seconds: u64,
) -> Option<String> {
    if window_seconds == 0 {
        return None;
    }
    let remaining_seconds = reset_at_unix_seconds
        .saturating_sub(now_unix_seconds)
        .min(window_seconds);
    let elapsed_seconds = window_seconds.saturating_sub(remaining_seconds);
    Some(format!(
        "{}/{}",
        format_reset_window_position_duration(elapsed_seconds, window_seconds),
        format_reset_window_total_duration(window_seconds)
    ))
}

fn format_reset_window_position_duration(seconds: u64, window_seconds: u64) -> String {
    const MINUTE_SECONDS: u64 = 60;
    const HOUR_SECONDS: u64 = 60 * MINUTE_SECONDS;
    const DAY_SECONDS: u64 = 24 * HOUR_SECONDS;

    if window_seconds >= DAY_SECONDS {
        let days = seconds / DAY_SECONDS;
        let hours = (seconds % DAY_SECONDS) / HOUR_SECONDS;
        if days > 0 && hours > 0 {
            format!("{days}d{hours}h")
        } else if days > 0 {
            format!("{days}d")
        } else if hours > 0 {
            format!("{hours}h")
        } else {
            "0h".to_string()
        }
    } else if window_seconds >= HOUR_SECONDS {
        let hours = seconds / HOUR_SECONDS;
        let minutes = elapsed_minutes_after_hour(seconds);
        if hours > 0 && minutes > 0 {
            format!("{hours}h{minutes}m")
        } else if hours > 0 {
            format!("{hours}h")
        } else {
            format!("{minutes}m")
        }
    } else if window_seconds >= MINUTE_SECONDS {
        if seconds > 0 {
            format!("{}m", seconds.div_ceil(MINUTE_SECONDS))
        } else {
            "0m".to_string()
        }
    } else if seconds > 0 {
        format!("{seconds}s")
    } else {
        "0s".to_string()
    }
}

fn elapsed_minutes_after_hour(seconds: u64) -> u64 {
    const HOUR_SECONDS: u64 = 60 * 60;
    const MINUTE_SECONDS: u64 = 60;

    let minutes = (seconds % HOUR_SECONDS) / MINUTE_SECONDS;
    if seconds > 0 && seconds < HOUR_SECONDS && minutes == 0 {
        1
    } else {
        minutes
    }
}

fn format_reset_window_total_duration(seconds: u64) -> String {
    const MINUTE_SECONDS: u64 = 60;
    const HOUR_SECONDS: u64 = 60 * MINUTE_SECONDS;
    const DAY_SECONDS: u64 = 24 * HOUR_SECONDS;

    if seconds % DAY_SECONDS == 0 {
        format!("{}d", seconds / DAY_SECONDS)
    } else if seconds % HOUR_SECONDS == 0 {
        format!("{}h", seconds / HOUR_SECONDS)
    } else if seconds % MINUTE_SECONDS == 0 {
        format!("{}m", seconds / MINUTE_SECONDS)
    } else {
        format!("{seconds}s")
    }
}

fn format_quota_percent(percent: u64) -> String {
    format!("{}%", percent.min(100))
}

pub fn render_zjstatus_tab_label_formats(mode: &str) -> Result<TabLabelFormats, BarRenderError> {
    render_zjstatus_tab_label_formats_with_style(mode, &DARK_BAR_STYLE)
}

fn render_zjstatus_tab_label_formats_with_style(
    mode: &str,
    style: &BarStyle,
) -> Result<TabLabelFormats, BarRenderError> {
    match mode {
        TAB_LABEL_MODE_FULL => Ok(render_tab_label_formats(style, true)),
        TAB_LABEL_MODE_COMPACT => Ok(render_tab_label_formats(style, false)),
        _ => Err(BarRenderError::InvalidTabLabelMode {
            mode: mode.to_string(),
        }),
    }
}

fn render_tab_label_formats(style: &BarStyle, include_name: bool) -> TabLabelFormats {
    let name = if include_name { " {name}" } else { "" };
    TabLabelFormats {
        tab_normal: format!(
            r##"tab_normal   "{} [{{index}}]{} ""##,
            style.tab_normal, name
        ),
        tab_normal_fullscreen: format!(
            r##"tab_normal_fullscreen "{} [{{index}}]{} [] ""##,
            style.tab_normal, name
        ),
        tab_normal_sync: format!(
            r##"tab_normal_sync       "{} [{{index}}]{} <> ""##,
            style.tab_normal, name
        ),
        tab_normal_bell: format!(
            r##"tab_normal_bell "{} [{{index}}]{} {{sync_indicator}}{{fullscreen_indicator}}""##,
            style.tab_bell, name
        ),
        tab_normal_flashing_bell: format!(
            r##"tab_normal_flashing_bell "{} [{{index}}]{} {{sync_indicator}}{{fullscreen_indicator}}""##,
            style.tab_flashing_bell, name
        ),
        tab_active: format!(
            r##"tab_active   "{} [{{index}}]{} {{floating_indicator}}""##,
            style.tab_active, name
        ),
        tab_active_fullscreen: format!(
            r##"tab_active_fullscreen "{} [{{index}}]{} {{fullscreen_indicator}}""##,
            style.tab_active, name
        ),
        tab_active_sync: format!(
            r##"tab_active_sync       "{} [{{index}}]{} {{sync_indicator}}""##,
            style.tab_active, name
        ),
        tab_rename: format!(
            r##"tab_rename    "{} {{index}} {{name}} {{floating_indicator}} ""##,
            style.tab_active
        ),
    }
}

fn render_widget_with_style(
    widget: &str,
    request: &BarRenderRequest,
    style: &BarStyle,
    chrome: WidgetChrome,
    first: bool,
) -> Result<String, BarRenderError> {
    let prefix = chrome.styled_segment_prefix(first, style);
    let inline = |style_prefix: &str, text: String| {
        format!("{}{}{}", prefix, style_prefix, chrome.frame.frame(&text))
    };
    let command = |style_prefix: &str, placeholder: &str| {
        format!("{}{}{}", prefix, style_prefix, placeholder)
    };
    match widget {
        WIDGET_SESSION => Ok(inline(style.session, "{session}".to_string())),
        WIDGET_EDITOR => Ok(inline(style.widget, format!(" {}", request.editor_label))),
        WIDGET_SHELL => Ok(inline(style.widget, format!("❯{}", request.shell_label))),
        WIDGET_TERM => Ok(command(style.widget, COMMAND_TERM)),
        WIDGET_WORKSPACE => Ok(PIPE_WORKSPACE.to_string()),
        WIDGET_CLAUDE_USAGE => Ok(command(style.usage, COMMAND_CLAUDE_USAGE)),
        WIDGET_CODEX_USAGE => Ok(command(style.usage, COMMAND_CODEX_USAGE)),
        WIDGET_OPENCODE_GO_USAGE => Ok(command(style.usage, COMMAND_OPENCODE_GO_USAGE)),
        WIDGET_CPU => Ok(command(style.system_usage, COMMAND_CPU)),
        WIDGET_RAM => Ok(command(style.system_usage, COMMAND_RAM)),
        _ => Err(BarRenderError::InvalidWidgetTrayEntry {
            entry: widget.to_string(),
        }),
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    fn render_request(widget_tray: &[&str]) -> BarRenderRequest {
        BarRenderRequest {
            widget_tray: widget_tray
                .iter()
                .map(|widget| widget.to_string())
                .collect(),
            editor_label: "hx".to_string(),
            shell_label: "nu".to_string(),
            terminal_label: "ghostty".to_string(),
            custom_text: String::new(),
        }
    }

    fn temp_test_dir(name: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("nova_bar_{name}_{}_{}", std::process::id(), unique));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[cfg(unix)]
    fn write_tokenusage_provider_script(bin_dir: &std::path::Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        std::fs::create_dir_all(bin_dir).unwrap();
        let path = bin_dir.join("tu");
        let shell = std::env::var_os("PATH")
            .and_then(|path| {
                std::env::split_paths(&path)
                    .map(|entry| entry.join("sh"))
                    .find(|candidate| candidate.is_file())
            })
            .unwrap_or_else(|| std::path::PathBuf::from("/bin/sh"));
        let script = body.replacen("#!/usr/bin/env sh", &format!("#!{}", shell.display()), 1);
        std::fs::write(&path, script).unwrap();
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }

    fn write_opencode_go_test_db(path: &std::path::Path, now: u64) {
        let connection = rusqlite::Connection::open(path).unwrap();
        connection
            .execute(
                "CREATE TABLE message (time_created INTEGER NOT NULL, data TEXT NOT NULL)",
                [],
            )
            .unwrap();
        let rows = [
            (
                now.saturating_sub(60),
                r#"{"role":"assistant","providerID":"opencode-go","tokens":{"total":1000},"cost":3.0}"#,
            ),
            (
                now.saturating_sub(6 * 60 * 60),
                r#"{"role":"assistant","providerID":"opencode-go","tokens":{"input":1200,"output":800},"cost":6.0}"#,
            ),
            (
                now.saturating_sub(8 * 24 * 60 * 60),
                r#"{"role":"assistant","providerID":"opencode-go","tokens":{"total":30000},"cost":30.0}"#,
            ),
            (
                now.saturating_sub(60),
                r#"{"role":"user","providerID":"opencode-go","tokens":{"total":900000},"cost":99.0}"#,
            ),
            (
                now.saturating_sub(60),
                r#"{"role":"assistant","providerID":"other","tokens":{"total":900000},"cost":99.0}"#,
            ),
        ];
        for (created_at, data) in rows {
            connection
                .execute(
                    "INSERT INTO message (time_created, data) VALUES (?1, ?2)",
                    rusqlite::params![unix_millis_from_seconds_saturating(created_at), data],
                )
                .unwrap();
        }
    }

    // Defends: the bar registry preserves the existing zjstatus widgets and exact segment syntax.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_existing_widget_tray_segments() {
        let rendered = render_widget_tray_segment(&render_request(DEFAULT_WIDGET_TRAY)).unwrap();

        assert_eq!(
            rendered,
            " #[fg=#ff0088,bold]{session} #[fg=#6c7086,bold]• #[fg=#00ff88,bold] hx #[fg=#6c7086,bold]• #[fg=#00ff88,bold]❯nu #[fg=#6c7086,bold]• #[fg=#00ff88,bold]{command_term} #[fg=#6c7086,bold]• #[fg=#bb88ff,bold]{command_codex_usage} #[fg=#6c7086,bold]• #[fg=#ff6600]{command_cpu} #[fg=#6c7086,bold]• #[fg=#ff6600]{command_ram}"
        );
    }

    // Defends: square-frame mode treats session as a normal framed widget instead of a hardcoded bare placeholder.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_session_with_square_widget_frame() {
        let rendered = render_widget_tray_segment_with_style(
            &render_request(&["session", "editor"]),
            &DARK_BAR_STYLE,
            WidgetChrome::parse(WIDGET_FRAME_SQUARE, WIDGET_SEPARATOR_SPACE).unwrap(),
        )
        .unwrap();

        assert_eq!(
            rendered,
            " #[fg=#ff0088,bold][{session}] #[fg=#00ff88,bold][ hx]"
        );
    }

    // Defends: a deliberately empty tray stays empty rather than introducing stray spacing.
    // Strength: defect=1 behavior=2 resilience=1 cost=2 uniqueness=2 total=8/10
    #[test]
    fn renders_empty_widget_tray_without_padding() {
        let rendered = render_widget_tray_segment(&render_request(&[])).unwrap();

        assert_eq!(rendered, "");
    }

    // Defends: session name display is controlled by the widget tray instead of being hardcoded into the runtime template.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_session_name_as_widget_tray_segment() {
        let rendered = render_widget_tray_segment(&render_request(&["session"])).unwrap();

        assert_eq!(rendered, " #[fg=#ff0088,bold]{session}");
    }

    // Regression: session follows the configured widget separator when it is not first.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_session_with_configured_separator_when_not_first() {
        let rendered = render_widget_tray_segment(&render_request(&["editor", "session"])).unwrap();

        assert_eq!(
            rendered,
            " #[fg=#00ff88,bold] hx #[fg=#6c7086,bold]• #[fg=#ff0088,bold]{session}"
        );
    }

    // Regression: the active-tab workspace widget renders through a pushed pipe placeholder, not an async command result that can lag behind tab focus.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_workspace_widget_as_pushed_pipe_placeholder() {
        let rendered = render_widget_tray_segment(&render_request(&["workspace"])).unwrap();

        assert_eq!(rendered, "{pipe_workspace}");
    }

    // Regression: agent usage widgets render through cache readers so expensive providers are never polled by zjstatus.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_agent_usage_widgets_as_cached_command_placeholders() {
        let rendered = render_widget_tray_segment(&render_request(&[
            "claude_usage",
            "codex_usage",
            "opencode_go_usage",
        ]))
        .unwrap();

        assert_eq!(
            rendered,
            " #[fg=#bb88ff,bold]{command_claude_usage} #[fg=#6c7086,bold]• #[fg=#bb88ff,bold]{command_codex_usage} #[fg=#6c7086,bold]• #[fg=#bb88ff,bold]{command_opencode_go_usage}"
        );
    }

    fn runtime_bar_config() -> NovaRuntimeBarConfig {
        NovaRuntimeBarConfig {
            zjstatus_plugin_url: "file:/runtime/share/zjstatus.wasm".to_string(),
            widget_tray: vec![
                "session".to_string(),
                "editor".to_string(),
                "workspace".to_string(),
                "cpu".to_string(),
            ],
            widget_frame: WIDGET_FRAME_NONE.to_string(),
            widget_separator: WIDGET_SEPARATOR_DOT.to_string(),
            editor_label: "hx".to_string(),
            shell_label: "nu".to_string(),
            terminal_label: "ghostty".to_string(),
            custom_text: "demo".to_string(),
            appearance_mode: "dark".to_string(),
            tab_label_mode: "compact".to_string(),
            nu_bin: "/runtime/bin/nu".to_string(),
            yzx_control_bin: "/runtime/bin/yzx_control".to_string(),
            nova_bar_widget_bin: "/runtime/bin/nova_bar_widget".to_string(),
            runtime_dir: "/runtime".to_string(),
            claude_usage_display: "both".to_string(),
            claude_usage_periods: vec!["5h".to_string(), "week".to_string()],
            codex_usage_display: "quota".to_string(),
            codex_usage_periods: vec!["5h".to_string(), "week".to_string()],
            opencode_go_usage_display: "both".to_string(),
            opencode_go_usage_periods: vec![
                "5h".to_string(),
                "week".to_string(),
                "month".to_string(),
            ],
        }
    }

    fn runtime_assignment<'a>(rendered: &'a str, key: &str) -> &'a str {
        let line = rendered
            .lines()
            .find(|line| line.split_whitespace().next() == Some(key))
            .unwrap_or_else(|| panic!("missing runtime assignment {key}"));
        let (_, quoted) = line
            .split_once('"')
            .unwrap_or_else(|| panic!("runtime assignment {key} has no opening quote"));
        quoted
            .rsplit_once('"')
            .unwrap_or_else(|| panic!("runtime assignment {key} has no closing quote"))
            .0
    }

    // Regression: integrated Yazelix KDL shape is owned by the child runtime template, not hardcoded in main or rebuilt as Rust string assembly.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_nova_runtime_controller_block_from_template() {
        let rendered = render_nova_runtime_background_plugin_block(&runtime_bar_config()).unwrap();

        assert!(NOVA_RUNTIME_BAR_TEMPLATE.contains(RUNTIME_ACTIVE_THEME_FIELDS_PLACEHOLDER));
        assert!(rendered.starts_with(r#""file:/runtime/share/zjstatus.wasm" {"#));
        assert!(!rendered.contains("tab_display_count"));
        assert!(rendered.contains(r#"host_theme_mode "dark""#));
        assert!(rendered.contains(r##"host_theme_light_tab_normal "#[fg=#5c5f77] [{index}] ""##));
        assert_eq!(
            runtime_assignment(&rendered, "format_right"),
            "#[fg=#ff0088,bold]{session}{segment}#[fg=#00ff88,bold] hx{segment}{pipe_workspace}{segment}#[fg=#ff6600]{command_cpu}{segment}#[fg=#ffff00,bold][demo]{segment}#[fg=#00ccff,bold]{command_version}"
        );
        assert!(rendered.contains(r#"format_left   "{tabs}""#));
        assert!(!rendered.contains("{mode}"));
        assert_eq!(
            runtime_assignment(&rendered, "tab_normal"),
            "#[fg=#ffff00] [{index}] "
        );
        assert!(rendered.contains(
            r##"tab_normal_bell "#[fg=#ff0088,bold] [{index}] {sync_indicator}{fullscreen_indicator}""##
        ));
        assert!(rendered.contains(r##"tab_bell_indicator       """##));
        assert!(rendered.contains(r##"pipe_workspace_format "#[fg=#00ff88,bold]{output}""##));
        assert!(rendered.contains(r#"command_term_command "/runtime/bin/nova_bar_widget term""#));
        assert!(rendered.contains(r##"command_term_format "{stdout}""##));
        assert!(rendered.contains(r##"command_term_rendermode "raw""##));
        assert_eq!(
            runtime_assignment(&rendered, "command_term_placeholder"),
            " …"
        );
        assert!(!rendered.contains("command_yazelix_tabs_command"));
        assert!(!rendered.contains("command_workspace_command"));
        assert!(!rendered.contains("command_cursor"));
        assert!(rendered.contains(
            r#"command_codex_usage_command "/runtime/bin/nova_bar_widget codex --display quota --periods 5h,week --widget-frame none --widget-separator empty --widget-first false""#
        ));
        assert!(
            rendered
                .contains(r#"--widget-frame none --widget-separator empty --widget-first false"#)
        );
        assert!(rendered.contains(r##"command_codex_usage_format "{stdout}""##));
        assert!(rendered.contains(r##"command_codex_usage_rendermode "raw""##));
        assert_eq!(
            runtime_assignment(&rendered, "command_codex_usage_placeholder"),
            "codex …"
        );
        assert_eq!(
            runtime_assignment(&rendered, "command_cpu_placeholder"),
            "cpu …"
        );
        assert_eq!(
            runtime_assignment(&rendered, "command_ram_placeholder"),
            "ram …"
        );
        assert!(rendered.contains(
            r#"command_version_command "/runtime/bin/nova_bar_widget version --runtime-dir /runtime""#
        ));
        assert_eq!(
            runtime_assignment(&rendered, "command_version_placeholder"),
            "…"
        );
        assert!(!rendered.contains(RUNTIME_PLACEHOLDER_PREFIX));
    }

    // Regression: generated command args use canonical enum values after validation, not raw config text.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn runtime_plugin_normalizes_widget_chrome_args() {
        let mut config = runtime_bar_config();
        config.widget_frame = " square ".to_string();
        config.widget_separator = " pipe ".to_string();

        let rendered = render_nova_runtime_background_plugin_block(&config).unwrap();

        assert!(rendered.contains(
            r#"command_cpu_command "/runtime/bin/nova_bar_widget cpu --widget-frame square --widget-separator empty --widget-first false""#
        ));
        assert!(rendered.contains(r##"pipe_workspace_format "#[fg=#00ff88,bold][{output}]""##));
        assert!(rendered.contains(r##"command_cpu_format "{stdout}""##));
        assert_eq!(
            runtime_assignment(&rendered, "command_codex_usage_placeholder"),
            "[codex …]"
        );
        assert_eq!(
            runtime_assignment(&rendered, "command_cpu_placeholder"),
            "[cpu …]"
        );
        assert_eq!(
            runtime_assignment(&rendered, "format_right"),
            "#[fg=#ff0088,bold][{session}]{segment}#[fg=#00ff88,bold][ hx]{segment}{pipe_workspace}{segment}#[fg=#ff6600]{command_cpu}{segment}#[fg=#ffff00,bold][demo]{segment}#[fg=#00ccff,bold]{command_version}"
        );
    }

    // Defends: unsupported widget chrome settings fail before emitting invalid zjstatus KDL.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn runtime_plugin_rejects_invalid_widget_chrome() {
        let mut config = runtime_bar_config();
        config.widget_frame = "curly".to_string();
        let frame_error = render_nova_runtime_background_plugin_block(&config).unwrap_err();
        assert_eq!(frame_error.code(), "invalid_widget_frame");

        let mut config = runtime_bar_config();
        config.widget_separator = "comma".to_string();
        let separator_error = render_nova_runtime_background_plugin_block(&config).unwrap_err();
        assert_eq!(separator_error.code(), "invalid_widget_separator");
    }

    // Defends: light appearance uses a purpose-built status-bar palette rather than dark-mode neon on a pale terminal.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_nova_runtime_controller_block_with_light_palette() {
        let mut config = runtime_bar_config();
        config.appearance_mode = "light".to_string();
        let rendered = render_nova_runtime_background_plugin_block(&config).unwrap();

        assert!(rendered.contains(r#"host_theme_mode "light""#));
        assert!(rendered.contains(r##"host_theme_dark_tab_normal "#[fg=#ffff00] [{index}] ""##));
        assert_eq!(
            runtime_assignment(&rendered, "format_right"),
            "#[fg=#7c3f97,bold]{session}{segment}#[fg=#2f7d32,bold] hx{segment}{pipe_workspace}{segment}#[fg=#a24f00]{command_cpu}{segment}#[fg=#9a5a00,bold][demo]{segment}#[fg=#1e66f5,bold]{command_version}"
        );
        assert!(!rendered.contains("mode_normal"));
        assert_eq!(
            runtime_assignment(&rendered, "tab_active"),
            "#[bg=#ccd0da,fg=#303446,bold] [{index}] {floating_indicator}"
        );
        assert!(rendered.contains(
            r##"tab_normal_bell "#[fg=#b4637a,bold] [{index}] {sync_indicator}{fullscreen_indicator}""##
        ));
        assert!(rendered.contains(
            r##"tab_normal_flashing_bell "#[bg=#b4637a,fg=#fffaf3,bold] [{index}] {sync_indicator}{fullscreen_indicator}""##
        ));
        assert!(rendered.contains(r##"pipe_workspace_format "#[fg=#2f7d32,bold]{output}""##));
        assert!(rendered.contains(r##"command_codex_usage_format "{stdout}""##));
        assert!(rendered.contains(r##"command_codex_usage_rendermode "raw""##));
        assert!(rendered.contains(r##"command_cpu_format "{stdout}""##));
        assert!(!rendered.contains("command_cursor"));
    }

    // Defends: launch rendering and live host-theme switching consume one complete field set instead of drifting key by key.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn active_and_live_runtime_theme_fields_stay_in_parity() {
        let mut config = runtime_bar_config();
        let chrome = WidgetChrome::parse(&config.widget_frame, &config.widget_separator).unwrap();
        let dark_theme = runtime_theme_fields(&config, &DARK_BAR_STYLE, chrome).unwrap();
        let light_theme = runtime_theme_fields(&config, &LIGHT_BAR_STYLE, chrome).unwrap();
        let keys = dark_theme
            .iter()
            .map(|(key, _)| *key)
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(keys.len(), dark_theme.len(), "theme keys must be unique");
        assert_eq!(
            keys,
            light_theme.iter().map(|(key, _)| *key).collect(),
            "dark and light palettes must cover the same fields"
        );

        for (mode, active_theme) in [
            (APPEARANCE_MODE_DARK, &dark_theme),
            (APPEARANCE_MODE_LIGHT, &light_theme),
        ] {
            config.appearance_mode = mode.to_string();
            let rendered = render_nova_runtime_background_plugin_block(&config).unwrap();

            for (key, value) in active_theme {
                assert_eq!(runtime_assignment(&rendered, key), escape_kdl_string(value));
                assert_eq!(
                    runtime_assignment(&rendered, key),
                    runtime_assignment(&rendered, &format!("host_theme_{mode}_{key}")),
                    "active {mode} field {key} must equal its live overlay"
                );
            }
        }
    }

    // Defends: vanilla users keep a simple standalone preset and do not inherit integrated Yazelix runtime placeholders.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=2 total=8/10
    #[test]
    fn standalone_preset_keeps_runtime_template_separate() {
        let standalone = include_str!("../presets/nova_bar.kdl");

        assert!(standalone.contains("__NOVA_BAR_ZJSTATUS_WASM__"));
        assert!(!standalone.contains(RUNTIME_PLACEHOLDER_PREFIX));
        assert!(!standalone.contains("yzx_control"));
    }

    // Defends: CPU widget math uses /proc/stat deltas and hides malformed samples.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn calculates_cpu_widget_percent_from_proc_stat_snapshots() {
        let before = "cpu  100 0 50 850 0 0 0 0\n";
        let after = "cpu  130 0 70 900 0 0 0 0\n";

        assert_eq!(cpu_usage_percent_from_proc_stat(before, after), Some(50));
        assert_eq!(cpu_usage_percent_from_proc_stat("cpu  1 2\n", after), None);
        assert_eq!(cpu_usage_percent_from_proc_stat(after, before), None);
    }

    // Defends: RAM widget math uses MemAvailable rather than Yazelix/Nushell runtime state.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn calculates_ram_widget_percent_from_meminfo() {
        let meminfo = "\
MemTotal:       1000000 kB
MemFree:         100000 kB
MemAvailable:   250000 kB
";

        assert_eq!(ram_usage_percent_from_meminfo(meminfo), Some(75));
        assert_eq!(
            ram_usage_percent_from_meminfo("MemTotal: 0 kB\nMemAvailable: 0 kB\n"),
            None
        );
        assert_eq!(
            ram_usage_percent_from_meminfo("MemTotal: 10 kB\nMemAvailable: 20 kB\n"),
            None
        );
    }

    // Defends: repeated CPU/RAM widget invocations can read one shared sampled value instead of resampling.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn system_usage_cache_serves_fresh_cpu_and_ram_values() {
        let dir = temp_test_dir("system_usage_cache");
        let path = dir.join("system_usage_cache_v1.json");
        let cache = SystemUsageCache {
            schema_version: SYSTEM_USAGE_CACHE_SCHEMA_VERSION,
            captured_unix_millis: 10_000,
            cpu_percent: Some(37),
            ram_percent: Some(64),
        };

        write_system_usage_cache(&path, &cache).unwrap();

        let fresh = read_fresh_system_usage_cache(&path, 12_000).unwrap();
        assert_eq!(
            system_usage_widget_text(&fresh, SystemUsageWidget::Cpu),
            "37%"
        );
        assert_eq!(
            system_usage_widget_text(&fresh, SystemUsageWidget::Ram),
            "64%"
        );
        assert!(read_fresh_system_usage_cache(&path, 16_000).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    // Regression: abandoned lock files must remain reusable while live samplers stay exclusive.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn system_usage_cache_lock_recovers_and_stays_exclusive() {
        let dir = temp_test_dir("system_usage_lock");
        let path = dir.join("system_usage_cache_v1.json");
        let lock_path = system_usage_cache_lock_path(&path);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(lock_path, []).unwrap();

        let first = acquire_system_usage_cache_lock(&path).expect("first lock");
        assert!(acquire_system_usage_cache_lock(&path).is_none());
        drop(first);
        assert!(acquire_system_usage_cache_lock(&path).is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    // Regression: simultaneous CPU/RAM widget refreshes should not blink to unknown while one process samples.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn system_usage_cache_lock_contention_serves_recent_stale_values() {
        let dir = temp_test_dir("system_usage_stale_lock");
        let path = dir.join("system_usage_cache_v1.json");
        let cache = SystemUsageCache {
            schema_version: SYSTEM_USAGE_CACHE_SCHEMA_VERSION,
            captured_unix_millis: 10_000,
            cpu_percent: Some(37),
            ram_percent: Some(64),
        };
        write_system_usage_cache(&path, &cache).unwrap();

        let lock = acquire_system_usage_cache_lock(&path).expect("lock");
        let stale = refresh_system_usage_cache_if_needed(&path, 16_000).unwrap();
        assert_eq!(
            system_usage_widget_text(&stale, SystemUsageWidget::Cpu),
            "37%"
        );
        assert_eq!(
            system_usage_widget_text(&stale, SystemUsageWidget::Ram),
            "64%"
        );
        drop(lock);
        let _ = std::fs::remove_dir_all(dir);
    }

    // Regression: the first simultaneous CPU/RAM widget pair should wait for the sampler owner instead of rendering unknown.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn system_usage_cache_lock_contention_waits_for_initial_sample() {
        let dir = temp_test_dir("system_usage_cold_lock");
        let path = dir.join("system_usage_cache_v1.json");

        let lock = acquire_system_usage_cache_lock(&path).expect("lock");
        let writer_path = path.clone();
        let writer = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let cache = SystemUsageCache {
                schema_version: SYSTEM_USAGE_CACHE_SCHEMA_VERSION,
                captured_unix_millis: 10_000,
                cpu_percent: Some(37),
                ram_percent: Some(64),
            };
            write_system_usage_cache(&writer_path, &cache).unwrap();
        });

        let sampled = refresh_system_usage_cache_if_needed(&path, 10_000).unwrap();
        assert_eq!(
            system_usage_widget_text(&sampled, SystemUsageWidget::Cpu),
            "37%"
        );
        assert_eq!(
            system_usage_widget_text(&sampled, SystemUsageWidget::Ram),
            "64%"
        );

        writer.join().unwrap();
        drop(lock);
        let _ = std::fs::remove_dir_all(dir);
    }

    // Regression: abandoned refresh locks must not make arbitrarily old system-usage values look live.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn system_usage_cache_lock_contention_rejects_too_old_values() {
        let dir = temp_test_dir("system_usage_too_old_lock");
        let path = dir.join("system_usage_cache_v1.json");
        let cache = SystemUsageCache {
            schema_version: SYSTEM_USAGE_CACHE_SCHEMA_VERSION,
            captured_unix_millis: 10_000,
            cpu_percent: Some(37),
            ram_percent: Some(64),
        };
        write_system_usage_cache(&path, &cache).unwrap();

        let lock = acquire_system_usage_cache_lock(&path).expect("lock");
        assert!(refresh_system_usage_cache_if_needed(&path, 46_000).is_none());
        drop(lock);
        let _ = std::fs::remove_dir_all(dir);
    }

    // Regression: dynamic command placeholders must preserve stable spacing around safe widgets.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn dynamic_widgets_do_not_leave_spacing_artifacts() {
        let rendered =
            render_widget_tray_segment(&render_request(&["editor", "workspace", "shell"])).unwrap();

        assert_eq!(
            rendered,
            " #[fg=#00ff88,bold] hx{pipe_workspace} #[fg=#6c7086,bold]• #[fg=#00ff88,bold]❯nu"
        );
    }

    // Defends: custom text remains trim-aware and does not reserve bar space when absent.
    // Strength: defect=1 behavior=2 resilience=1 cost=2 uniqueness=2 total=8/10
    #[test]
    fn renders_custom_text_segment_only_when_present() {
        assert_eq!(
            render_custom_text_segment("  verdant-lake  "),
            "#[fg=#ffff00,bold][verdant-lake] "
        );
        assert_eq!(render_custom_text_segment("   "), "");
    }

    // Defends: Codex cached usage facts support quota, token, both, partial quota, and elapsed reset-window labels outside Yazelix.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_codex_usage_widget_from_cached_facts() {
        let facts = WindowedAgentUsageFacts {
            updated_at_unix_seconds: Some(1_700_001_800),
            five_hour_tokens: Some(1_234_567),
            weekly_tokens: Some(345_000_000),
            five_hour_remaining_percent: Some(88),
            weekly_remaining_percent: Some(101),
            five_hour_reset_at_unix_seconds: Some(1_700_018_000),
            weekly_reset_at_unix_seconds: Some(1_700_604_800),
            five_hour_window_seconds: Some(18_000),
            weekly_window_seconds: Some(604_800),
            ..WindowedAgentUsageFacts::default()
        };

        assert_eq!(
            render_codex_usage_status_widget(&facts, AgentUsageDisplay::Quota),
            " [codex 30m/5h|88% 0h/7d|100%]"
        );
        assert_eq!(
            render_codex_usage_status_widget(&facts, AgentUsageDisplay::Token),
            " [codex 30m/5h|1.23M 0h/7d|345M]"
        );
        assert_eq!(
            render_codex_usage_status_widget(&facts, AgentUsageDisplay::Both),
            " [codex 30m/5h|1.23M|88% 0h/7d|345M|100%]"
        );
    }

    // Regression: Codex quota mode hides token-only windows instead of presenting them as quota limits.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn codex_usage_widget_handles_partial_quota_facts() {
        let facts = WindowedAgentUsageFacts {
            five_hour_tokens: Some(42_000),
            ..WindowedAgentUsageFacts::default()
        };

        assert_eq!(
            render_codex_usage_status_widget(&facts, AgentUsageDisplay::Quota),
            ""
        );
        assert_eq!(
            render_codex_usage_status_widget(&facts, AgentUsageDisplay::Both),
            " [codex 5h|42k]"
        );
        assert_eq!(
            render_codex_usage_status_widget(
                &WindowedAgentUsageFacts::default(),
                AgentUsageDisplay::Both
            ),
            ""
        );
    }

    // Defends: quota display refreshes official Codex limits without discovering or caching local history.
    // Regression: an unavailable or slow active-token block used to consume the whole refresh budget before quota was queried.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn codex_usage_widget_refreshes_shared_cache_from_tokenusage_provider() {
        let temp = temp_test_dir("codex_usage_refresh");
        let bin_dir = temp.join("bin");
        let calls_path = bin_dir.join("calls");
        write_tokenusage_provider_script(
            &bin_dir,
            r#"#!/usr/bin/env sh
printf '%s\n' "$*" >> "${0%/*}/calls"
case " $* " in
*" --official-limits "*)
  printf '%s\n' '{"official_codex":{"primary_used_percent":51.0,"secondary_used_percent":20.0,"primary_resets_at":8200,"primary_window_mins":300,"secondary_resets_at":260200,"secondary_window_mins":10080}}'
  ;;
*)
exec sleep 10
  ;;
esac
"#,
        );
        let cache_path = temp.join("agent_usage").join("codex_usage_cache_v4.json");

        let text = codex_usage_widget_text(CodexUsageWidgetOptions {
            cache_path: &cache_path,
            path_var: Some(bin_dir.as_os_str()),
            now_unix_seconds: 1_000,
            max_age_seconds: 60,
            error_backoff_seconds: 120,
            timeout: std::time::Duration::from_secs(1),
            display: AgentUsageDisplay::Quota,
            periods: &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
        })
        .unwrap();

        assert_eq!(text, " [codex 3h/5h|49% 4d/7d|80%]");
        assert_eq!(
            read_codex_usage_shared_cache_value(&cache_path)
                .unwrap()
                .pointer("/codex/status")
                .and_then(serde_json::Value::as_str),
            Some("ok")
        );
        let calls = std::fs::read_to_string(&calls_path).unwrap();
        let calls = calls.lines().collect::<Vec<_>>();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].contains(" --official-limits "));
        assert!(calls[0].contains(" --codex-sessions-dir "));
        assert!(calls[0].contains(" --no-incremental-cache"));

        write_tokenusage_provider_script(
            &bin_dir,
            r#"#!/usr/bin/env sh
printf '%s\n' "$*" >> "${0%/*}/calls"
case " $* " in
*" --official-limits "*)
  printf '%s\n' '{"official_codex":{"primary_used_percent":1.0,"primary_resets_at":605800,"primary_window_mins":10080}}'
  ;;
*)
exec sleep 10
  ;;
esac
"#,
        );
        let text = codex_usage_widget_text(CodexUsageWidgetOptions {
            cache_path: &cache_path,
            path_var: Some(bin_dir.as_os_str()),
            now_unix_seconds: 1_060,
            max_age_seconds: 60,
            error_backoff_seconds: 120,
            timeout: std::time::Duration::from_secs(1),
            display: AgentUsageDisplay::Quota,
            periods: &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
        })
        .unwrap();

        assert_eq!(text, " [codex 0h/7d|99%]");
        let cache = read_codex_usage_shared_cache_value(&cache_path).unwrap();
        assert_eq!(
            cache
                .pointer("/codex/five_hour_remaining_percent")
                .and_then(serde_json::Value::as_u64),
            None
        );
        assert_eq!(
            cache
                .pointer("/codex/weekly_remaining_percent")
                .and_then(serde_json::Value::as_u64),
            Some(99)
        );
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: a partial Codex refresh cannot multiply its timeout across probes or retry on every bar repaint.
    // Regression: one successful probe followed by a timeout used to keep partial facts retryable every 10 seconds.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn codex_usage_partial_refresh_is_bounded_and_backs_off() {
        let temp = temp_test_dir("codex_usage_partial_backoff");
        let bin_dir = temp.join("bin");
        let calls_path = bin_dir.join("calls");
        write_tokenusage_provider_script(
            &bin_dir,
            r#"#!/usr/bin/env sh
printf '%s|%s|%s\n' "$RAYON_NUM_THREADS" "$TOKIO_WORKER_THREADS" "$*" >> "${0%/*}/calls"
if [ "$1" = "blocks" ]; then
  printf '%s\n' '{"blocks":[{"isActive":true,"totals":{"total_tokens":138456789}}]}'
else
  exec sleep 10
fi
"#,
        );
        let cache_path = temp.join("agent_usage").join("codex_usage_cache_v4.json");
        write_json_value_atomic(
            &cache_path,
            &serde_json::json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 900u64,
                    "five_hour_tokens": 100u64,
                    "weekly_tokens": 1_337_000_000u64,
                    "weekly_remaining_percent": 80u64,
                    "weekly_reset_at_unix_seconds": 260_200u64,
                    "weekly_window_seconds": 604_800u64,
                    "status": "ok"
                }
            }),
        )
        .unwrap();
        let timeout = std::time::Duration::from_secs(1);
        let render = |now_unix_seconds| {
            codex_usage_widget_text(CodexUsageWidgetOptions {
                cache_path: &cache_path,
                path_var: Some(bin_dir.as_os_str()),
                now_unix_seconds,
                max_age_seconds: 0,
                error_backoff_seconds: 1_800,
                timeout,
                display: AgentUsageDisplay::Both,
                periods: &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
            })
            .unwrap()
        };
        let started = std::time::Instant::now();

        let text = render(1_000);

        assert!(started.elapsed() < std::time::Duration::from_secs(3));
        assert_eq!(text, " [codex~ 5h|138M 4d/7d|1.34B|80%]");
        let cache = read_codex_usage_shared_cache_value(&cache_path).unwrap();
        let entry = &cache["codex"];
        assert_eq!(entry["status"].as_str(), Some("partial"));
        assert_eq!(entry["backoff_until_unix_seconds"].as_u64(), Some(2_800));
        let calls = std::fs::read_to_string(&calls_path).unwrap();
        let first_calls = calls.lines().collect::<Vec<_>>();
        assert_eq!(first_calls.len(), 2);
        assert!(
            first_calls
                .iter()
                .all(|call| call.starts_with("1|1|") && call.contains(" --workers 1 "))
        );
        assert!(first_calls[0].contains("|blocks "));
        assert!(first_calls[1].contains("|weekly "));

        assert_eq!(render(1_010), text);
        assert_eq!(std::fs::read_to_string(&calls_path).unwrap(), calls);
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: Codex cache freshness and error backoff are enforced by nova-bar, not by Yazelix wrappers.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn codex_usage_shared_cache_respects_freshness_and_backoff() {
        let temp = temp_test_dir("codex_usage_backoff");
        let cache_path = temp.join("codex_usage_cache_v4.json");
        write_json_value_atomic(
            &cache_path,
            &serde_json::json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 138_456_789u64,
                    "weekly_tokens": 1_337_000_000u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64,
                    "five_hour_reset_at_unix_seconds": 8_200u64,
                    "weekly_reset_at_unix_seconds": 260_200u64,
                    "five_hour_window_seconds": 18_000u64,
                    "weekly_window_seconds": 604_800u64,
                    "status": "ok"
                }
            }),
        )
        .unwrap();

        assert!(codex_usage_shared_cache_is_fresh(
            &cache_path,
            1_100,
            600,
            AgentUsageDisplay::Both
        ));
        assert!(!codex_usage_shared_cache_is_fresh(
            &cache_path,
            1_700,
            600,
            AgentUsageDisplay::Both
        ));

        write_json_value_atomic(
            &cache_path,
            &serde_json::json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 1_700u64,
                    "error": "missing tu",
                    "backoff_until_unix_seconds": 2_000u64,
                    "status": "error"
                }
            }),
        )
        .unwrap();

        assert!(codex_usage_shared_cache_is_backing_off(&cache_path, 1_999));
        assert!(!codex_usage_shared_cache_is_backing_off(&cache_path, 2_000));
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: corrected duration-based Codex facts do not share a cache namespace with legacy position-based writers in open sessions.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn agent_usage_cache_paths_derive_from_status_cache_path() {
        let status_cache =
            std::path::Path::new("/state/sessions/window_a/status/status_bar_cache.json");

        assert_eq!(
            agent_usage_cache_path_from_status_cache_path(
                status_cache,
                "codex_usage_cache",
                CODEX_USAGE_CACHE_SCHEMA_VERSION,
            ),
            Some(std::path::PathBuf::from(
                "/state/sessions/agent_usage/codex_usage_cache_v4.json"
            ))
        );
    }

    // Defends: the standalone Claude command owns tokenusage probing, cache writes, and rendering without Yazelix runtime paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn claude_usage_widget_refreshes_shared_cache_from_tokenusage_provider() {
        let temp = temp_test_dir("claude_usage_refresh");
        let bin_dir = temp.join("bin");
        write_tokenusage_provider_script(
            &bin_dir,
            r#"#!/usr/bin/env sh
case " $* " in
*" --official-limits "*)
  printf '%s\n' '{"official_claude":{"primary_used_percent":51.0,"secondary_used_percent":20.0}}'
  ;;
*)
if [ "$1" = "blocks" ]; then
  printf '%s\n' '{"blocks":[{"isActive":true,"totals":{"total_tokens":15456373}}]}'
elif [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":66610005}}]}'
else
  exit 1
fi
  ;;
esac
"#,
        );
        let cache_path = temp.join("agent_usage").join("claude_usage_cache_v1.json");

        let text = claude_usage_widget_text(ClaudeUsageWidgetOptions {
            cache_path: &cache_path,
            path_var: Some(bin_dir.as_os_str()),
            now_unix_seconds: 1_000,
            max_age_seconds: 600,
            error_backoff_seconds: 1_800,
            timeout: std::time::Duration::from_secs(5),
            display: AgentUsageDisplay::Both,
            periods: &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
        })
        .unwrap();

        assert_eq!(text, " [claude 5h|15.5M|49% wk|66.6M|80%]");
        assert_eq!(
            read_claude_usage_shared_cache_value(&cache_path)
                .unwrap()
                .pointer("/claude/status")
                .and_then(serde_json::Value::as_str),
            Some("ok")
        );
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: Claude cache freshness and error backoff are enforced by nova-bar, not by Yazelix wrappers.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn claude_usage_shared_cache_respects_freshness_and_backoff() {
        let temp = temp_test_dir("claude_usage_backoff");
        let cache_path = temp.join("claude_usage_cache_v1.json");
        write_json_value_atomic(
            &cache_path,
            &serde_json::json!({
                "schema_version": CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
                "claude": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 15_456_373u64,
                    "weekly_tokens": 66_610_005u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64,
                    "status": "ok"
                }
            }),
        )
        .unwrap();

        assert!(claude_usage_shared_cache_is_fresh(&cache_path, 1_100, 600));
        assert!(!claude_usage_shared_cache_is_fresh(&cache_path, 1_700, 600));

        write_json_value_atomic(
            &cache_path,
            &serde_json::json!({
                "schema_version": CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
                "claude": {
                    "updated_at_unix_seconds": 1_700u64,
                    "error": "missing tu",
                    "backoff_until_unix_seconds": 2_000u64,
                    "status": "error"
                }
            }),
        )
        .unwrap();

        assert!(claude_usage_shared_cache_is_backing_off(&cache_path, 1_999));
        assert!(!claude_usage_shared_cache_is_backing_off(
            &cache_path,
            2_000
        ));
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: generic provider usage rendering covers Claude-style configured windows without Yazelix cache paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_windowed_agent_usage_widget_for_claude_facts() {
        let facts = WindowedAgentUsageFacts {
            five_hour_tokens: Some(15_456_373),
            weekly_tokens: Some(66_610_005),
            five_hour_remaining_percent: Some(49),
            weekly_remaining_percent: Some(80),
            ..WindowedAgentUsageFacts::default()
        };
        let periods = [AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly];

        assert_eq!(
            render_windowed_agent_usage_status_widget(
                "claude",
                &facts,
                &periods,
                AgentUsageDisplay::Both
            ),
            " [claude 5h|15.5M|49% wk|66.6M|80%]"
        );
        assert_eq!(
            render_windowed_agent_usage_status_widget(
                "claude",
                &facts,
                &periods,
                AgentUsageDisplay::Token
            ),
            " [claude 5h|15.5M wk|66.6M]"
        );
        assert_eq!(
            render_windowed_agent_usage_status_widget(
                "claude",
                &facts,
                &periods,
                AgentUsageDisplay::Quota
            ),
            " [claude 5h|49% wk|80%]"
        );
    }

    // Defends: OpenCode Go standalone rendering supports configured 5h/week/month windows and missing-cache hiding.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_windowed_agent_usage_widget_for_opencode_go_facts() {
        let facts = WindowedAgentUsageFacts {
            five_hour_tokens: Some(138_424_632),
            weekly_tokens: Some(1_335_519_960),
            monthly_tokens: Some(2_220_000_000),
            five_hour_remaining_percent: Some(49),
            weekly_remaining_percent: Some(80),
            monthly_remaining_percent: Some(70),
            ..WindowedAgentUsageFacts::default()
        };
        let periods = [
            AgentUsagePeriod::FiveHour,
            AgentUsagePeriod::Weekly,
            AgentUsagePeriod::Monthly,
        ];

        assert_eq!(
            render_windowed_agent_usage_status_widget(
                "go",
                &facts,
                &periods,
                AgentUsageDisplay::Both
            ),
            " [go 5h|138M|49% wk|1.34B|80% mo|2.22B|70%]"
        );
        assert_eq!(
            render_windowed_agent_usage_status_widget(
                "go",
                &facts,
                &periods,
                AgentUsageDisplay::Token
            ),
            " [go 5h|138M wk|1.34B mo|2.22B]"
        );
        assert_eq!(
            render_windowed_agent_usage_status_widget(
                "go",
                &WindowedAgentUsageFacts::default(),
                &periods,
                AgentUsageDisplay::Both
            ),
            ""
        );
    }

    // Defends: the standalone OpenCode Go command owns DB probing, cost-limit windows, cache writes, and rendering without Yazelix runtime paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn opencode_go_usage_widget_refreshes_shared_cache_from_db() {
        let temp = temp_test_dir("opencode_go_usage_refresh");
        let now = 10_000_000;
        let db_path = temp.join("opencode.db");
        write_opencode_go_test_db(&db_path, now);
        let cache_path = temp
            .join("agent_usage")
            .join("opencode_go_usage_cache_v1.json");

        let text = opencode_go_usage_widget_text(OpenCodeGoUsageWidgetOptions {
            cache_path: &cache_path,
            db_paths: &[db_path],
            now_unix_seconds: now,
            max_age_seconds: 600,
            error_backoff_seconds: 1_800,
            display: AgentUsageDisplay::Both,
            periods: &[
                AgentUsagePeriod::FiveHour,
                AgentUsagePeriod::Weekly,
                AgentUsagePeriod::Monthly,
            ],
        })
        .unwrap();

        assert_eq!(text, " [go 5h|1k|75% wk|3k|70% mo|33k|35%]");
        assert_eq!(
            read_opencode_go_usage_shared_cache_value(&cache_path)
                .unwrap()
                .pointer("/opencode_go/status")
                .and_then(serde_json::Value::as_str),
            Some("ok")
        );
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: OpenCode Go cache freshness and error backoff are enforced by nova-bar, not by Yazelix wrappers.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn opencode_go_usage_shared_cache_respects_freshness_and_backoff() {
        let temp = temp_test_dir("opencode_go_usage_backoff");
        let cache_path = temp.join("opencode_go_usage_cache_v1.json");
        write_json_value_atomic(
            &cache_path,
            &serde_json::json!({
                "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
                "opencode_go": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 1_000u64,
                    "weekly_tokens": 3_000u64,
                    "monthly_tokens": 33_000u64,
                    "five_hour_remaining_percent": 75u64,
                    "weekly_remaining_percent": 70u64,
                    "monthly_remaining_percent": 35u64,
                    "status": "ok"
                }
            }),
        )
        .unwrap();

        assert!(opencode_go_usage_shared_cache_is_fresh(
            &cache_path,
            1_100,
            600
        ));
        assert!(!opencode_go_usage_shared_cache_is_fresh(
            &cache_path,
            1_700,
            600
        ));

        write_json_value_atomic(
            &cache_path,
            &serde_json::json!({
                "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
                "opencode_go": {
                    "updated_at_unix_seconds": 1_700u64,
                    "error": "missing OpenCode DB",
                    "backoff_until_unix_seconds": 2_000u64,
                    "status": "error"
                }
            }),
        )
        .unwrap();

        assert!(opencode_go_usage_shared_cache_is_backing_off(
            &cache_path,
            1_999
        ));
        assert!(!opencode_go_usage_shared_cache_is_backing_off(
            &cache_path,
            2_000
        ));
        let _ = std::fs::remove_dir_all(temp);
    }

    // Regression: unsupported widget names must fail fast instead of leaving broken zjstatus placeholders.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rejects_unknown_widget_tray_entries() {
        let error =
            render_widget_tray_segment(&render_request(&["editor", "weather"])).unwrap_err();

        assert_eq!(
            error,
            BarRenderError::InvalidWidgetTrayEntry {
                entry: "weather".to_string()
            }
        );
        assert_eq!(error.code(), "invalid_widget_tray_entry");
    }

    // Defends: full tab labels keep the existing index plus name format unless compact mode is explicitly enabled.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_full_tab_label_formats_by_default_contract() {
        let formats = render_zjstatus_tab_label_formats(TAB_LABEL_MODE_FULL).unwrap();

        assert!(formats.tab_normal.contains("[{index}] {name}"));
        assert!(!formats.tab_normal.contains("{bell_indicator}"));
        assert!(formats.tab_normal_bell.contains("[{index}] {name}"));
        assert!(!formats.tab_normal_bell.contains("{bell_indicator}"));
        assert!(
            !formats
                .tab_normal_flashing_bell
                .contains("{bell_indicator}")
        );
        assert!(formats.tab_active.contains("[{index}] {name}"));
        assert!(formats.tab_rename.contains("{index} {name}"));
    }

    // Defends: compact tab labels remove tab names from normal rendering while preserving index and state indicators.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_compact_tab_label_formats_without_names() {
        let formats = render_zjstatus_tab_label_formats(TAB_LABEL_MODE_COMPACT).unwrap();

        assert_eq!(
            formats.tab_normal,
            r##"tab_normal   "#[fg=#ffff00] [{index}] ""##
        );
        assert!(
            formats
                .tab_normal_bell
                .contains("[{index}] {sync_indicator}")
        );
        assert!(!formats.tab_normal_bell.contains("{bell_indicator}"));
        assert!(!formats.tab_normal_bell.contains("{name}"));
        assert!(formats.tab_normal_fullscreen.contains("{index}] []"));
        assert!(formats.tab_active_sync.contains("{sync_indicator}"));
        assert!(!formats.tab_active.contains("{name}"));
        assert!(formats.tab_rename.contains("{name}"));
    }

    // Regression: unsupported tab label modes fail fast instead of emitting broken zjstatus KDL.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rejects_unknown_tab_label_mode() {
        let error = render_zjstatus_tab_label_formats("tiny").unwrap_err();

        assert_eq!(
            error,
            BarRenderError::InvalidTabLabelMode {
                mode: "tiny".to_string()
            }
        );
        assert_eq!(error.code(), "invalid_tab_label_mode");
    }
}
