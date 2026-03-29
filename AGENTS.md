# Agentic Coding Guide

This document provides guidance for AI agents working on the Korers codebase.

## Ground rules

* No command shall be run without asking the user first.

## Project Structure

```
src/
├── main.rs          # Application entry point
├── kodi/            # Kodi JSON-RPC client
│   ├── client.rs    # HTTP client for Kodi API
│   ├── discovery.rs # SSDP/UDP host discovery
│   └── types.rs    # Data types and serialization
├── host/            # Host configuration management
│   ├── mod.rs       # Host struct definition
│   └── manager.rs   # Host persistence (JSON config)
└── ui/              # Relm4 UI components
    ├── main_window.rs  # Main window with sidebar
    ├── remote.rs       # D-pad remote control
    ├── now_playing.rs  # Current media display
    ├── host_list.rs    # Host discovery/selection
    └── discovery.rs    # (placeholder)
```

## Key Conventions

### Relm4 Components
- Use `#[relm4::component]` macro for components
- Define `Model`, `Msg` (input), and `Widgets` structs
- Use `view!` macro for declarative UI definition
- Message naming: `ComponentNameMsg::ActionName`
- Use `ComponentSender<Model>` for emitting outputs

### Async Operations
- All Kodi API calls are async (Tokio)
- Use `.await` for API calls
- Run async tasks in spawned tokio runtime or use `async_runtime` crate

### Error Handling
- Use `thiserror` for error types
- `ClientError` enum in `kodi/client.rs` for API errors
- `ManagerError` in `host/manager.rs` for config errors

### Naming
- Modules: snake_case (`host_manager`, not `hostManager`)
- Types/Structs: PascalCase
- Fields: snake_case
- Messages: PascalCase with action verb

## Adding New Features

### 1. New Kodi API Method

Add to `src/kodi/client.rs`:
```rust
pub async fn method_name(&self, params: ParamsType) -> Result<ResponseType, ClientError> {
    #[derive(Serialize)]
    struct Params { /* fields */ }
    #[derive(Deserialize)]
    struct Response { /* fields */ }
    
    self.call(JsonRpcRequest::new("Method.Name").with_params(Params { /* */ }))
        .await
}
```

### 2. New UI Component

Create new file in `src/ui/`:
```rust
use relm4::{ComponentParts, ComponentSender, SimpleComponent, view};

pub struct Model { /* state */ }
pub enum Msg { /* messages */ }

#[relm4::component]
impl SimpleComponent for Model {
    type Init = /* */;
    type Input = Msg;
    type Output = /* */;
    type Widgets = Widgets;

    view! { /* UI definition */ }

    fn init(/* */) -> ComponentParts<Self> { /* */ }
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) { /* */ }
    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) { /* */ }
}

#[relm4::macros::widget_struct]
pub struct Widgets { /* widget fields */ }
```

### 3. Adding Types

Add to appropriate module's `types.rs` or inline in client:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewType {
    pub field: Type,
}
```

## Testing

Run tests with:
```bash
cargo test
```

For UI testing, consider using GTK4's testing utilities or manual widget verification.

## Linting

Run before committing:
```bash
cargo clippy
cargo fmt
```

## Dependencies

See `Cargo.toml` for all dependencies. Key ones:
- `gtk4` / `relm4` - UI
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde` / `serde_json` - Serialization
- `dns-sd` - Service discovery
- `tracing` - Logging
