# Nova Bar

Nova Bar is a standalone Zellij top-bar package for users who want Nova's bar
without adopting the full Nova workspace. It packages the `zjstatus` renderer,
Nova's preset, and optional command widgets.

## Install Shape

The flake package is:

```bash
nix build github:Yazelix/nova-bar#nova_bar
nix profile install github:Yazelix/nova-bar#nova_bar
```

The package installs:

- `bin/nova_bar_widget`
- `share/nova_bar/zjstatus.wasm`
- `share/nova_bar/nova_bar.kdl`
- `share/nova_bar/nova_bar.template.kdl`
- `share/nova_bar/nova_runtime_bar.template.kdl`
- `share/nova_bar/examples/custom_command_widgets.kdl`
- `share/nova_bar/examples/standalone_zellij_layout.kdl`
- `share/nova_bar/examples/nova_runtime_widgets.kdl`
- `share/doc/nova_bar/README.md`

Use `nova_bar.kdl` as a Zellij layout plugin block. The template keeps `__NOVA_BAR_ZJSTATUS_WASM__` for users who want to substitute a different pinned `zjstatus.wasm`. The runtime template is for Nova and embedders; vanilla Zellij users can ignore it. The example snippets are small blocks to copy into the plugin body rather than alternate full presets.

## Minimal Zellij Layout Snippet

```kdl
layout {
    pane size=1 borderless=true {
        // Paste the contents of share/nova_bar/nova_bar.kdl here
    }
    pane
}
```

The packaged `nova_bar.kdl` points at the package's `zjstatus.wasm` with a `file:` URL. The wasm comes from the repo's pinned `zjstatus` flake input.

## Generic Boundary

The standalone default is intentionally generic:

- mode
- tabs
- session
- datetime
- Nova colors and tab overflow behavior

It does not require:

- `yzx`
- `yzx_control`
- Nova runtime paths
- the Nova pane orchestrator
- Nushell
- a full Nova installation

The optional provider widgets only need their own upstream facts:

- `codex_usage` and `claude_usage` use `tokenusage` when it is available on `PATH`
- `opencode_go_usage` reads OpenCode Go SQLite databases from default locations or explicit `--db` paths
- `cpu` and `ram` read a shared `sysinfo`-backed cache with a short freshness window

None of those widget commands require Nova runtime paths, `yzx_control`, or a Nova session cache.

Nova keeps zjstatus `{tabs}` as the integrated tab strip so Zellij owns live tab identity, focus, click handling, and bell events. Execution and agent activity are not rendered in tab labels; Nova's Radar sidebar owns that presentation.

The integrated bar gives tabs priority over widgets. The version badge disappears
first, followed by widgets from right to left; the configured left-to-right order
is preserved. Empty widgets and their separators take no space, and widgets return
as the bar widens. If tabs alone overflow, zjstatus keeps the active tab visible
and shortens its name only as a last resort. Nova Bar emits complete `{segment}`
bodies and `format_right_separator`; zjstatus owns width and click geometry under
its BAR-WIDTH-001 contract. Standalone presets retain their existing configuration.
Nova's integrated preset shows compact command placeholders on the first render
and replaces each one when its result arrives.

## Optional Command Widgets

Standalone users can add zjstatus command widgets directly in their own copied preset. Command stdout should be short plain text because the KDL format owns the style.

The main customization knobs are:

- `format_left`, `format_center`, and `format_right` for widget order
- inline `#[fg=...]` and `#[bg=...]` style tags for color
- mode and tab format keys for labels and tab display
- `command_*_command`, `command_*_format`, and `command_*_interval` for custom command widgets

Generic zjstatus placeholders such as `{mode}`, `{tabs}`, `{session}`, and `{datetime}` work without Nova. To add a host/status command widget, start from:

```kdl
format_right "#[fg=#ff0088,bold]{session} {datetime} {command_host} #[fg=#00ccff,bold]nova bar "

command_host_command "hostname -s"
command_host_format " #[fg=#00ff88,bold][{stdout}]"
command_host_interval "30"
```

The packaged `share/nova_bar/examples/custom_command_widgets.kdl` contains a slightly larger version of this pattern.

## Complete Standalone Widget Layout

`share/nova_bar/examples/standalone_zellij_layout.kdl` is a complete plain Zellij layout that uses every bar-owned non-workspace widget:

```kdl
layout {
    pane size=1 borderless=true {
        plugin location="file:/absolute/path/to/share/nova_bar/zjstatus.wasm" {
            format_left "{mode} {tabs}"
            format_center ""
            format_right "#[fg=#ff0088,bold]{session}{command_claude_usage}{command_codex_usage}{command_opencode_go_usage}{command_cpu}{command_ram} #[fg=#00ccff,bold]nova bar "

            command_claude_usage_command "nova_bar_widget claude"
            command_claude_usage_format "#[fg=#bb88ff,bold]{stdout}"
            command_claude_usage_interval "10"

            command_codex_usage_command "nova_bar_widget codex"
            command_codex_usage_format "#[fg=#bb88ff,bold]{stdout}"
            command_codex_usage_interval "10"

            command_opencode_go_usage_command "nova_bar_widget opencode_go"
            command_opencode_go_usage_format "#[fg=#bb88ff,bold]{stdout}"
            command_opencode_go_usage_interval "10"

            command_cpu_command "nova_bar_widget cpu"
            command_cpu_format " #[fg=#ff6600][cpu {stdout}]"
            command_cpu_interval "5"

            command_ram_command "nova_bar_widget ram"
            command_ram_format " #[fg=#ff6600][ram {stdout}]"
            command_ram_interval "5"
        }
    }
    pane
}
```

Replace the `zjstatus.wasm` path with the installed package path. If you install through Nix profiles, `nix profile list` shows the profile entry and package output.

Provider widgets maintain their own cache, lock, freshness, and error-backoff files under `$XDG_CACHE_HOME/nova_bar` or `$HOME/.cache/nova_bar`. CPU/RAM share one short-lived system-usage cache so command-widget bursts do not resample system metrics in every process. Use `--cache` only when overriding the default. Nova may omit it because the full runtime exports `YAZELIX_STATUS_BAR_CACHE_PATH`.

Codex quota display queries official limits once per minute without reading or updating local session history. A failed refresh waits two minutes; retained values use the `codex~` label until a refresh succeeds. Token and combined displays keep their 10-minute cache and 30-minute failure backoff because they still read local history.

## Standalone Fact Renderers

The Rust crate also exposes renderer helpers for embedders that want Nova-style widget text without a Nova runtime:

- `render_zjstatus_bar_segments` for editor, shell, terminal, custom text, and widget-tray placeholders
- `render_zjstatus_tab_label_formats` for full and compact tab labels
- `render_nova_runtime_plugin_block` for the integrated Nova zjstatus plugin block from typed runtime config, `appearance_mode`, and the child-owned runtime KDL template
- `render_codex_usage_status_widget` for cached Codex usage facts
- `render_windowed_agent_usage_status_widget` for Claude, OpenCode Go, or another cached windowed provider

Minimal cached provider example:

```rust
use nova_bar::{
    AgentUsageDisplay, AgentUsagePeriod, WindowedAgentUsageFacts,
    render_windowed_agent_usage_status_widget,
};

let text = render_windowed_agent_usage_status_widget(
    "claude",
    &WindowedAgentUsageFacts {
        five_hour_tokens: Some(15_456_373),
        weekly_tokens: Some(66_610_005),
        five_hour_remaining_percent: Some(49),
        weekly_remaining_percent: Some(80),
        ..WindowedAgentUsageFacts::default()
    },
    &[AgentUsagePeriod::FiveHour, AgentUsagePeriod::Weekly],
    AgentUsageDisplay::Both,
);
```

Those helpers are facts-in, styled-text-out. They do not read `~/.config/yazelix`, `~/.local/share/yazelix`, `yzx_control`, Nova session cache files, tokenusage, OpenCode databases, or pane-orchestrator state.

## Nova-Specific Widgets

Workspace remains Nova-only because it is derived from Nova session facts and pushed into a `pipe_workspace` widget by the Nova pane orchestrator. Terminal display is also Nova-only in the integrated runtime because `nova_bar_widget term` reads the running session's `YAZELIX_SESSION_TERMINAL` value instead of trusting a shared generated layout. The Nova badge is runtime-only: its `version` command reads the exact packaged version and `stable`, `main`, or `edge` channel from `runtime_identity.json` and renders `NOVA <compact-version> <CHANNEL>`.

CPU, RAM, Codex, Claude, and OpenCode Go widgets are bar-owned standalone commands. Nova-only integration for those widgets is limited to generated layout wiring and default cache-path derivation from the full runtime.

Nova consumes this child repo for integrated zjstatus plugin rendering and the standalone package. The child repo packages `zjstatus.wasm` from its pinned `zjstatus` flake input, so the package does not require manual artifact copying. The standalone pin supports native bell tab formatting. Nova supplies a narrow Yazelix `zjstatus` fork for host-theme switching and separator-safe workspace pipes.

`nova_bar_widget render-nova-runtime --json <config>` accepts typed runtime config from Nova and returns the complete child-owned zjstatus plugin block rendered from `nova_runtime_bar.template.kdl`. The runtime config includes `appearance_mode` so Nova Bar can own dark and light palettes. The integrated template uses zjstatus `{tabs}` for live Zellij tab state, terminal-bell styling, and native layout indicators. Nova core owns workspace facts, session config, and runtime path resolution. This repo owns widget rendering, tab formatting, pipe and command-widget KDL, and the generic zjstatus plugin shape.

Nova makes this repo's `zjstatus` input follow its own pin. Standalone users get the pin recorded in this repo's `flake.lock`.

Use `share/nova_bar/examples/nova_runtime_widgets.kdl` only inside a full Nova runtime or after replacing the Nova-only workspace and version helpers. Use `share/nova_bar/examples/standalone_zellij_layout.kdl` for plain Zellij.

## Current Limit

Zellij and zjstatus presets do not have a native include or variables layer. Edit the standalone KDL directly for brand, color, order, and command-widget changes; copy `nova_bar.template.kdl` when substituting a different pinned `zjstatus.wasm`. Nova uses a separate runtime template so standalone customizations stay plain Zellij KDL.

## Release Process

Maintainers update the pinned zjstatus input deliberately, then validate:

```bash
nix flake lock --update-input zjstatus
nix build .#nova_bar
cargo test
```

If the standalone preset grows beyond zjstatus configuration, the next step is a real plugin decision rather than forking zjstatus by default.
