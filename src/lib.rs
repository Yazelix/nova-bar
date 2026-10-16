mod agent_usage;
mod appearance;

pub use agent_usage::*;

pub use appearance::{
    APPEARANCE_MODE_AUTO, APPEARANCE_MODE_DARK, APPEARANCE_MODE_LIGHT, default_appearance_mode,
    normalize_appearance_arg,
};
use appearance::{BarStyle, DARK_BAR_STYLE, LIGHT_BAR_STYLE, runtime_bar_appearance};
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
const RUNTIME_NU_BIN_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_NU_BIN__";
const RUNTIME_YZX_CONTROL_BIN_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_YZX_CONTROL_BIN__";
const RUNTIME_WIDGET_BIN_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_WIDGET_BIN__";
const RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";
const RUNTIME_CLAUDE_USAGE_PERIODS_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_CLAUDE_USAGE_PERIODS__";
const RUNTIME_CLAUDE_USAGE_WIDGET_ARGS_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_CLAUDE_USAGE_WIDGET_ARGS__";
const RUNTIME_CODEX_USAGE_PERIODS_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_CODEX_USAGE_PERIODS__";
const RUNTIME_CODEX_USAGE_WIDGET_ARGS_PLACEHOLDER: &str =
    "__YAZELIX_RUNTIME_CODEX_USAGE_WIDGET_ARGS__";
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
    #[serde(default = "default_claude_usage_periods")]
    pub claude_usage_periods: Vec<String>,
    #[serde(default = "default_codex_usage_periods")]
    pub codex_usage_periods: Vec<String>,
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
            "notification_format_unread",
            format!("{} {{message}} ", style.notification_unread),
        ),
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
            RUNTIME_CLAUDE_USAGE_PERIODS_PLACEHOLDER,
            escape_kdl_string(&agent_usage_periods_arg(&config.claude_usage_periods)),
        ),
        (RUNTIME_CLAUDE_USAGE_WIDGET_ARGS_PLACEHOLDER, widget_args()),
        (
            RUNTIME_CODEX_USAGE_PERIODS_PLACEHOLDER,
            escape_kdl_string(&agent_usage_periods_arg(&config.codex_usage_periods)),
        ),
        (RUNTIME_CODEX_USAGE_WIDGET_ARGS_PLACEHOLDER, widget_args()),
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
            widget_placeholder(&render_agent_usage_status_body("claude", "…")),
        ),
        (
            RUNTIME_COMMAND_CODEX_USAGE_FORMAT_PLACEHOLDER,
            "{stdout}".to_string(),
        ),
        (
            RUNTIME_COMMAND_CODEX_USAGE_PLACEHOLDER,
            widget_placeholder(&render_agent_usage_status_body("codex", "…")),
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
            r##"tab_normal   "{} [{{index}}{}] ""##,
            style.tab_normal, name
        ),
        tab_normal_fullscreen: format!(
            r##"tab_normal_fullscreen "{} [{{index}}{}] [] ""##,
            style.tab_normal, name
        ),
        tab_normal_sync: format!(
            r##"tab_normal_sync       "{} [{{index}}{}] <> ""##,
            style.tab_normal, name
        ),
        tab_normal_bell: format!(
            r##"tab_normal_bell "{} [{{index}}{}] {{sync_indicator}}{{fullscreen_indicator}}""##,
            style.tab_bell, name
        ),
        tab_normal_flashing_bell: format!(
            r##"tab_normal_flashing_bell "{} [{{index}}{}] {{sync_indicator}}{{fullscreen_indicator}}""##,
            style.tab_flashing_bell, name
        ),
        tab_active: format!(
            r##"tab_active   "{} [{{index}}{}] ""##,
            style.tab_active, name
        ),
        tab_active_fullscreen: format!(
            r##"tab_active_fullscreen "{} [{{index}}{}] {{fullscreen_indicator}}""##,
            style.tab_active, name
        ),
        tab_active_sync: format!(
            r##"tab_active_sync       "{} [{{index}}{}] {{sync_indicator}}""##,
            style.tab_active, name
        ),
        tab_rename: format!(
            r##"tab_rename    "{} [{{index}} {{name}}] ""##,
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
        WIDGET_CLAUDE_USAGE => Ok(command(style.claude_usage, COMMAND_CLAUDE_USAGE)),
        WIDGET_CODEX_USAGE => Ok(command(style.codex_usage, COMMAND_CODEX_USAGE)),
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

    // Defends: the bar registry preserves the existing zjstatus widgets and exact segment syntax.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_existing_widget_tray_segments() {
        let rendered = render_widget_tray_segment(&render_request(DEFAULT_WIDGET_TRAY)).unwrap();

        assert_eq!(
            rendered,
            " #[fg=#ff0088,bold]{session} #[fg=#6c7086,bold]• #[fg=#00ff88,bold] hx #[fg=#6c7086,bold]• #[fg=#00ff88,bold]❯nu #[fg=#6c7086,bold]• #[fg=#00ff88,bold]{command_term} #[fg=#6c7086,bold]• #[fg=#ffffff]{command_codex_usage} #[fg=#6c7086,bold]• #[fg=#ff6600]{command_cpu} #[fg=#6c7086,bold]• #[fg=#ff6600]{command_ram}"
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
            " #[fg=#d97757]{command_claude_usage} #[fg=#6c7086,bold]• #[fg=#ffffff]{command_codex_usage} #[fg=#6c7086,bold]• #[fg=#bb88ff,bold]{command_opencode_go_usage}"
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
            claude_usage_periods: vec!["5h".to_string(), "week".to_string()],
            codex_usage_periods: vec!["5h".to_string(), "week".to_string()],
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
            "#[fg=#a0a6af] [{index}] "
        );
        assert!(rendered.contains(
            r##"tab_normal_bell "#[fg=#ff0088,bold] [{index}] {sync_indicator}{fullscreen_indicator}""##
        ));
        assert!(rendered.contains(r##"tab_bell_indicator       """##));
        assert!(rendered.contains(r##"pipe_workspace_format "#[fg=#00ff88,bold]{output}""##));
        assert!(rendered.contains(
            r##"notification_format_unread "#[bg=#ff6600,fg=#000000,bold] {message} ""##
        ));
        assert!(rendered.contains(
            r##"host_theme_light_notification_format_unread "#[bg=#ccd0da,fg=#303446,bold] {message} ""##
        ));
        assert!(rendered.contains(r#"notification_show_interval "5""#));
        assert!(rendered.contains(r#"notification_toast_min_width "15""#));
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
            r#"command_codex_usage_command "/runtime/bin/nova_bar_widget codex --periods 5h,week --widget-frame none --widget-separator empty --widget-first false""#
        ));
        assert!(
            rendered
                .contains(r#"--widget-frame none --widget-separator empty --widget-first false"#)
        );
        assert!(rendered.contains(r##"command_codex_usage_format "{stdout}""##));
        assert!(rendered.contains(r##"command_codex_usage_rendermode "raw""##));
        assert_eq!(
            runtime_assignment(&rendered, "command_codex_usage_placeholder"),
            "\u{ec81} …"
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
            "[\u{ec81} …]"
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
        assert!(rendered.contains(r##"host_theme_dark_tab_normal "#[fg=#a0a6af] [{index}] ""##));
        assert_eq!(
            runtime_assignment(&rendered, "format_right"),
            "#[fg=#7c3f97,bold]{session}{segment}#[fg=#2f7d32,bold] hx{segment}{pipe_workspace}{segment}#[fg=#a24f00]{command_cpu}{segment}#[fg=#9a5a00,bold][demo]{segment}#[fg=#1e66f5,bold]{command_version}"
        );
        assert!(!rendered.contains("mode_normal"));
        assert_eq!(
            runtime_assignment(&rendered, "tab_active"),
            "#[fg=#9a5a00,bold] [{index}] "
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

    // Full labels keep the same outline when focus changes; native indicators remain separate.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_full_tab_label_formats_by_default_contract() {
        let formats = render_zjstatus_tab_label_formats(TAB_LABEL_MODE_FULL).unwrap();

        assert!(!formats.tab_normal.contains("{bell_indicator}"));
        assert!(!formats.tab_normal_bell.contains("{bell_indicator}"));
        assert!(
            !formats
                .tab_normal_flashing_bell
                .contains("{bell_indicator}")
        );
        for style in [&DARK_BAR_STYLE, &LIGHT_BAR_STYLE] {
            let formats = render_tab_label_formats(style, true);
            for label in [
                &formats.tab_normal,
                &formats.tab_normal_fullscreen,
                &formats.tab_normal_sync,
                &formats.tab_normal_bell,
                &formats.tab_normal_flashing_bell,
            ] {
                assert!(label.contains("[{index} {name}]"), "{label}");
            }
            for label in [
                &formats.tab_active,
                &formats.tab_active_fullscreen,
                &formats.tab_active_sync,
                &formats.tab_rename,
            ] {
                assert!(label.contains("[{index} {name}]"), "{label}");
                assert!(!label.contains("bg="), "{label}");
                assert!(label.contains("bold"), "{label}");
                assert!(!label.contains("{floating_indicator}"), "{label}");
            }
            assert!(
                formats
                    .tab_active_fullscreen
                    .contains("] {fullscreen_indicator}")
            );
            assert!(formats.tab_active_sync.contains("] {sync_indicator}"));
        }
    }

    // Defends: compact tab labels remove tab names from normal rendering while preserving index and state indicators.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_compact_tab_label_formats_without_names() {
        let formats = render_zjstatus_tab_label_formats(TAB_LABEL_MODE_COMPACT).unwrap();

        assert_eq!(
            formats.tab_normal,
            r##"tab_normal   "#[fg=#a0a6af] [{index}] ""##
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
        assert!(!formats.tab_active.contains("{floating_indicator}"));
        assert!(!formats.tab_rename.contains("{floating_indicator}"));
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
