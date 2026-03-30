# Korers

A Rust GTK4 desktop remote for Kodi media center.

## Overview

- **Entry**: `src/main.rs` - declarative flow via `App::new().chain().run()`
- **App**: `src/app.rs` - effect container, builder chain
- **Effects**: `src/effects/` - async effect trait definitions
- **UI**: Relm4 0.9 + GTK4 (widget builders)
- **Kodi**: JSON-RPC via HTTP (`reqwest`)
- **Discovery**: SSDP/UDP on `239.255.255.250:1900` (runs on main thread)
- **Config**: `~/.config/korers/hosts.json`

## Main Flow Pattern

```rust
fn main() {
    App::new()
        .init_logging()
        .load_hosts()
        .show_window()
        .show_host_selection()
        .run();
}
```

## Modules

| Path | Purpose |
|------|---------|
| `src/app.rs` | App struct, effect container |
| `src/effects/mod.rs` | Effect trait definitions |
| `src/effects/gtk.rs` | UI effects |
| `src/effects/discovery.rs` | Discovery effects |
| `src/kodi/client.rs` | All Kodi API methods |
| `src/kodi/discovery.rs` | SSDP host discovery |
| `src/host/manager.rs` | Host config persistence |
| `src/host/mod.rs` | Host struct definition |

## Rules

- GTK widgets are NOT thread-safe - UI updates must be on main thread
- See `AGENTS.md` for Relm4 0.9 quirks and coding conventions

## Kodi API

Use Kodi JSON-RPC documentation online. Methods called via `KodiClient::call()`.

## Status

See `TODO.md` for feature roadmap. Core infrastructure done, UI integration in progress.
