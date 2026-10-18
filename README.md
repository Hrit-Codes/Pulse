# Pulse

**Peer-to-peer file sharing for devices on the same local network.**

Pulse is a cross-platform desktop application that discovers nearby devices, verifies them with a one-time PIN, and transfers files directly between them. No accounts, no cloud relay, no third-party server — the router carries the packets but never participates in the application, and no data is stored anywhere except on the devices themselves.

> **Status:** Alpha · v0.1.0 — Device discovery, PIN-verified transfers, resume, and the desktop UI are functional.

## Features

### Implemented

- **Peer discovery** — automatic UDP broadcast on the LAN; nearby devices appear without manual IP entry
- **PIN-verified transfers** — the sender displays a 4-digit PIN that the receiver must enter before any data is exchanged
- **Chunked transfer** — files are split into 1 MiB chunks with per-chunk acknowledgements, enabling accurate progress reporting and partial retries
- **Integrity verification** — SHA-256 is computed independently by both sender and receiver and compared on completion
- **Resume** — interrupted transfers pick up from the last confirmed chunk using persisted state
- **Transfer session UI** — live progress, speed, direction, and status; paused or failed transfers can be resumed from the UI
- **Themes** — light, dark, and system modes, persisted across restarts
- **Self-test** — a loopback transfer to `127.0.0.1` validates the full pipeline on a single machine
- **Command-line interface** — the core networking stack is also available as a standalone binary for headless use and testing

## Tech stack

| Component     | Technology                                                          |
| ------------- | ------------------------------------------------------------------- |
| Frontend      | React 19, TypeScript, Tailwind CSS 4, Vite 7                        |
| Desktop shell | Tauri v2                                                            |
| Backend       | Rust — Tokio, rusqlite (SQLite), sha2, bincode, clap                |
| Transport     | UDP for discovery (port 45820) · TCP for transfer (ports 9000–9001) |

## Getting started

### Prerequisites

- [Node.js](https://nodejs.org/) 20 or later (LTS recommended)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

### Installation

```bash
git clone https://github.com/Hrit-Codes/Pulse.git
cd Pulse
npm install
```

### Run in development

```bash
npm run tauri dev
```

The first run compiles the Rust backend and may take a few minutes. The frontend dev server runs on port 1420.

> **Note:** running `cargo run` directly from `src-tauri/` opens a blank window, because the desktop shell expects the Vite dev server to be available. Always start the application with `npm run tauri dev`.

### Production build

```bash
npm run tauri build
```

## Command-line interface

The core stack can be operated without the graphical interface:

```bash
cd src-tauri

# Discover peers on the LAN and print the results
cargo run --bin cli -- discover

# Listen for discovery broadcasts from other peers
cargo run --bin cli -- listen-discover

# Send a file to a peer
cargo run --bin cli -- send 192.168.1.24 --file ./report.pdf

# Accept incoming transfers (default port 9000)
cargo run --bin cli -- receive

# Run the resume listener (port 9001)
cargo run --bin cli -- resend
```

## Architecture

Three layers, bridged by Tauri's IPC:

| Layer            | Technology         | Responsibility                                                      |
| ---------------- | ------------------ | ------------------------------------------------------------------- |
| Presentation     | React + TypeScript | UI state, rendering, and events. No networking or filesystem logic. |
| Application core | Rust (Tokio)       | Discovery, protocol handling, transfer management, persistence.     |
| Network          | UDP + TCP          | UDP broadcast to find peers; TCP for all file and data transfer.    |

The UI calls Rust commands (`send_file`, `list_devices`, `resume`, …) via `invoke()` and receives events back (`devices_changed`, `pin_required`, …) over Tauri IPC.

### Project structure

```
src/                            # React frontend
├── App.tsx                     # Shell, page switching, transfer state
├── api.ts                      # Typed wrappers over Tauri invoke/listen
├── components/                 # Pages and UI components
├── context/ThemeContext.tsx    # Theme management, persisted to disk
└── hooks/useDevices.ts         # Peer discovery lifecycle

src-tauri/src/                  # Rust backend
├── lib.rs                      # Tauri builder, shared state, command handlers
├── protocol/                   # Message framing and message types
├── discover/                   # UDP broadcaster and listener
├── transfer/                   # Sender, receiver, resume
├── storage/                    # SQLite persistence layer
└── bin/cli.rs                  # Standalone CLI binary
```

### Wire protocol

1. **Discovery (UDP, port 45820)** — a peer broadcasts `Discover`; responders reply with device ID, name, and port, building a local peer registry.
2. **Connection (TCP, port 9000)** — the sender opens a connection to the chosen peer.
3. **Framing** — every message is `length + type + payload`, with bincode-encoded payloads. Message types include `Discover`, `TransferRequest`, `FileMetadata`, `Chunk`, `ChunkAck`, `FileHash`, `ResumeRequest`, and `RequestPin`.
4. **PIN handshake** — the sender generates a 4-digit PIN; the receiver must submit it before any file data is transferred. A mismatch rejects the transfer.
5. **Chunked transfer** — files are split into 1 MiB chunks; the receiver acknowledges each chunk as it arrives.
6. **Verification** — both sides compute SHA-256 over the complete file and compare the hashes.
7. **Resume** — after an interruption, the receiver reports which chunks it already holds and the sender re-sends only what is missing.

### Persistence

Each device maintains its own local SQLite database (`pulse.db` in the platform data directory), with partial chunk data stored alongside it. No shared or central database exists.

| Table             | Contents                                                           |
| ----------------- | ------------------------------------------------------------------ |
| `device_identity` | Local device ID and name (created on first run)                    |
| `transfers`       | Transfer metadata and status (`in_progress`, `complete`, `failed`) |
| `chunks`          | Per-chunk progress used for resume                                 |

## Security

- **PIN approval (implemented):** no transfer begins until the receiver enters the sender's 4-digit PIN.
- **Integrity (implemented):** SHA-256 verification detects corrupted or truncated files.

## Design principles

- Get **one file, one peer, correctly transferred and verified** before adding resume, sync, or concurrency.
- Keep the transport layer (TCP) and the application protocol (chunks, resume, sync) cleanly separated.
- Make all transfer state explicit and persisted in SQLite, never implicit in memory.
- Prefer correctness over performance optimization early on.
- Prove core logic through the CLI first; then expose it through Tauri commands.

## Roadmap

| Phase | Focus                                                                | Status      |
| ----- | -------------------------------------------------------------------- | ----------- |
| 0     | Project scaffold (Tauri + React + TypeScript)                        | Complete    |
| 1     | Networking foundations — UDP discovery, framing, protocol v1         | Complete    |
| 2     | File transfer — chunking, SHA-256 verification, resume, PIN approval | Complete    |
| 3     | Desktop UI — devices, transfers, settings, themes                    | Complete    |
| 4     | Security — TLS transport encryption, trusted-device pairing          | Not started |
| 5     | Folder sync — filesystem watcher, reconciliation, conflict handling  | Not started |
| 6     | Benchmarking — throughput, resume correctness, resource usage        | Not started |

## Known limitations

- Transport is not yet encrypted; TLS is planned (phase 4).
- The _History_ navigation entry does not have a page yet.
- The discovery toggle in Settings is not connected to the backend.
- Folder sync and trusted-device management are not implemented.
- The CLI `resume` and `change-name` subcommands still contain development placeholders.

## Out of scope (for now)

Deferred as future extensions, not part of the MVP:

- Wi-Fi Direct (for networks without a shared LAN)
- Multi-peer fan-out transfers
- Parallel chunk streams
- Bandwidth throttling
- Transfer queues
