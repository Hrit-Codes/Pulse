# Pulse

Peer-to-peer file sharing and folder sync for local networks. No cloud, no central server — devices on the same LAN discover each other, pair once, and transfer or sync files directly.

> **Status:** Early development. The Tauri + React scaffold is in place; core networking and transfer logic is not yet implemented. This README describes the project vision and planned architecture alongside what exists today.

## Goals

- **Discover** other Pulse instances on the local network automatically (UDP broadcast)
- **Pair** devices with a one-time verification code — no accounts, no server-issued credentials
- **Send files** directly between paired devices with progress tracking, chunked transfer, and resume-on-failure
- **Sync folders** continuously between paired devices with conflict-safe handling (never silently overwrites)
- **Encrypt everything** in transit via TLS; nothing passes through a third-party server

## Why no server

The router carries the packets — that's the physical network. But it never acts as an application server. It doesn't store files, doesn't know about transfers, doesn't hold any account data. Peers talk directly to each other at the application level.

## Current state

The project is scaffolded with Tauri v2, React 19, TypeScript, and Vite. The Rust backend has the Tauri builder wired up with placeholder commands. No domain logic (networking, transfer, sync, persistence) has been implemented yet.

```
src-tauri/src/
├── main.rs       # Entry point — calls lib.rs
└── lib.rs        # Tauri builder + command registration

src/
├── main.tsx      # React root
├── App.tsx       # Scaffold UI
└── App.css
```

## Planned architecture

Three layers, bridged by Tauri IPC:

| Layer | Tech | Responsibility |
|---|---|---|
| Presentation | React + TypeScript | UI — device list, transfer progress, sync status, settings. No networking or filesystem logic. |
| Application core | Rust (Tokio, SQLite) | Discovery, protocol handling, transfer management, sync logic, security, persistence. |
| Network | UDP + TCP/TLS | UDP broadcast to find peers; TCP+TLS for all file/data transfer. |

The UI will call Rust functions (`send_files()`, `get_devices()`, etc.) and Rust will push events back (progress updates, new peers found) over Tauri's IPC.

### Planned project layout

```
src-tauri/src/
├── main.rs
├── lib.rs
├── error.rs                # Error types
├── bin/
│   └── cli.rs              # Standalone CLI for testing without the GUI
├── protocol.rs             # Message framing + MessageType definitions
├── protocol/
│   └── frame.rs
├── discovery.rs            # UDP peer discovery
├── discovery/
│   ├── broadcaster.rs
│   └── listener.rs
├── transfer.rs             # Chunked file transfer + verification
├── transfer/
│   ├── sender.rs
│   └── receiver.rs
├── storage.rs              # SQLite persistence layer
└── storage/
    └── schema.rs
```

`protocol`, `discovery`, `transfer`, and `storage` will be pure Rust library modules — shared by both `main.rs` (the Tauri app) and `bin/cli.rs` (a standalone terminal tool for developing and testing core logic without the GUI).

## How a transfer will work

1. **Discovery (UDP)** — a peer broadcasts `DISCOVER`; others on the LAN reply with device ID, name, IP, port, and capabilities. Builds a local peer registry.
2. **Connection (TCP + TLS)** — a TCP connection is opened to the chosen peer and upgraded to TLS.
3. **Custom protocol** — a message protocol runs on top of the TCP stream. Each message is framed as `length + type + payload`. Message types include `TRANSFER_REQUEST`, `FILE_METADATA`, `CHUNK`, `CHUNK_ACK`, `SYNC_UPDATE`.
4. **Chunked transfer** — files are split into fixed-size chunks (e.g. 1 MB). This enables progress bars, partial retries, and resume.
5. **Verification** — the receiver computes a SHA-256 hash of the reassembled file and compares it against the sender's hash.
6. **Resume on failure** — the receiver reports which chunks it already has; the sender re-sends only what's missing. Transfer state is persisted in SQLite.

## Folder sync

A separate, continuous system (transfers are user-initiated; sync runs in the background):

- A filesystem watcher detects create/modify/delete/rename events in a shared folder
- Local metadata (hash, version, timestamp, origin device) is diffed against the peer's manifest
  - **Local changed only** → upload
  - **Remote changed only** → download
  - **Both changed** → conflict

**Conflict policy:** never silently destroy data. On conflict, both versions are kept (e.g. `notes.md` and `notes.conflict-deviceA.md`) and the user resolves it manually.

## Security model

- Discovery does not imply trust. Appearing on the LAN via UDP broadcast grants no permissions.
- Devices must be explicitly **paired** before any transfer or sync.
- Pairing shows a short verification code on both devices, derived from the TLS handshake — the user visually confirms both codes match (similar to Bluetooth pairing or Signal safety numbers).
- Once paired, device identity is stored locally and trusted automatically for future connections.
- All data in transit is encrypted via TLS.

## Local storage

No shared or central database. Each peer maintains its own local SQLite file tracking:

- Known devices and pairings
- Transfer state and per-chunk progress
- Sync roots and synced file metadata

## Development

**Prerequisites:** [Node.js](https://nodejs.org/), [Rust](https://www.rust-lang.org/tools/install), and the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
# Install frontend dependencies
npm install

# Run the full app (frontend + backend)
npm run tauri dev
```

> Note: running `cargo run` directly from `src-tauri/` will show a blank window because the Rust shell expects the Vite dev server to be available. Always use `npm run tauri dev` during development.

Once the CLI tool is implemented:

```bash
cd src-tauri
cargo test
cargo run --bin cli -- discover       # broadcast and print responding peers
cargo run --bin cli -- listen         # listen for discovery + incoming transfers
cargo run --bin cli -- send <ip> --file <path>
```

## Build philosophy

- Get **one file, one peer, correctly transferred and verified** before adding resume, sync, or concurrency.
- Keep the transport layer (TCP) and the application protocol (files/chunks/sync) cleanly separated.
- Make all state explicit and persisted (SQLite), not implicit in memory.
- Prefer correctness over performance optimization early on.
- Build and test core logic via the CLI first; wire into Tauri commands once proven.

## Roadmap

| Phase | Focus | Status |
|---|---|---|
| 0 | Project scaffold (Tauri + React + TypeScript) | Done |
| 1 | Networking foundations — TCP/UDP, discovery, protocol v1 | Not started |
| 2 | File transfer — chunking, SHA-256 verification, resume, cancellation | Not started |
| 3 | Folder sync — filesystem watcher, reconciliation, conflict handling | Not started |
| 4 | Security — pairing, TLS, benchmarking (throughput, resume correctness, resource usage) | Not started |

## Out of scope (for now)

Deferred as future extensions, not part of the MVP:

- Wi-Fi Direct (for when there's no shared LAN)
- Multi-peer fan-out transfers
- Parallel chunk streams
- Bandwidth throttling
- Transfer queues

## Tech stack

**Current:**
- **Frontend:** React 19, TypeScript, Vite 7
- **Backend:** Rust (edition 2021)
- **Bridge:** Tauri v2

**Planned additions:**
- Tokio (async runtime)
- SQLite via `sqlx` (persistence)
- `sha2` (SHA-256 file verification)
- `clap` (CLI)
- `rustls` or native TLS (encrypted transport)
- Tailwind CSS (styling)
