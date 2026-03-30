# Agentic Coding Guide

This document provides guidance for AI agents working on the Korers codebase.

## Ground rules

* No command shall be run without asking the user first.

## Project Structure

```
src/
├── main.rs          # Application entry point (declarative flow)
├── app.rs           # App struct, effect container, builder chain
├── effects/         # Effect traits and implementations
│   ├── mod.rs       # Trait definitions
│   ├── gtk.rs       # UI operations
│   ├── discovery.rs # SSDP discovery
│   └── kodi.rs      # Kodi client operations
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

### Main Entry Point Pattern

The `main.rs` should read like English - a declarative specification of app flow. Use the builder pattern:

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

The `App` struct lives in `app.rs` and holds the effect container:

```rust
pub struct App {
    effects: Effects,
}

struct Effects {
    host_manager: HostManager,
    discovery: DiscoveryService,
}

impl App {
    pub fn new() -> Self { ... }
    pub fn init_logging(self) -> Self { ... }
    pub fn load_hosts(self) -> Self { ... }
    pub fn show_window(self) -> Self { ... }
    pub fn show_host_selection(self) -> Self { ... }
    pub fn run(self) { gtk::main(); }
}
```

### Relm4 Components
- Use `#[relm4::component]` macro for components
- Define `Model`, `Msg` (input), and `Widgets` structs
- Use `view!` macro for declarative UI definition
- Message naming: `ComponentNameMsg::ActionName`
- Use `ComponentSender<Model>` for emitting outputs
- **State lives in Model** - the Relm4 component manages all app state

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

## Important Learnings

### Relm4 0.9 quirks

The codebase uses Relm4 0.9, which has some API differences from newer versions:

1. **`view!` macro limitations**: The macro has specific syntax requirements. For complex UIs, it's often easier to build the UI programmatically in `main.rs` rather than using the `#[relm4::component]` macro.

2. **Widget builders**: Use builder pattern with `.builder()` for constructing widgets:
   ```rust
   gtk::Button::builder()
       .label("Discover")
       .sensitive(true)
       .build()
   ```

3. **Margin methods**: GTK4 renamed many margin methods:
   - `set_margin(x)` → `set_margin_all(x)` or use `margin_start()`, `margin_end()`, etc.
   - `set_spacing(x)` instead of constructor parameter

4. **ListBoxRow**: Add children before appending:
   ```rust
   let row = gtk::ListBoxRow::new();
   row.set_child(Some(&box_));
   list.append(&row);
   ```

### Threading with GTK

GTK widgets are NOT thread-safe. Critical rules:

1. **Never pass GTK widgets to `std::thread::spawn`**: This will fail with `*mut c_void cannot be sent between threads safely`

2. **UI updates must happen on main thread**: Use synchronous operations in callbacks, or if using threads:
   - Do background work in thread
   - Pass only simple data (strings, ints) back to main thread
   - Update widgets in the main thread after thread completes

3. **Clone widgets before moving into closures**:
   ```rust
   discover_button.clone().connect_clicked(move |_| {
       // use the cloned button
   });
   ```

4. **Quick operations can run on main thread**: For fast operations like SSDP discovery, run them directly on the main thread rather than spawning threads:
   ```rust
   discover_button.connect_clicked(move |btn| {
       let discovery = DiscoveryService::new();
       match discovery.discover_all(5) {
           Ok(hosts) => { /* update UI */ }
           Err(e) => { /* show error */ }
       }
   });
   ```

### Simple UI Approach

For complex UIs, build the UI in `app.rs` or a Relm4 component:

```rust
let app = Application::builder()
    .application_id("org.korers.app")
    .build();

app.connect_activate(|app| {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Korers")
        .default_width(800)
        .default_height(600)
        .build();
    
    // Build UI programmatically here
    // Access widgets via cloned references in callbacks
});
```

See `src/app.rs` for the effect container pattern.

### Async Operations in Relm4

All Kodi API calls are async. Use `Command::future` to run async operations:

```rust
use relm4::Command;

fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
    match msg {
        Msg::Discover => {
            let fut = async_discovery();
            Command::future(fut.map(|hosts| Msg::DiscoveryComplete(Ok(hosts))));
        }
        Msg::Connect(host) => {
            let fut = async_connect(host);
            Command::future(fut.map(|result| Msg::ConnectionComplete(result)));
        }
    }
}
```

Effects are dispatched via the `effects/` module. See `src/effects/mod.rs` for trait definitions.

### SSDP/UDP Discovery

The discovery service uses:
- Multicast address: `239.255.255.250:1900`
- MSEARCH request with headers: `MAN: "ssdp:discover"`, `MX: 3`, `ST: ssdp:all`
- Parse `Location:` header for host IP and port
