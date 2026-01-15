# Bevy + SpacetimeDB Multiplayer Game - Documentation

> **Token Conservation**: Only read guides from `ClaudeInstructions/` folders relevant to your current task.

## Project Overview

Multiplayer game with server-authoritative architecture.

**Tech Stack:**
- Game Client: Rust + Bevy 0.15
- Backend: Rust + SpacetimeDB 1.0 (WebAssembly module)
- Database: SpacetimeDB (maincloud hosted)

## Quick Start Commands

### Server
```bash
cd server
cargo build --target=wasm32-unknown-unknown --release
spacetime publish YOUR_DB_NAME --server maincloud
```

### Client
```bash
cd client
cargo run                    # Debug
cargo build --release        # Release
```

### Regenerate Bindings
```bash
cd server
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```

## Data Flow

```
Client Input (WASD) --> Reducer (update_player_input) --> Server Tick (20Hz)
    --> EntityPosition Update --> Client Sync --> Interpolation (60fps)
```

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `client/` | Bevy game client |
| `server/` | SpacetimeDB module |
| `client/src/systems/` | ECS systems |
| `client/src/module_bindings/` | Auto-generated (do not edit) |

## Navigate to Documentation

- **[Client Documentation](client/CLAUDE.md)** - Bevy, ECS, rendering, input
- **[Server Documentation](server/CLAUDE.md)** - SpacetimeDB, tables, reducers

## Guide Selection Helper

| Task | Go To |
|------|-------|
| Add ECS component | client/CLAUDE.md |
| Add database table | server/CLAUDE.md |
| Add reducer | server/CLAUDE.md |
| Fix network sync | client/ClaudeInstructions/client-networking-guide.md |
| Deploy to cloud | server/ClaudeInstructions/build-deployment-guide.md |
