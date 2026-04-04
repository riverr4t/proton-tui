# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```bash
# Build and run
cargo run --release

# Install globally
cargo install --path .

# Check code (fast compilation check)
cargo check

# Format code (required by CI)
cargo fmt

# Lint code (CI fails on warnings)
cargo clippy -- -D warnings
```

## Architecture Overview

Proton TUI is a Rust terminal application for connecting to ProtonVPN via WireGuard. It uses async/await throughout with Tokio runtime.

### Core Modules

**Authentication (`auth.rs`)** - Implements SRP-6a (Secure Remote Password) protocol for ProtonVPN authentication. Handles the multi-step flow: fetch SRP parameters, verify PGP signature on modulus, compute client proofs, submit authentication, and handle 2FA if required.

**API Client (`api.rs`)** - `ProtonClient` wraps reqwest for ProtonVPN API calls. Main endpoints: `/vpn/logicals` (server list), `/vpn/v1/certificate` (WireGuard cert registration).

**Token Management (`tokens.rs`)** - Persists auth tokens to `~/.config/proton-tui/tokens.json` with 0600 permissions.

**WireGuard (`wireguard.rs`)** - Generates WireGuard config files. Uses Ed25519 key generation with X25519 derivation for the WireGuard key exchange.

### Application State (`app/`)

- `mod.rs` - Main `App` struct holding server list, UI state, connection status
- `state.rs` - Enums: `InputMode`, `SplitFocus`, `DisplayItem`, `ConnectionStatus`
- `connection.rs` - WireGuard management via `sudo wg-quick up/down`, traffic stats from `/proc/net/dev`
- `navigation.rs` - Tree view navigation (expand/collapse countries, selection)
- `search.rs` - Incremental server filtering with Emacs-style keybindings
- `split_view.rs` - Dual-pane view logic (countries left, servers right)

### UI Layer (`ui/mod.rs`)

Ratatui-based rendering with two view modes:
- **Tree View**: Hierarchical country → server list with expandable sections
- **Split View**: Dual-pane with country list (35%) and server list (65%)

### Event Loop (`main.rs`)

1. Initialize crossterm terminal
2. Load tokens or show login form
3. Fetch server list
4. Poll keyboard events (100ms), update state, render with ratatui
5. TUI suspends/resumes around sudo prompts for WireGuard operations

### Key Technical Details

- Interface name: `proton-tui0`
- Config path: `$XDG_RUNTIME_DIR/proton-tui/proton-tui0.conf` (falls back to `/tmp/proton-tui/proton-tui0.conf`; mode 600, on tmpfs so private keys don't persist to disk)
- Install binary: `cargo install --path .` (installs to `~/.cargo/bin/`) or copy `target/release/proton-tui` to a dir on the session PATH (e.g. `~/.local/share/omarchy/bin/` on Omarchy)
- Linux only (requires `wg-quick`)
- Requires sudo for VPN operations
- DNS is not modified (relies on system DNS)

## Conventional Commits

All commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `refactor` - Code change that neither fixes a bug nor adds a feature
- `test` - Adding or updating tests
- `chore` - Maintenance tasks (deps, config)
- `ci` - CI/CD changes

**Scopes** (optional): `ui`, `app`, `auth`, `api`, `wireguard`

**Breaking changes:** Add `!` after type/scope and include `BREAKING CHANGE:` footer.

**Examples:**
```
feat(ui): add split view mode for server selection
fix(auth): handle expired refresh tokens correctly
docs: update architecture overview
refactor(app)!: restructure connection state handling

BREAKING CHANGE: ConnectionStatus enum variants renamed
```

## Dependencies

Key crates: `ratatui` (TUI), `crossterm` (terminal), `reqwest` (HTTP), `tokio` (async), `ed25519-dalek`/`sha2`/`bcrypt`/`num-bigint` (SRP crypto).
