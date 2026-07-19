# Starforce Canvas (Hot DOF)

A high-performance, interactive MapleStory Starforce Enhancement Simulator & Node Graph Visualizer built in Rust using **Dioxus 0.7** and WebAssembly (WASM).

## Overview

**Starforce Canvas** enables players and theorycrafters to model, simulate, and analyze complex equipment starforcing scenarios. From individual item enhancement calculations to multi-equipment Transfer Hammer chain flows on a visual node canvas, the application provides real-time Monte Carlo statistics, cost histograms, boom probability heatmaps, and per-star friction analysis.

## Key Features

- **Fast Monte Carlo Engine**: Run millions of simulation trials in milliseconds using binned histogram metrics (`FastMetrics`) and high-throughput RNG (`SmallRng`).
- **Flexible Enhancement Rules**: Supports standard KMS rates alongside custom 15–21 star enhancement levels (Level 1 to Level 4).
- **Event & Safeguard Toggles**: Seamlessly test Safeguard, Star Catch (+5% success with adjusted boom formula), and Shining Star Force (SSF) cost (-30%) and boom (-30%) events.
- **Visual Node Canvas**: Drag-and-drop node canvas allowing equipment dependency chaining and transfer hammer propagation.
- **Detailed Statistical Breakdown**: Percentile estimations (50th, 75th, 90th, 95th, 99th), cost distributions, and per-star friction analytics (mesos lost & booms per star level).

## Quick Start

### Prerequisites

- **Rust toolchain** (latest stable recommended) with target `wasm32-unknown-unknown`.
- **Dioxus CLI** (`dx`):
  ```sh
  curl -sSL http://dioxus.dev/install.sh | sh
  # or via cargo:
  cargo install dioxus-cli
  ```

### Development Server

Run the development server targeting the web:

```sh
dx serve
```

The application will compile to WebAssembly and launch in your local browser (default: `http://localhost:8080`).

### Desktop / Mobile Target (Optional)

```sh
dx serve --platform desktop
```

## Commands

| Command | Description |
|---------|-------------|
| `dx serve` | Start development server with hot-reloading for Web |
| `dx serve --platform desktop` | Launch native desktop application |
| `cargo check` | Check code for compilation errors |
| `cargo test` | Run test suite |
| `cargo clippy` | Run Rust linter for code style and performance checks |
| `cargo build --release` | Build optimized release package |

## Architecture & Design Decisions

The application is structured into two core modules:
1. **Simulation Logic (`src/starforce.rs` & `src/lib.rs`)**: Standalone Rust module containing KMS cost formulas, rate tables, event logic, and Monte Carlo histogram collectors (`FastMetrics`).
2. **UI & Canvas System (`src/main.rs` & `src/components/`)**: Built using Dioxus 0.7 RSX macro syntax, Signal state primitives (`use_signal`, `use_memo`), and custom canvas layout components.

### Architectural Decision Records (ADRs)

Key architectural choices are documented in [`docs/decisions/`](file:///D:/Projects/starforce-canvas/docs/decisions/):

- [ADR-001: Choice of Dioxus 0.7 for Frontend Architecture](file:///D:/Projects/starforce-canvas/docs/decisions/ADR-001-dioxus-web-framework.md)
- [ADR-002: Monte Carlo Simulation Engine and Binned Metrics Storage](file:///D:/Projects/starforce-canvas/docs/decisions/ADR-002-starforce-simulation-engine.md)
- [ADR-003: Node Canvas Graph Propagation for Equipment Chains](file:///D:/Projects/starforce-canvas/docs/decisions/ADR-003-canvas-node-propagation.md)

## Project Structure

```
starforce-canvas/
├── assets/             # Static assets (images, icons)
├── docs/
│   └── decisions/      # Architecture Decision Records (ADRs)
├── src/
│   ├── components/     # UI components (buttons, canvas nodes)
│   ├── lib.rs          # Module declarations
│   ├── main.rs         # Root Dioxus application & canvas pages
│   └── starforce.rs    # Core enhancement simulation & cost calculations
├── Cargo.toml          # Rust package dependencies & feature flags
├── Dioxus.toml         # Dioxus project configuration
└── tailwind.css        # Styling configuration
```

## Contributing

1. Fork or clone the repository.
2. Ensure `cargo clippy` passes without warnings.
3. Follow the Dioxus 0.7 state management conventions (prefer Signals over legacy `use_state`).
4. Write ADRs in `docs/decisions/` for significant architectural modifications.
