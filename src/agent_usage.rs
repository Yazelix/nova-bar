//! Provider quota widgets: CodexBar-backed Codex and Claude windows, local OpenCode Go
//! limits, their shared caches, and Eon's quota text.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentUsagePeriod {
    FiveHour,
    Weekly,
    Monthly,
}

impl AgentUsagePeriod {
    /// Window label used when the provider reports no duration.
    pub fn duration_label(self) -> &'static str {
        match self {
            Self::FiveHour => "5h",
            Self::Weekly => "7d",
            Self::Monthly => "30d",
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
    fn is_empty(&self) -> bool {
        self.five_hour_remaining_percent.is_none()
            && self.weekly_remaining_percent.is_none()
            && self.monthly_remaining_percent.is_none()
    }
}

/// A provider whose quota windows come from the CodexBar CLI, with Eon's sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaProvider {
    Codex,
    Claude,
}

impl QuotaProvider {
    pub fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub fn cache_schema_version(self) -> i64 {
        match self {
            Self::Codex => CODEX_USAGE_CACHE_SCHEMA_VERSION,
            Self::Claude => CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
        }
    }

    pub fn cache_file_name(self) -> String {
        format!(
            "{}_usage_cache_v{}.json",
            self.name(),
            self.cache_schema_version()
        )
    }

    /// Minimum seconds between reads. Claude's OAuth usage endpoint rate-limits frequent
    /// readers, so Claude reads every five minutes, aligned with Eon.
    pub fn default_max_age_seconds(self) -> u64 {
        match self {
            Self::Codex => 60,
            Self::Claude => 300,
        }
    }

    /// Seconds to wait after a failed read.
    pub fn default_error_backoff_seconds(self) -> u64 {
        match self {
            Self::Codex => 120,
            Self::Claude => 300,
        }
    }

    fn source(self) -> &'static str {
        match self {
            Self::Codex => "cli",
            Self::Claude => "oauth",
        }
    }

    fn reported_source(self) -> &'static str {
        match self {
            Self::Codex => "codex-cli",
            Self::Claude => "oauth",
        }
    }
}

pub const CODEX_USAGE_CACHE_SCHEMA_VERSION: i64 = 5;
pub const CLAUDE_USAGE_CACHE_SCHEMA_VERSION: i64 = 2;
pub const OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION: i64 = 2;
pub const AGENT_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub const OPENCODE_GO_PROVIDER_ID: &str = "opencode-go";
pub const OPENCODE_GO_FIVE_HOUR_SECONDS: u64 = 5 * 60 * 60;
pub const OPENCODE_GO_WEEK_SECONDS: u64 = 7 * 24 * 60 * 60;
pub const OPENCODE_GO_MONTH_SECONDS: u64 = 30 * 24 * 60 * 60;
pub const OPENCODE_GO_FIVE_HOUR_LIMIT_USD: f64 = 12.0;
pub const OPENCODE_GO_WEEKLY_LIMIT_USD: f64 = 30.0;
pub const OPENCODE_GO_MONTHLY_LIMIT_USD: f64 = 60.0;
const FIVE_HOUR_WINDOW_SECONDS: u64 = 5 * 60 * 60;
const WEEKLY_WINDOW_SECONDS: u64 = 7 * 24 * 60 * 60;
const CODEXBAR_MAX_OUTPUT_BYTES: usize = 64 * 1024;
/// CodexBar readings older than this are rejected, as in Eon.
const CODEXBAR_STALE_AFTER_SECONDS: u64 = 60 * 60;
const OPENCODE_GO_CACHE_KEY: &str = "opencode_go";

#[derive(Debug, Clone, Copy)]
pub struct QuotaWidgetOptions<'a> {
    pub cache_path: &'a std::path::Path,
    pub path_var: Option<&'a std::ffi::OsStr>,
    pub now_unix_seconds: u64,
    pub max_age_seconds: u64,
    pub error_backoff_seconds: u64,
    pub timeout: std::time::Duration,
    pub periods: &'a [AgentUsagePeriod],
}

#[derive(Debug, Clone, Copy)]
pub struct OpenCodeGoUsageWidgetOptions<'a> {
    pub cache_path: &'a std::path::Path,
    pub db_paths: &'a [std::path::PathBuf],
    pub now_unix_seconds: u64,
    pub max_age_seconds: u64,
    pub error_backoff_seconds: u64,
    pub periods: &'a [AgentUsagePeriod],
}

/// Refreshes the provider cache when due and renders its body, such as `<mark> 3h/5h 74% · 6d4h/7d 42%`.
pub fn quota_usage_widget_body_text(
    provider: QuotaProvider,
    options: QuotaWidgetOptions<'_>,
) -> Result<String, String> {
    refresh_quota_cache(provider, options)?;
    Ok(read_cached_facts(
        options.cache_path,
        provider.name(),
        provider.cache_schema_version(),
    )
    .map(|facts| {
        render_windowed_agent_usage_status_body(
            provider.name(),
            &current_windows(facts, options.now_unix_seconds),
            options.periods,
        )
    })
    .unwrap_or_default())
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
    Ok(read_cached_facts(
        options.cache_path,
        OPENCODE_GO_CACHE_KEY,
        OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
    )
    .map(|facts| render_windowed_agent_usage_status_body("go", &facts, options.periods))
    .unwrap_or_default())
}

/// Standalone presets frame a usage body as ` [body]`; empty bodies stay empty.
pub fn render_standalone_usage_widget(body: &str) -> String {
    if body.is_empty() {
        String::new()
    } else {
        format!(" [{body}]")
    }
}

pub fn quota_usage_cache_path_from_env(provider: QuotaProvider) -> Option<std::path::PathBuf> {
    status_bar_cache_path_from_env().and_then(|path| {
        agent_usage_cache_path_from_status_cache_path(
            &path,
            &format!("{}_usage_cache", provider.name()),
            provider.cache_schema_version(),
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

pub(crate) fn status_bar_cache_path_from_env() -> Option<std::path::PathBuf> {
    std::env::var_os("YAZELIX_STATUS_BAR_CACHE_PATH")
        .map(std::path::PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

pub fn render_windowed_agent_usage_status_widget(
    label: &str,
    facts: &WindowedAgentUsageFacts,
    periods: &[AgentUsagePeriod],
) -> String {
    render_standalone_usage_widget(&render_windowed_agent_usage_status_body(
        label, facts, periods,
    ))
}

pub fn render_windowed_agent_usage_status_body(
    label: &str,
    facts: &WindowedAgentUsageFacts,
    periods: &[AgentUsagePeriod],
) -> String {
    render_agent_usage_status_body(label, &render_windowed_agent_usage_summary(facts, periods))
}

/// Leads a usage summary with its provider mark: OpenAI's blossom, Claude's splat, or the label itself.
pub(crate) fn render_agent_usage_status_body(label: &str, summary: &str) -> String {
    if summary.is_empty() {
        return String::new();
    }
    let label = match label {
        "codex" => "\u{ec81}",
        "claude" => "\u{ec82}",
        _ => label,
    };
    format!("{label} {summary}")
}

/// Eon's quota text: `<elapsed>/<total> <remaining>%` per window, joined with ` · `.
pub fn render_windowed_agent_usage_summary(
    facts: &WindowedAgentUsageFacts,
    periods: &[AgentUsagePeriod],
) -> String {
    periods
        .iter()
        .filter_map(|period| {
            let (remaining_percent, reset_at, window_seconds) = period_values(facts, *period);
            let window_minutes = window_seconds
                .map(|seconds| seconds / 60)
                .filter(|m| *m > 0);
            let duration = window_minutes
                .map(quota_duration)
                .unwrap_or_else(|| period.duration_label().to_string());
            let label = match (facts.updated_at_unix_seconds, reset_at, window_minutes) {
                (Some(observed_at), Some(reset_at), Some(minutes)) => format!(
                    "{}/{duration}",
                    quota_window_position(minutes, reset_at, observed_at)
                ),
                _ => duration,
            };
            Some(format!("{label} {}%", remaining_percent?.min(100)))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn period_values(
    facts: &WindowedAgentUsageFacts,
    period: AgentUsagePeriod,
) -> (Option<u64>, Option<u64>, Option<u64>) {
    match period {
        AgentUsagePeriod::FiveHour => (
            facts.five_hour_remaining_percent,
            facts.five_hour_reset_at_unix_seconds,
            facts.five_hour_window_seconds,
        ),
        AgentUsagePeriod::Weekly => (
            facts.weekly_remaining_percent,
            facts.weekly_reset_at_unix_seconds,
            facts.weekly_window_seconds,
        ),
        AgentUsagePeriod::Monthly => (facts.monthly_remaining_percent, None, None),
    }
}

// Mirrors Eon Venus `quota_duration`.
fn quota_duration(minutes: u64) -> String {
    if minutes.is_multiple_of(24 * 60) {
        format!("{}d", minutes / (24 * 60))
    } else if minutes.is_multiple_of(60) {
        format!("{}h", minutes / 60)
    } else {
        format!("{minutes}m")
    }
}

// Mirrors Eon Venus `quota_window_position`: how far into the window the observation is.
fn quota_window_position(window_minutes: u64, reset_at: u64, observed_at: u64) -> String {
    let total_seconds = window_minutes * 60;
    let remaining_seconds = reset_at.saturating_sub(observed_at).min(total_seconds);
    let elapsed_minutes = (total_seconds - remaining_seconds) / 60;
    if window_minutes >= 24 * 60 {
        match (
            elapsed_minutes / (24 * 60),
            elapsed_minutes % (24 * 60) / 60,
        ) {
            (0, 0) => "0h".into(),
            (0, hours) => format!("{hours}h"),
            (days, 0) => format!("{days}d"),
            (days, hours) => format!("{days}d{hours}h"),
        }
    } else if window_minutes >= 60 {
        match (elapsed_minutes / 60, elapsed_minutes % 60) {
            (0, minutes) => format!("{minutes}m"),
            (hours, 0) => format!("{hours}h"),
            (hours, minutes) => format!("{hours}h{minutes}m"),
        }
    } else {
        format!("{elapsed_minutes}m")
    }
}

/// Hides windows whose reset has passed since the reading.
fn current_windows(mut facts: WindowedAgentUsageFacts, now: u64) -> WindowedAgentUsageFacts {
    if facts
        .five_hour_reset_at_unix_seconds
        .is_some_and(|reset| reset <= now)
    {
        facts.five_hour_remaining_percent = None;
    }
    if facts
        .weekly_reset_at_unix_seconds
        .is_some_and(|reset| reset <= now)
    {
        facts.weekly_remaining_percent = None;
    }
    facts
}

pub fn refresh_quota_cache(
    provider: QuotaProvider,
    options: QuotaWidgetOptions<'_>,
) -> Result<bool, String> {
    let (key, schema) = (provider.name(), provider.cache_schema_version());
    let now = options.now_unix_seconds;
    let due = || {
        cache_refresh_is_due(
            options.cache_path,
            key,
            schema,
            now,
            options.max_age_seconds,
        )
    };
    if !due() {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_agent_usage_cache_lock(options.cache_path, key, schema, now)?
    else {
        return Ok(false);
    };
    if !due() {
        return Ok(false);
    }
    let facts = collect_codexbar_quota(provider, options.path_var, now, options.timeout)
        .unwrap_or_else(|error| {
            retained_windows(
                read_cached_facts(options.cache_path, key, schema),
                now,
                error,
            )
        });
    write_cached_facts(
        options.cache_path,
        key,
        schema,
        &facts,
        now,
        options.error_backoff_seconds,
    )?;
    Ok(true)
}

/// A failed read keeps the last windows that have not reset yet.
fn retained_windows(
    previous: Option<WindowedAgentUsageFacts>,
    now: u64,
    error: String,
) -> WindowedAgentUsageFacts {
    let mut facts = WindowedAgentUsageFacts {
        updated_at_unix_seconds: Some(now),
        error: Some(error),
        ..WindowedAgentUsageFacts::default()
    };
    let Some(previous) = previous else {
        return facts;
    };
    if previous
        .five_hour_reset_at_unix_seconds
        .is_some_and(|reset| reset > now)
    {
        facts.five_hour_remaining_percent = previous.five_hour_remaining_percent;
        facts.five_hour_reset_at_unix_seconds = previous.five_hour_reset_at_unix_seconds;
        facts.five_hour_window_seconds = previous.five_hour_window_seconds;
    }
    if previous
        .weekly_reset_at_unix_seconds
        .is_some_and(|reset| reset > now)
    {
        facts.weekly_remaining_percent = previous.weekly_remaining_percent;
        facts.weekly_reset_at_unix_seconds = previous.weekly_reset_at_unix_seconds;
        facts.weekly_window_seconds = previous.weekly_window_seconds;
    }
    if !facts.is_empty() {
        facts.updated_at_unix_seconds = previous.updated_at_unix_seconds;
    }
    facts
}

fn collect_codexbar_quota(
    provider: QuotaProvider,
    path_var: Option<&std::ffi::OsStr>,
    now: u64,
    timeout: std::time::Duration,
) -> Result<WindowedAgentUsageFacts, String> {
    let path_var = path_var.ok_or("missing PATH")?;
    let program = find_command_in_path_var(path_var, "codexbar").ok_or("missing codexbar")?;
    let mut command = std::process::Command::new(program);
    command
        .args(["usage", "--provider", provider.name(), "--source"])
        .arg(provider.source())
        .args(["--format", "json", "--json-only", "--web-timeout", "8"])
        .stdin(std::process::Stdio::null());
    let output = run_command_with_timeout(command, timeout)
        .map_err(|error| format!("failed to run codexbar: {error}"))?
        .ok_or("codexbar timed out")?;
    if output.stdout.len() > CODEXBAR_MAX_OUTPUT_BYTES {
        return Err("codexbar output exceeds 64 KiB".to_string());
    }
    codexbar_quota_from_json(&output.stdout, provider, now, timeout)
}

/// Validates one CodexBar `usage --format json` result as Eon's EON-C27 collector does.
/// `now` is sampled before the read, so a reading may be stamped up to the read deadline later.
fn codexbar_quota_from_json(
    bytes: &[u8],
    provider: QuotaProvider,
    now: u64,
    timeout: std::time::Duration,
) -> Result<WindowedAgentUsageFacts, String> {
    let invalid = || format!("invalid codexbar {} reading", provider.name());
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let [result] = value.as_array().map(Vec::as_slice).ok_or_else(invalid)? else {
        return Err(invalid());
    };
    if result["provider"] != provider.name() || result["source"] != provider.reported_source() {
        return Err(invalid());
    }
    if !result["error"].is_null() {
        return Err(format!("codexbar {} read failed", provider.name()));
    }
    let mut facts = WindowedAgentUsageFacts {
        updated_at_unix_seconds: Some(now),
        ..WindowedAgentUsageFacts::default()
    };
    let usage = &result["usage"];
    if usage.is_null() {
        return Ok(facts);
    }
    if !usage["dataConfidence"].is_null() && usage["dataConfidence"] != "authoritative" {
        return Err(invalid());
    }
    let observed_at = utc_timestamp(&usage["updatedAt"]).ok_or_else(invalid)?;
    let read_deadline = now.saturating_add(timeout.as_secs().max(1));
    if observed_at > read_deadline || now.saturating_sub(observed_at) > CODEXBAR_STALE_AFTER_SECONDS
    {
        return Err(invalid());
    }
    if !usage["tertiary"].is_null() {
        return Err(invalid());
    }
    facts.updated_at_unix_seconds = Some(observed_at);
    for name in ["primary", "secondary"] {
        let window = &usage[name];
        if window.is_null() {
            continue;
        }
        let used = window["usedPercent"]
            .as_f64()
            .filter(|used| (0.0..=100.0).contains(used))
            .ok_or_else(invalid)?;
        let window_seconds = window["windowMinutes"]
            .as_u64()
            .and_then(|minutes| minutes.checked_mul(60))
            .ok_or_else(invalid)?;
        let reset_at = if window["resetsAt"].is_null() {
            None
        } else {
            Some(utc_timestamp(&window["resetsAt"]).ok_or_else(invalid)?)
        };
        let (percent, reset, duration) = match window_seconds {
            FIVE_HOUR_WINDOW_SECONDS => (
                &mut facts.five_hour_remaining_percent,
                &mut facts.five_hour_reset_at_unix_seconds,
                &mut facts.five_hour_window_seconds,
            ),
            WEEKLY_WINDOW_SECONDS => (
                &mut facts.weekly_remaining_percent,
                &mut facts.weekly_reset_at_unix_seconds,
                &mut facts.weekly_window_seconds,
            ),
            _ => return Err(invalid()),
        };
        if duration.is_some() {
            return Err(invalid());
        }
        *duration = Some(window_seconds);
        if reset_at.is_some_and(|reset| reset <= now) {
            continue;
        }
        *percent = Some((100.0 - used).floor() as u64);
        *reset = reset_at;
    }
    Ok(facts)
}

/// Parses CodexBar's UTC `YYYY-MM-DDTHH:MM:SS[.fraction]Z` timestamps; other forms are rejected.
fn utc_timestamp(value: &serde_json::Value) -> Option<u64> {
    let (date, time) = value.as_str()?.strip_suffix('Z')?.split_once('T')?;
    let time = match time.split_once('.') {
        Some((whole, fraction))
            if !fraction.is_empty() && fraction.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            whole
        }
        Some(_) => return None,
        None => time,
    };
    let field = |text: &str, range: std::ops::Range<usize>, max: i64| {
        let part = text.get(range)?;
        part.bytes()
            .all(|byte| byte.is_ascii_digit())
            .then(|| part.parse::<i64>().ok())
            .flatten()
            .filter(|value| *value <= max)
    };
    if date.len() != 10 || &date[4..5] != "-" || &date[7..8] != "-" {
        return None;
    }
    if time.len() != 8 || &time[2..3] != ":" || &time[5..6] != ":" {
        return None;
    }
    let (year, month, day) = (
        field(date, 0..4, 9999)?,
        field(date, 5..7, 12)?,
        field(date, 8..10, 31)?,
    );
    let (hour, minute, second) = (
        field(time, 0..2, 23)?,
        field(time, 3..5, 59)?,
        field(time, 6..8, 59)?,
    );
    if month == 0 || day == 0 {
        return None;
    }
    // Days from 1970-01-01 in the proleptic Gregorian calendar (Howard Hinnant's algorithm).
    let shifted_year = if month <= 2 { year - 1 } else { year };
    let era = shifted_year.div_euclid(400);
    let year_of_era = shifted_year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    u64::try_from(days * 86_400 + hour * 3_600 + minute * 60 + second).ok()
}

#[cfg(feature = "providers")]
pub fn refresh_opencode_go_usage_shared_cache(
    shared_path: &std::path::Path,
    db_paths: &[std::path::PathBuf],
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
) -> Result<bool, String> {
    let (key, schema) = (
        OPENCODE_GO_CACHE_KEY,
        OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
    );
    let due = || cache_refresh_is_due(shared_path, key, schema, now, max_age_seconds);
    if !due() {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_agent_usage_cache_lock(shared_path, key, schema, now)? else {
        return Ok(false);
    };
    if !due() {
        return Ok(false);
    }
    let facts = collect_opencode_go_usage_facts_from_dbs(db_paths, now);
    write_cached_facts(shared_path, key, schema, &facts, now, error_backoff_seconds)?;
    Ok(true)
}

pub fn read_agent_usage_cache_value(
    path: &std::path::Path,
    schema_version: i64,
) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&raw).ok()?;
    (cache
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        == Some(schema_version))
    .then_some(cache)
}

fn read_cached_facts(
    path: &std::path::Path,
    key: &str,
    schema_version: i64,
) -> Option<WindowedAgentUsageFacts> {
    read_agent_usage_cache_value(path, schema_version)?
        .get(key)
        .map(windowed_usage_facts_from_cache_entry)
}

/// A cache is due after its fresh reading ages out or its failure backoff ends.
fn cache_refresh_is_due(
    path: &std::path::Path,
    key: &str,
    schema_version: i64,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    let Some(cache) = read_agent_usage_cache_value(path, schema_version) else {
        return true;
    };
    let Some(entry) = cache.get(key) else {
        return true;
    };
    let number = |name| entry.get(name).and_then(serde_json::Value::as_u64);
    if let Some(backoff_until) = number("backoff_until_unix_seconds") {
        return now >= backoff_until;
    }
    number("checked_at_unix_seconds")
        .is_none_or(|checked_at| now.saturating_sub(checked_at) >= max_age_seconds)
}

fn write_cached_facts(
    path: &std::path::Path,
    key: &str,
    schema_version: i64,
    facts: &WindowedAgentUsageFacts,
    now: u64,
    error_backoff_seconds: u64,
) -> Result<(), String> {
    let mut entry = serde_json::Map::new();
    let mut insert = |name: &str, value: Option<u64>| {
        if let Some(value) = value {
            entry.insert(name.to_string(), serde_json::json!(value));
        }
    };
    insert("checked_at_unix_seconds", Some(now));
    insert("updated_at_unix_seconds", facts.updated_at_unix_seconds);
    insert(
        "five_hour_remaining_percent",
        facts.five_hour_remaining_percent,
    );
    insert("weekly_remaining_percent", facts.weekly_remaining_percent);
    insert("monthly_remaining_percent", facts.monthly_remaining_percent);
    insert(
        "five_hour_reset_at_unix_seconds",
        facts.five_hour_reset_at_unix_seconds,
    );
    insert(
        "weekly_reset_at_unix_seconds",
        facts.weekly_reset_at_unix_seconds,
    );
    insert("five_hour_window_seconds", facts.five_hour_window_seconds);
    insert("weekly_window_seconds", facts.weekly_window_seconds);
    let status = match (&facts.error, facts.is_empty()) {
        (None, _) => "ok",
        (Some(_), false) => "partial",
        (Some(_), true) => "error",
    };
    if let Some(error) = &facts.error {
        insert(
            "backoff_until_unix_seconds",
            Some(now.saturating_add(error_backoff_seconds)),
        );
        entry.insert("error".to_string(), serde_json::json!(error));
    }
    entry.insert("status".to_string(), serde_json::json!(status));
    write_json_value_atomic(
        path,
        &serde_json::json!({
            "schema_version": schema_version,
            key: serde_json::Value::Object(entry),
        }),
    )
}

fn windowed_usage_facts_from_cache_entry(entry: &serde_json::Value) -> WindowedAgentUsageFacts {
    let number = |name| entry.get(name).and_then(serde_json::Value::as_u64);
    WindowedAgentUsageFacts {
        updated_at_unix_seconds: number("updated_at_unix_seconds"),
        five_hour_remaining_percent: number("five_hour_remaining_percent"),
        weekly_remaining_percent: number("weekly_remaining_percent"),
        monthly_remaining_percent: number("monthly_remaining_percent"),
        five_hour_reset_at_unix_seconds: number("five_hour_reset_at_unix_seconds"),
        weekly_reset_at_unix_seconds: number("weekly_reset_at_unix_seconds"),
        five_hour_window_seconds: number("five_hour_window_seconds"),
        weekly_window_seconds: number("weekly_window_seconds"),
        error: entry
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

struct AgentUsageCacheLock {
    path: std::path::PathBuf,
}

impl Drop for AgentUsageCacheLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.path);
    }
}

fn try_acquire_agent_usage_cache_lock(
    shared_path: &std::path::Path,
    key: &str,
    schema_version: i64,
    now: u64,
) -> Result<Option<AgentUsageCacheLock>, String> {
    let lock_path =
        shared_path.with_file_name(format!(".{key}_usage_cache_v{schema_version}.lock"));
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create usage cache lock parent: {error}"))?;
    }
    match std::fs::create_dir(&lock_path) {
        Ok(()) => Ok(Some(AgentUsageCacheLock { path: lock_path })),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if agent_usage_cache_lock_is_stale(&lock_path, now) {
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

fn agent_usage_cache_lock_is_stale(lock_path: &std::path::Path, now: u64) -> bool {
    std::fs::metadata(lock_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| {
            now.saturating_sub(duration.as_secs()) > AGENT_USAGE_LOCK_STALE_AFTER_SECONDS
        })
        .unwrap_or(false)
}

#[cfg(feature = "providers")]
pub fn collect_opencode_go_usage_facts_from_dbs(
    db_paths: &[std::path::PathBuf],
    now: u64,
) -> WindowedAgentUsageFacts {
    if db_paths.is_empty() {
        return WindowedAgentUsageFacts {
            updated_at_unix_seconds: Some(now),
            error: Some("missing OpenCode DB".to_string()),
            ..WindowedAgentUsageFacts::default()
        };
    }

    let mut cost = OpenCodeGoUsageWindows::default();
    let mut opened_any = false;
    let mut first_error = None;
    for path in db_paths {
        match collect_opencode_go_usage_windows_from_db(path, now) {
            Ok(windows) => {
                opened_any = true;
                cost.five_hour_cost_usd += windows.five_hour_cost_usd;
                cost.weekly_cost_usd += windows.weekly_cost_usd;
                cost.monthly_cost_usd += windows.monthly_cost_usd;
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
            updated_at_unix_seconds: Some(now),
            error: first_error.or_else(|| Some("OpenCode DB unavailable".to_string())),
            ..WindowedAgentUsageFacts::default()
        };
    }

    WindowedAgentUsageFacts {
        updated_at_unix_seconds: Some(now),
        five_hour_remaining_percent: Some(remaining_percent_from_cost_limit(
            cost.five_hour_cost_usd,
            OPENCODE_GO_FIVE_HOUR_LIMIT_USD,
        )),
        weekly_remaining_percent: Some(remaining_percent_from_cost_limit(
            cost.weekly_cost_usd,
            OPENCODE_GO_WEEKLY_LIMIT_USD,
        )),
        monthly_remaining_percent: Some(remaining_percent_from_cost_limit(
            cost.monthly_cost_usd,
            OPENCODE_GO_MONTHLY_LIMIT_USD,
        )),
        ..WindowedAgentUsageFacts::default()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg(feature = "providers")]
pub struct OpenCodeGoUsageWindows {
    pub five_hour_cost_usd: f64,
    pub weekly_cost_usd: f64,
    pub monthly_cost_usd: f64,
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
    let cost_since = |window_seconds| {
        query_opencode_go_cost_usd(&connection, now.saturating_sub(window_seconds))
    };
    Ok(OpenCodeGoUsageWindows {
        five_hour_cost_usd: cost_since(OPENCODE_GO_FIVE_HOUR_SECONDS)?,
        weekly_cost_usd: cost_since(OPENCODE_GO_WEEK_SECONDS)?,
        monthly_cost_usd: cost_since(OPENCODE_GO_MONTH_SECONDS)?,
    })
}

#[cfg(feature = "providers")]
pub fn query_opencode_go_cost_usd(
    connection: &rusqlite::Connection,
    since_unix_seconds: u64,
) -> Result<f64, String> {
    connection
        .query_row(
            r#"
            SELECT COALESCE(SUM(COALESCE(json_extract(data, '$.cost'), 0.0)), 0.0)
            FROM message
            WHERE time_created >= ?1
              AND json_extract(data, '$.role') = 'assistant'
              AND json_extract(data, '$.providerID') = ?2
            "#,
            rusqlite::params![
                unix_millis_from_seconds_saturating(since_unix_seconds),
                OPENCODE_GO_PROVIDER_ID
            ],
            |row| Ok(row.get::<_, f64>(0)?.max(0.0)),
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

fn find_command_in_path_var(
    path_var: &std::ffi::OsStr,
    command_name: &str,
) -> Option<std::path::PathBuf> {
    std::env::split_paths(path_var)
        .map(|entry| entry.join(command_name))
        .find(|candidate| candidate.is_file())
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

#[cfg(test)]
mod tests {
    use super::*;

    const OBSERVED: u64 = 1_791_936_000; // 2026-10-14T00:00:00Z
    const NOW: u64 = OBSERVED + 60;

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

    fn reading(provider: &str, source: &str) -> serde_json::Value {
        serde_json::json!([{
            "provider": provider,
            "source": source,
            "usage": {
                "updatedAt": "2026-10-14T00:00:00Z",
                "primary": {"usedPercent": 23.5, "windowMinutes": 300, "resetsAt": "2026-10-14T02:00:00Z"},
                "secondary": {"usedPercent": 2, "windowMinutes": 10080, "resetsAt": "2026-10-19T20:59:59Z"},
                "tertiary": null
            },
            "identity": {"accountEmail": "discard@example.invalid"}
        }])
    }

    fn parse(
        value: &serde_json::Value,
        provider: QuotaProvider,
    ) -> Result<WindowedAgentUsageFacts, String> {
        codexbar_quota_from_json(
            &serde_json::to_vec(value).unwrap(),
            provider,
            NOW,
            std::time::Duration::from_secs(10),
        )
    }

    #[cfg(unix)]
    fn write_codexbar_script(bin_dir: &std::path::Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        std::fs::create_dir_all(bin_dir).unwrap();
        let path = bin_dir.join("codexbar");
        let shell = std::env::var_os("PATH")
            .and_then(|path| {
                std::env::split_paths(&path)
                    .map(|entry| entry.join("sh"))
                    .find(|candidate| candidate.is_file())
            })
            .unwrap_or_else(|| std::path::PathBuf::from("/bin/sh"));
        std::fs::write(&path, format!("#!{}\n{body}", shell.display())).unwrap();
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

    // Defends: every provider renders Eon's quota text: elapsed/total when a reset is known, the window
    // duration otherwise, and ` · ` between windows; empty facts hide the widget.
    #[test]
    fn renders_eon_quota_text() {
        let codex = parse(&reading("codex", "codex-cli"), QuotaProvider::Codex).unwrap();
        let periods = [AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly];

        assert_eq!(
            render_windowed_agent_usage_status_body("codex", &codex, &periods),
            "\u{ec81} 3h/5h 76% · 1d3h/7d 98%"
        );
        assert_eq!(
            render_windowed_agent_usage_status_widget("codex", &codex, &periods),
            " [\u{ec81} 3h/5h 76% · 1d3h/7d 98%]"
        );
        let claude = WindowedAgentUsageFacts {
            five_hour_remaining_percent: Some(100),
            weekly_remaining_percent: Some(101),
            ..WindowedAgentUsageFacts::default()
        };
        assert_eq!(
            render_windowed_agent_usage_status_body("claude", &claude, &periods),
            "\u{ec82} 5h 100% · 7d 100%"
        );
        let go = WindowedAgentUsageFacts {
            five_hour_remaining_percent: Some(75),
            weekly_remaining_percent: Some(70),
            monthly_remaining_percent: Some(35),
            ..WindowedAgentUsageFacts::default()
        };
        assert_eq!(
            render_windowed_agent_usage_status_body(
                "go",
                &go,
                &[
                    AgentUsagePeriod::FiveHour,
                    AgentUsagePeriod::Weekly,
                    AgentUsagePeriod::Monthly
                ]
            ),
            "go 5h 75% · 7d 70% · 30d 35%"
        );
        assert_eq!(
            render_windowed_agent_usage_status_widget(
                "go",
                &WindowedAgentUsageFacts::default(),
                &periods
            ),
            ""
        );
        assert_eq!(
            quota_window_position(300, OBSERVED + 18_000, OBSERVED),
            "0m"
        );
        assert_eq!(
            quota_window_position(300, OBSERVED + 2_400, OBSERVED),
            "4h20m"
        );
        assert_eq!(
            quota_window_position(10_080, OBSERVED + 86_400, OBSERVED),
            "6d"
        );
    }

    // Defends: CodexBar readings pass Eon's EON-C27 validation; anything else never becomes quota.
    #[test]
    fn codexbar_readings_are_validated_like_eon() {
        let claude = parse(&reading("claude", "oauth"), QuotaProvider::Claude).unwrap();
        assert_eq!(claude.updated_at_unix_seconds, Some(OBSERVED));
        assert_eq!(claude.five_hour_remaining_percent, Some(76));
        assert_eq!(
            claude.five_hour_reset_at_unix_seconds,
            Some(OBSERVED + 7_200)
        );
        assert_eq!(claude.weekly_remaining_percent, Some(98));
        assert_eq!(claude.weekly_window_seconds, Some(604_800));

        let mut idle = reading("claude", "oauth");
        idle[0]["usage"]["primary"]["resetsAt"] = serde_json::Value::Null;
        idle[0]["usage"]["secondary"]["resetsAt"] = "2026-10-13T00:00:00Z".into();
        let idle = parse(&idle, QuotaProvider::Claude).unwrap();
        assert_eq!(idle.five_hour_remaining_percent, Some(76));
        assert_eq!(idle.five_hour_reset_at_unix_seconds, None);
        assert_eq!(
            idle.weekly_remaining_percent, None,
            "a passed reset drops its window"
        );

        // Regression: the clock is sampled before CodexBar stamps its reading.
        let stamped_after_sample = codexbar_quota_from_json(
            &serde_json::to_vec(&reading("codex", "codex-cli")).unwrap(),
            QuotaProvider::Codex,
            OBSERVED - 2,
            std::time::Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(stamped_after_sample.weekly_remaining_percent, Some(98));

        let mut missing = reading("codex", "codex-cli");
        missing[0]["usage"] = serde_json::Value::Null;
        assert!(parse(&missing, QuotaProvider::Codex).unwrap().is_empty());

        type Mutation = fn(&mut serde_json::Value);
        let rejected: [(&str, Mutation); 9] = [
            ("wrong provider", |v| v[0]["provider"] = "claude".into()),
            ("wrong source", |v| v[0]["source"] = "web".into()),
            ("provider error", |v| {
                v[0]["error"] = serde_json::json!({"message": "x"})
            }),
            ("estimate", |v| {
                v[0]["usage"]["dataConfidence"] = "estimated".into()
            }),
            ("tertiary", |v| {
                v[0]["usage"]["tertiary"] = serde_json::json!({"usedPercent": 1})
            }),
            ("future", |v| {
                v[0]["usage"]["updatedAt"] = "2026-10-14T00:02:00Z".into()
            }),
            ("old", |v| {
                v[0]["usage"]["updatedAt"] = "2026-10-13T22:59:00Z".into()
            }),
            ("offset", |v| {
                v[0]["usage"]["updatedAt"] = "2026-10-14T00:00:00+00:00".into()
            }),
            ("window", |v| {
                v[0]["usage"]["primary"]["windowMinutes"] = 60.into()
            }),
        ];
        for (case, mutate) in rejected {
            let mut value = reading("codex", "codex-cli");
            mutate(&mut value);
            assert!(parse(&value, QuotaProvider::Codex).is_err(), "{case}");
        }
        assert!(parse(&serde_json::json!([]), QuotaProvider::Codex).is_err());
    }

    #[test]
    fn utc_timestamps_parse_only_codexbar_utc_form() {
        let ts = |text: &str| utc_timestamp(&serde_json::json!(text));
        assert_eq!(ts("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(ts("2000-03-01T00:00:00Z"), Some(951_868_800));
        assert_eq!(ts("2024-02-29T12:34:56.789Z"), Some(1_709_210_096));
        assert_eq!(ts("2026-10-14T00:00:00Z"), Some(OBSERVED));
        for bad in [
            "2026-10-14T00:00:00",
            "2026-10-14 00:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-10-14T24:00:00Z",
            "2026-10-14T00:00:00.Z",
            "+2026-10-14T00:00:00Z",
        ] {
            assert_eq!(ts(bad), None, "{bad}");
        }
    }

    // Defends: the widget runs CodexBar with Eon's exact arguments at most once per freshness window; a
    // failed read keeps windows that have not reset, backs off, and later recovers.
    #[cfg(unix)]
    #[test]
    fn quota_widget_refreshes_from_codexbar_and_retains_windows_on_failure() {
        let temp = temp_test_dir("codexbar_refresh");
        let bin_dir = temp.join("bin");
        let calls = bin_dir.join("calls");
        let reading_path = temp.join("reading.json");
        std::fs::write(&reading_path, reading("claude", "oauth").to_string()).unwrap();
        write_codexbar_script(
            &bin_dir,
            &format!(
                "printf '%s\\n' \"$*\" >> '{}'\ncat '{}'\n",
                calls.display(),
                reading_path.display()
            ),
        );
        let cache_path = temp
            .join("agent_usage")
            .join(QuotaProvider::Claude.cache_file_name());
        let render = |now_unix_seconds| {
            quota_usage_widget_body_text(
                QuotaProvider::Claude,
                QuotaWidgetOptions {
                    cache_path: &cache_path,
                    path_var: Some(bin_dir.as_os_str()),
                    now_unix_seconds,
                    max_age_seconds: 60,
                    error_backoff_seconds: 120,
                    timeout: std::time::Duration::from_secs(5),
                    periods: &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
                },
            )
            .unwrap()
        };
        let call_count = || std::fs::read_to_string(&calls).unwrap().lines().count();

        assert_eq!(render(NOW), "\u{ec82} 3h/5h 76% · 1d3h/7d 98%");
        assert_eq!(
            std::fs::read_to_string(&calls).unwrap(),
            "usage --provider claude --source oauth --format json --json-only --web-timeout 8\n"
        );
        assert_eq!(render(NOW + 30), "\u{ec82} 3h/5h 76% · 1d3h/7d 98%");
        assert_eq!(call_count(), 1, "fresh cache is not refreshed");

        std::fs::write(&reading_path, "not json").unwrap();
        assert_eq!(render(NOW + 7_200), "\u{ec82} 1d3h/7d 98%");
        assert_eq!(call_count(), 2);
        let cache =
            read_agent_usage_cache_value(&cache_path, CLAUDE_USAGE_CACHE_SCHEMA_VERSION).unwrap();
        assert_eq!(cache["claude"]["status"], "partial");
        assert_eq!(cache["claude"]["backoff_until_unix_seconds"], NOW + 7_320);
        assert_eq!(render(NOW + 7_300), "\u{ec82} 1d3h/7d 98%");
        assert_eq!(call_count(), 2, "backoff suppresses reads");

        let mut recovered = reading("claude", "oauth");
        recovered[0]["usage"]["updatedAt"] = "2026-10-14T02:02:00Z".into();
        recovered[0]["usage"]["primary"]["resetsAt"] = "2026-10-14T07:00:00Z".into();
        std::fs::write(&reading_path, recovered.to_string()).unwrap();
        assert_eq!(render(NOW + 7_320), "\u{ec82} 2m/5h 76% · 1d5h/7d 98%");
        assert_eq!(call_count(), 3);

        std::fs::remove_file(bin_dir.join("codexbar")).unwrap();
        let missing = temp.join("missing").join("claude_usage_cache.json");
        let text = quota_usage_widget_body_text(
            QuotaProvider::Claude,
            QuotaWidgetOptions {
                cache_path: &missing,
                path_var: Some(bin_dir.as_os_str()),
                now_unix_seconds: NOW,
                max_age_seconds: 60,
                error_backoff_seconds: 120,
                timeout: std::time::Duration::from_secs(5),
                periods: &[AgentUsagePeriod::FiveHour],
            },
        )
        .unwrap();
        assert_eq!(text, "");
        let cache =
            read_agent_usage_cache_value(&missing, CLAUDE_USAGE_CACHE_SCHEMA_VERSION).unwrap();
        assert_eq!(cache["claude"]["error"], "missing codexbar");
        assert_eq!(cache["claude"]["status"], "error");
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: OpenCode Go reads cost-limit windows from its local DBs and renders Eon's text.
    #[cfg(feature = "providers")]
    #[test]
    fn opencode_go_usage_widget_refreshes_shared_cache_from_db() {
        let temp = temp_test_dir("opencode_go_usage_refresh");
        let now = 10_000_000;
        let db_path = temp.join("opencode.db");
        write_opencode_go_test_db(&db_path, now);
        let cache_path = temp
            .join("agent_usage")
            .join("opencode_go_usage_cache_v2.json");
        let options = OpenCodeGoUsageWidgetOptions {
            cache_path: &cache_path,
            db_paths: std::slice::from_ref(&db_path),
            now_unix_seconds: now,
            max_age_seconds: 600,
            error_backoff_seconds: 1_800,
            periods: &[
                AgentUsagePeriod::FiveHour,
                AgentUsagePeriod::Weekly,
                AgentUsagePeriod::Monthly,
            ],
        };

        assert_eq!(
            opencode_go_usage_widget_body_text(options).unwrap(),
            "go 5h 75% · 7d 70% · 30d 35%"
        );
        let cache =
            read_agent_usage_cache_value(&cache_path, OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION)
                .unwrap();
        assert_eq!(cache["opencode_go"]["status"], "ok");
        std::fs::remove_file(&db_path).unwrap();
        assert!(
            !refresh_opencode_go_usage_shared_cache(&cache_path, &[db_path], now + 599, 600, 1_800)
                .unwrap()
        );
        let _ = std::fs::remove_dir_all(temp);
    }

    // Defends: the read cadence shared with Eon (eon-flf4): Claude every five minutes, Codex every minute.
    #[test]
    fn quota_read_cadence_matches_eon() {
        assert_eq!(QuotaProvider::Claude.default_max_age_seconds(), 300);
        assert_eq!(QuotaProvider::Claude.default_error_backoff_seconds(), 300);
        assert_eq!(QuotaProvider::Codex.default_max_age_seconds(), 60);
        assert_eq!(QuotaProvider::Codex.default_error_backoff_seconds(), 120);
    }

    // Defends: Nova's integrated cache path keeps one namespace per cache schema.
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
                "/state/sessions/agent_usage/codex_usage_cache_v5.json"
            ))
        );
    }
}
