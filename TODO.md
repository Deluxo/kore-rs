# Korers TODO

## Project Status: In Development

## Phase 1: Core Infrastructure

### Host Management

- [x] Host struct with basic fields (id, name, address, port)
- [x] Host struct with credentials (username, password)
- [x] Host struct with advanced options (MAC address, WoL port, TLS)
- [x] Host URL generation with auth
- [x] HostManager for persistence (JSON config)
- [x] Load hosts from config
- [x] Save hosts to config
- [x] Add host functionality
- [x] Remove host functionality
- [x] Update host functionality

### Kodi Discovery

- [x] UDP/SSDP discovery service
- [x] Parse discovery responses
- [x] Timeout handling
- [x] Multiple host detection

### Kodi API Client

- [x] Basic HTTP client setup
- [x] JSON-RPC request/response handling
- [x] Error handling (network, JSON, Kodi errors)
- [x] Ping method
- [x] GetSystemInfo method
- [x] Application properties (volume, mute)
- [x] Set volume
- [x] Set mute
- [x] Get active players
- [x] Player properties
- [x] Get current item
- [x] Play/Pause
- [x] Stop
- [x] Seek
- [x] Open (play media)
- [x] Go to (previous/next)
- [x] Playlist operations
- [x] Input actions (up/down/left/right/etc.)
- [x] Show notification
- [x] Get files/sources
- [x] Get movies
- [x] Get TV shows
- [x] Get songs
- [x] Get albums
- [x] Get favorites

## Phase 2: UI Components

### Main Window

- [x] Application window setup
- [x] Header bar with title
- [x] Sidebar navigation (StackSidebar)
- [x] Content stack for views
- [x] Status bar
- [x] AppModel with state management
- [x] AppView enum for navigation

### Host List

- [x] Host list UI component
- [x] Discover hosts button
- [x] Add host manually button
- [x] List of hosts display
- [x] Host selection
- [x] Remove host

### Remote Control

- [x] D-pad layout (up/down/left/right)
- [x] Select button
- [x] Back button
- [x] Home button
- [x] Info button
- [x] Context menu button
- [x] Transport controls (play/pause/stop/prev/next)
- [x] Volume controls
- [x] Numeric keypad (0-9)

### Now Playing

- [x] Thumbnail display
- [x] Title label
- [x] Artist label
- [x] Album label
- [x] Progress bar (scale)
- [x] Current time / duration labels
- [x] Transport controls
- [x] Volume controls

## Phase 3: Integration & Polish

### Main App Integration

- [ ] Connect host list to host manager
- [ ] Connect remote to Kodi client
- [ ] Connect now playing to Kodi client
- [ ] Real-time player state updates
- [ ] Periodic polling for now playing info
- [ ] Connect all components to main window

### Wake-on-LAN

- [ ] MAC address configuration
- [ ] Send WoL packet
- [ ] WoL button in UI

### Error Handling

- [ ] Connection error display
- [ ] Timeout handling in UI
- [ ] Offline host indicators
- [ ] Reconnection logic

### Settings

- [ ] Settings view/page
- [ ] Default host selection
- [ ] Polling interval configuration
- [ ] Theme preference (future)

## Phase 4: Advanced Features

### Media Library Browsing

- [ ] Movies view
- [ ] TV Shows view
- [ ] Music view
- [ ] File browser
- [ ] Favorites view

### Playlist Management

- [ ] View current playlist
- [ ] Add to playlist
- [ ] Clear playlist

### Keyboard Shortcuts

- [ ] Arrow keys for navigation
- [ ] Enter for select
- [ ] Escape for back
- [ ] Space for play/pause

## Future Considerations

- [ ] Multiple simultaneous connections
- [ ] Connection profiles
- [ ] Recording remote commands (macros)
- [ ] Touch-friendly layout for tablets
- [ ] Dark/Light theme support
- [ ] System tray integration
- [ ] Global hotkeys
- [ ] Activity logging

## Known Issues

- Discovery may not find all Kodi instances on all networks
- Image loading is not yet implemented
- Async UI updates need proper Tokio integration
