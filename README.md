# Korers

A Rust desktop remote control application for Kodi, built with Relm4 and GTK4 on Linux.

## Overview

Korers is an alternative to the official Kodi remote apps (Kore, etc.) designed specifically for Linux desktop environments. It provides a native GTK4 interface for controlling your Kodi media center.

## Features

### Host Management
- Auto-discovery of Kodi instances via UDP broadcast (SSDP)
- Manual host addition with custom address, port, and credentials
- Wake-on-LAN support
- Persistent host configuration

### Remote Control
- D-pad navigation (up/down/left/right/select)
- Transport controls (play/pause/stop/previous/next)
- Volume control with mute toggle
- Numeric keypad (0-9)
- Context menu, info, back, and home buttons

### Now Playing
- Current media info display (title, artist, album)
- Album art/thumbnail display
- Progress bar with seek capability
- Transport controls

### Media Library
- Browse movies, TV shows, music
- View favorites
- Playlist management

### Additional Features
- System notifications
- Input actions (context menu, info, etc.)

## Tech Stack

- **UI Framework**: Relm4 + GTK4
- **Async Runtime**: Tokio
- **HTTP Client**: Reqwest
- **Serialization**: Serde
- **Discovery**: dns-sd (SSDP)
- **Logging**: tracing

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

Or install the binary and run:
```bash
cargo install --path .
korers
```

## Configuration

Hosts are stored in `~/.config/korers/hosts.json`.

## License

MIT OR Apache-2.0
