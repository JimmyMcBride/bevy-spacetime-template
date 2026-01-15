# Build & Deployment Guide

## Prerequisites

### Required Tools
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# SpacetimeDB CLI
curl -sSf https://install.spacetimedb.com | sh
```

### Verify Installation
```bash
rustc --version          # Should show 1.70+
spacetime version        # Should show 1.0+
rustup target list | grep wasm32-unknown-unknown  # Should show "installed"
```

## Building the Server Module

### Debug Build
```bash
cd server
cargo build --target=wasm32-unknown-unknown
```

### Release Build (Recommended)
```bash
cd server
cargo build --target=wasm32-unknown-unknown --release
```

The WASM file will be at:
- Debug: `target/wasm32-unknown-unknown/debug/server.wasm`
- Release: `target/wasm32-unknown-unknown/release/server.wasm`

## Publishing to SpacetimeDB

### First Time (Create New Database)
```bash
cd server
spacetime publish YOUR_DB_NAME --server maincloud
```

### Update Existing Database
```bash
cd server
spacetime publish YOUR_DB_NAME --server maincloud
```

### Local Development Server
```bash
# Start local SpacetimeDB server
spacetime start

# Publish to local server
spacetime publish YOUR_DB_NAME --server http://localhost:3000
```

## Generating Client Bindings

After any server schema changes, regenerate client bindings:

```bash
cd server
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```

This creates the `module_bindings/` directory with:
- Type definitions for all tables
- Reducer function stubs
- Connection builder

## Monitoring & Logs

### View Logs
```bash
spacetime logs YOUR_DB_NAME --server maincloud
```

### Follow Logs (Live)
```bash
spacetime logs YOUR_DB_NAME --server maincloud -f
```

### Database Status
```bash
spacetime sql YOUR_DB_NAME --server maincloud "SELECT * FROM player"
```

## Environment Options

| Server | URI | Use Case |
|--------|-----|----------|
| maincloud | `https://maincloud.spacetimedb.com` | Production |
| testnet | `https://testnet.spacetimedb.com` | Testing |
| local | `http://localhost:3000` | Development |

## Troubleshooting

### "Module not found"
- Verify database name matches in publish and client
- Check `spacetime list --server maincloud` for your databases

### "WASM compilation failed"
- Ensure `wasm32-unknown-unknown` target is installed
- Check for Rust syntax errors with `cargo check`

### "Reducer failed"
- Check logs: `spacetime logs DB_NAME --server maincloud`
- Verify reducer signature matches client calls

### "Schema migration failed"
- For breaking changes, may need to delete and recreate database
- Use `spacetime delete YOUR_DB_NAME --server maincloud` (WARNING: destroys data)

## Deployment Checklist

1. Build release WASM
2. Test locally with local SpacetimeDB
3. Publish to testnet for staging
4. Regenerate client bindings
5. Test client against testnet
6. Publish to maincloud for production
7. Update client connection URI
