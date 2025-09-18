# SlingCraft Multiplayer Implementation Plan

## Goals
1. Integrate the lightyear bevy networking library.
2. Users can log in to the web version of the app enter a game code (UUID) to join an online game.
3. Once they join, they can see who else is in the game with them.
4. If someone leaves or joins the game while it is open, the list of current game participants is instantly updated.
5. Each user should be able to move a colored circle around in a box that everyone can see.

## Technical Architecture

### Network Transport
- **Primary**: WebTransport (QUIC-based, works on both native and WASM)
- **Fallback**: WebSocket for environments without WebTransport support
- **Protocol**: Client-server architecture with server authority
- **Advantages of WebTransport**:
  - Lower latency than WebSocket (uses QUIC/UDP under the hood)
  - Better performance for real-time games
  - Built-in stream multiplexing
  - Works through firewalls and NATs
- **Prediction**: Client-side prediction for responsive movement
- **Interpolation**: Smooth position updates for other players

### Core Components

#### Shared Components (client + server)
```rust
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerId(pub u32);

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlayerName(pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayerInput {
    Move { direction: Vec2 },
    None,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct GameSession {
    pub room_code: String,
    pub max_players: u32,
}
```

### Implementation Phases

## Phase 1: Basic Networking Setup

### Dependencies to Add
```toml
[dependencies]
lightyear = { version = "0.24.2", features = [
    "client",
    "server",
    "webtransport",
    "websocket",  # Fallback option
    "interpolation",
    "prediction",
    "netcode"
] }
uuid = { version = "1.10", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# For WebTransport support
[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = { version = "0.3", features = ["Window"] }
wasm-bindgen = "0.2"
```

### Core Files Structure
```
src/
├── main.rs           # Entry point with client/server selection
├── shared/           # Shared code between client and server
│   ├── mod.rs
│   ├── components.rs # Shared components and protocols
│   └── systems.rs    # Shared game logic
├── client/           # Client-specific code
│   ├── mod.rs
│   ├── ui.rs         # Game lobby and UI
│   ├── input.rs      # Input handling
│   └── rendering.rs  # Player circle rendering
└── server/           # Server-specific code
    ├── mod.rs
    ├── lobby.rs      # Room management
    └── gamestate.rs  # Server game state
```

### Step 1.1: Lightyear Integration
- [ ] Add lightyear dependency with WebTransport as primary transport
- [ ] Configure WebTransport server with TLS certificates (required for HTTPS)
- [ ] Create shared protocol definition
- [ ] Set up basic client-server connection with automatic fallback
- [ ] Implement heartbeat and connection management
- [ ] Configure QUIC parameters for optimal game performance

### Step 1.2: WebTransport Configuration
- [ ] Generate self-signed certificates for local development
- [ ] Configure WebTransport server settings:
  ```rust
  // Server configuration
  let transport_config = TransportConfig::WebTransport {
      server_addr: "127.0.0.1:443".parse().unwrap(),
      certificate: include_bytes!("../cert.pem").to_vec(),
      private_key: include_bytes!("../key.pem").to_vec(),
  };
  ```
- [ ] Implement connection fallback logic:
  ```rust
  // Client connection with fallback
  async fn connect_with_fallback() -> Result<ClientConnection> {
      // Try WebTransport first
      match connect_webtransport(server_addr).await {
          Ok(conn) => Ok(conn),
          Err(_) => {
              // Fallback to WebSocket
              connect_websocket(server_addr).await
          }
      }
  }
  ```

### Step 1.3: Basic Player Entity
- [ ] Define `Player` entity with position and color components
- [ ] Implement replication of player entities
- [ ] Add client-side prediction for local player movement
- [ ] Add interpolation for remote players

## Phase 2: Game Lobby System

### Step 2.1: Room Management
- [ ] Server-side room creation with UUID codes
- [ ] Room creator tracking and permissions
- [ ] Room join/leave functionality
- [ ] Maximum 32 players per room enforcement
- [ ] Room persistence for 1 week of inactivity
- [ ] Creator-initiated room closure
- [ ] Automatic cleanup after 1 week of no connections

### Step 2.2: Player List UI
- [ ] Real-time player list display in egui
- [ ] Player name input and validation
- [ ] Color assignment (random or selection)
- [ ] Connection status indicators

### Step 2.3: Game Code Interface
```rust
// UI components needed
struct LobbyUI {
    game_code_input: String,
    player_name: String,  // User-chosen, any name allowed
    connection_status: ConnectionStatus,
    players_in_room: Vec<PlayerInfo>,
    is_creator: bool,     // Shows close room button if true
    room_created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}

struct PlayerInfo {
    name: String,
    color: Color,
    is_creator: bool,
    connected_since: DateTime<Utc>,
}

enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}
```

## Phase 3: Real-time Gameplay

### Step 3.1: Player Movement
- [ ] Input capture for WASD/arrow keys
- [ ] Send movement inputs to server
- [ ] Server-side movement validation and physics
- [ ] Broadcast position updates to all clients

### Step 3.2: Game Area
- [ ] Define bounded play area (e.g., 800x600 box)
- [ ] Collision detection with boundaries
- [ ] Visual boundary rendering
- [ ] Camera setup for game view

### Step 3.3: Player Rendering
```rust
// Rendering system for player circles
fn render_players(
    mut gizmos: Gizmos,
    players: Query<(&PlayerPosition, &PlayerColor), With<PlayerId>>,
) {
    for (pos, color) in players.iter() {
        gizmos.circle_2d(
            Vec2::new(pos.x, pos.y),
            20.0, // radius
            Color::rgba(color.r, color.g, color.b, 1.0),
        );
    }
}
```

## Phase 4: Advanced Features

### Step 4.1: Network Optimization
- [ ] Implement delta compression for position updates
- [ ] Add interest management (only sync nearby players)
- [ ] Bandwidth limiting and quality adaptation
- [ ] Lag compensation techniques

### Step 4.2: Game Features
- [ ] Player trails or animation effects
- [ ] Simple collision detection between players
- [ ] Score or objective system
- [ ] Spectator mode for disconnected players

### Step 4.3: Error Handling & Polish
- [ ] Reconnection logic
- [ ] Graceful server shutdown handling
- [ ] Player timeout detection
- [ ] Network quality indicators

## Technical Considerations

### WebTransport vs WebSocket vs UDP Trade-offs
- **WebTransport**:
  - Best of both worlds: UDP-like performance with web compatibility
  - Uses QUIC protocol (UDP under the hood with reliability)
  - Lower latency than WebSocket (~50% reduction)
  - Native stream multiplexing and prioritization
  - Requires HTTPS/TLS certificates
- **WebSocket**:
  - Fallback option for older browsers
  - Works everywhere but higher latency
  - TCP-based, guaranteed ordering
- **UDP**:
  - Lowest latency but native-only
  - No built-in reliability or ordering
  - Firewall and NAT traversal issues

### Client-Side Prediction Strategy
1. **Local Input**: Apply movement immediately on client
2. **Server Reconciliation**: Compare with server state and correct if needed
3. **Interpolation**: Smooth other players' movements between updates

### State Synchronization
- **Server Authority**: All game state decisions made on server
- **Snapshot System**: Periodic full state sync for correction
- **Delta Updates**: Send only changes between snapshots

### Deployment Architecture
```
[Web Client] ←WebTransport→ [Game Server] ←WebTransport→ [Native Client]
      ↓ (fallback)              ↓
   WebSocket              [Room Manager]
                               ↓
                     [Persistent Storage] (optional)

Server Requirements:
- HTTPS with valid TLS certificate (for WebTransport)
- Port 443 for WebTransport (standard HTTPS port)
- Optional: Port for WebSocket fallback
```

## Testing Strategy

### Phase Testing
1. **Networking**: Test client-server connection stability
2. **Lobby**: Verify room creation, joining, and player list updates
3. **Gameplay**: Test movement synchronization and lag compensation
4. **Scale**: Test with multiple concurrent rooms and players

### Performance Targets (Initial)
- **Latency**: Best effort (not optimized)
- **Update Rate**: 20-30 Hz (sufficient for initial testing)
- **Concurrent Players**: Up to 32 players per room
- **Concurrent Rooms**: 5-10 rooms for initial implementation
- **Focus**: Functionality over performance initially

## System Requirements (Confirmed)

1. **Server Hosting**: Local/on-premises server for now
2. **Player Capacity**: Maximum 32 players per room
3. **Persistence**: Games persist until:
   - Creator manually closes the room
   - No players connected for 1 week
4. **Authentication**:
   - Game code-based entry (UUID)
   - Players can choose any display name after joining
   - No user accounts required
5. **Game Rules**: Players move colored circles in a shared space (extensible later)
6. **Performance**: Not a priority for initial implementation

## Phase 5: End-to-End Testing System

### E2E Testing Architecture

#### Test Framework
```toml
[dev-dependencies]
kittest = "0.1"  # For automated input control
egui = "0.32"    # All UI for test runner
chrono = "0.4"   # Timing and timestamps
```

#### Test Runner Window Layout
```
+------------------+--------------------------------+
|                  |  Client 1  |  Client 2  |     |
|  Test Steps      |------------|------------|     |
|  Panel           |  Client 3  |  Client 4  |     |
|  (Left Side)     |------------|------------|     |
|                  |  Client 5  |  Client 6  |     |
|  [▶ Play]        |------------|------------|     |
|  [⏸ Pause]       |    ...     |    ...     |     |
|  [↻ Reset]       |  (Tiled Client Windows)        |
+------------------+--------------------------------+
```

### Test Client Configuration
```rust
struct E2ETestConfig {
    num_clients: usize,        // 2-32 clients for testing
    grid_layout: (usize, usize), // e.g., (4, 8) for 32 clients
    window_size: Vec2,          // Size of each client window
    test_mode: TestMode,
}

enum TestMode {
    Automatic,      // Runs all steps without user intervention
    SemiAutomatic,  // Requires Enter key to proceed between steps
}

struct TestClient {
    id: usize,
    name: String,
    game_code: String,
    position: Vec2,
    color: Color,
    kittest_controller: KittestController,
}
```

### Test Steps Implementation
```rust
#[derive(Clone)]
struct TestStep {
    id: usize,
    name: String,
    description: String,
    action: TestAction,
    validation: TestValidation,
    status: TestStatus,
}

enum TestAction {
    CreateRoom { client_id: usize },
    JoinRoom { client_id: usize, code: String },
    MovePlayer { client_id: usize, direction: Vec2, duration: f32 },
    Disconnect { client_id: usize },
    Reconnect { client_id: usize },
    Wait { seconds: f32 },
}

enum TestValidation {
    PlayerCount(usize),
    PlayerPosition { client_id: usize, expected: Vec2, tolerance: f32 },
    RoomExists(String),
    PlayerVisible { observer_id: usize, target_id: usize },
    ConnectionStatus { client_id: usize, expected: bool },
}

enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed(String),
    Skipped,
}
```

### Test Scenarios

#### Scenario 1: Basic Room Creation and Join
1. Client 1 creates a room
2. Validate room code generated
3. Clients 2-4 join with the code
4. Validate all players see each other
5. Each client moves in different directions
6. Validate positions synchronized

#### Scenario 2: Connection Resilience
1. Create room with 5 clients
2. Client 3 disconnects
3. Validate other clients see disconnect
4. Client 3 reconnects
5. Validate state restoration
6. All clients move simultaneously
7. Validate no desync

#### Scenario 3: Maximum Capacity
1. Create room
2. Add clients incrementally up to 32
3. Validate 33rd client rejected
4. Remove 5 clients
5. Add 5 new clients
6. Validate room stability

#### Scenario 4: Room Persistence
1. Create room with 3 clients
2. All clients disconnect
3. Wait 30 seconds
4. Clients rejoin with same code
5. Validate room state preserved
6. Creator closes room
7. Validate room deleted

### Test UI Components (egui)
```rust
fn render_test_panel(ui: &mut Ui, test_state: &mut TestState) {
    ui.vertical(|ui| {
        ui.heading("E2E Test Runner");

        // Test mode selector
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.radio_value(&mut test_state.mode, TestMode::Automatic, "Auto");
            ui.radio_value(&mut test_state.mode, TestMode::SemiAutomatic, "Semi-Auto");
        });

        // Control buttons
        ui.horizontal(|ui| {
            if ui.button("▶ Play").clicked() {
                test_state.start();
            }
            if ui.button("⏸ Pause").clicked() {
                test_state.pause();
            }
            if ui.button("↻ Reset").clicked() {
                test_state.reset();
            }
        });

        ui.separator();

        // Test steps list
        ScrollArea::vertical().show(ui, |ui| {
            for (idx, step) in test_state.steps.iter().enumerate() {
                ui.horizontal(|ui| {
                    // Status indicator
                    match step.status {
                        TestStatus::Passed => ui.colored_label(Color32::GREEN, "✓"),
                        TestStatus::Failed(_) => ui.colored_label(Color32::RED, "✗"),
                        TestStatus::Running => ui.colored_label(Color32::YELLOW, "⟳"),
                        TestStatus::Pending => ui.colored_label(Color32::GRAY, "○"),
                        TestStatus::Skipped => ui.colored_label(Color32::GRAY, "-"),
                    };

                    // Step name and description
                    let is_current = idx == test_state.current_step;
                    if is_current {
                        ui.strong(format!("{}: {}", step.name, step.description));
                    } else {
                        ui.label(format!("{}: {}", step.name, step.description));
                    }
                });

                // Show failure reason if failed
                if let TestStatus::Failed(reason) = &step.status {
                    ui.indent("failure", |ui| {
                        ui.colored_label(Color32::RED, reason);
                    });
                }
            }
        });

        ui.separator();

        // Progress bar
        let progress = test_state.current_step as f32 / test_state.steps.len() as f32;
        ui.add(ProgressBar::new(progress).text(format!(
            "Step {}/{}",
            test_state.current_step + 1,
            test_state.steps.len()
        )));
    });
}
```

### Kittest Integration
```rust
use kittest::{Input, Key, MouseButton};

impl TestClient {
    async fn execute_input(&mut self, input: TestInput) {
        match input {
            TestInput::Move(direction) => {
                // Simulate WASD or arrow keys based on direction
                let key = match direction {
                    Vec2::UP => Key::W,
                    Vec2::DOWN => Key::S,
                    Vec2::LEFT => Key::A,
                    Vec2::RIGHT => Key::D,
                    _ => return,
                };
                self.kittest_controller.press_key(key).await;
            }
            TestInput::Click(position) => {
                self.kittest_controller.click_at(position).await;
            }
            TestInput::TypeText(text) => {
                self.kittest_controller.type_text(&text).await;
            }
        }
    }
}
```

### Test Execution Flow
```rust
async fn run_e2e_test(config: E2ETestConfig) {
    let mut test_state = TestState::new(config);

    // Phase 1: Run automatic mode
    test_state.mode = TestMode::Automatic;
    println!("Running automatic test...");

    while !test_state.is_complete() {
        test_state.execute_current_step().await;

        if test_state.current_step_failed() {
            println!("Test failed at step: {}", test_state.current_step_name());
            break;
        }
    }

    if test_state.all_passed() {
        println!("Automatic test passed! Switching to semi-automatic mode for inspection.");

        // Phase 2: Run semi-automatic mode for user inspection
        test_state.reset();
        test_state.mode = TestMode::SemiAutomatic;

        println!("Press Enter to advance through each test step...");

        while !test_state.is_complete() {
            // Wait for user input in semi-automatic mode
            if test_state.mode == TestMode::SemiAutomatic {
                wait_for_enter().await;
            }

            test_state.execute_current_step().await;

            println!("Step {}: {} - {:?}",
                test_state.current_step,
                test_state.current_step_name(),
                test_state.current_step_status()
            );
        }
    }

    // Generate test report
    test_state.generate_report();
}
```

### Test Data Collection
```rust
struct TestMetrics {
    step_durations: Vec<Duration>,
    network_latencies: Vec<f32>,
    frame_rates: Vec<f32>,
    memory_usage: Vec<usize>,
    packet_loss: f32,
    total_duration: Duration,
}

impl TestState {
    fn generate_report(&self) -> TestReport {
        TestReport {
            timestamp: Utc::now(),
            total_steps: self.steps.len(),
            passed_steps: self.steps.iter().filter(|s| matches!(s.status, TestStatus::Passed)).count(),
            failed_steps: self.steps.iter().filter(|s| matches!(s.status, TestStatus::Failed(_))).count(),
            metrics: self.metrics.clone(),
            failures: self.collect_failures(),
        }
    }
}
```

## Development Setup for WebTransport

### Local Development Requirements
1. **TLS Certificates**: WebTransport requires HTTPS even for localhost
   ```bash
   # Generate self-signed certificate for local development
   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
     -subj "/CN=localhost"
   ```

2. **Browser Configuration**:
   - Chrome/Edge: Navigate to `chrome://flags/#allow-insecure-localhost`
   - Enable "Allow invalid certificates for resources loaded from localhost"

3. **Server Port**: Use port 443 (standard HTTPS) or configure custom port

### Production Requirements
- Valid TLS certificate (Let's Encrypt recommended)
- Domain name with HTTPS enabled
- Firewall rules allowing UDP traffic on port 443
- CDN compatibility check (some CDNs don't support WebTransport yet)

## Next Steps

### Immediate Priority: E2E Testing Framework
1. **Set up test infrastructure**:
   - Add kittest dependency for input automation
   - Create test runner with tiled client display
   - Implement test step system with egui UI

2. **Implement basic multiplayer**:
   - Add lightyear 0.24.2 with WebTransport
   - Create minimal server with room management
   - Implement game code-based authentication
   - Basic player movement and synchronization

3. **Create E2E test scenarios**:
   - Basic connection test (2 clients)
   - Room capacity test (up to 32 clients)
   - Persistence test (1 week timeout)
   - Creator control test (room closure)

### Implementation Schedule
- **Week 1**:
  - E2E test framework setup
  - Basic networking with Lightyear
  - Simple room creation/join

- **Week 2**:
  - Complete E2E test scenarios
  - Room persistence (1 week timeout)
  - 32 player support

- **Week 3**:
  - Run automatic tests until stable
  - Execute semi-automatic tests for validation
  - Polish and bug fixes

### Test-Driven Development Approach
1. **Build test harness first** - Multiple clients in tiled view
2. **Implement minimal server** - Just enough for tests
3. **Run automatic tests** - Iterate until all pass
4. **Run semi-automatic** - Manual inspection and validation
5. **Refine based on results** - Fix issues found during testing
   