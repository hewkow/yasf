# ADR-001: Choice of Dioxus 0.7 for Frontend Architecture

## Status
Accepted

## Date
2026-07-19

## Context
Starforce Canvas requires an interactive, high-performance web interface capable of rendering complex node graphs, live simulation charts, and handling millions of Monte Carlo iterations without UI thread freezing. 

Key Requirements:
- High performance for compute-intensive probabilistic simulations.
- Modern reactive UI component architecture with fine-grained reactivity.
- Ability to target WebAssembly (WASM) directly from Rust.
- Seamless single-language codebase (Rust for simulation core and UI components).

## Decision
Use **Dioxus 0.7.1** targeting `web` (WASM).

## Alternatives Considered

### Yew / Leptos
- Pros: Rust web frameworks with good WASM support.
- Cons: Leptos uses fine-grained signal signals with a different JSX-like syntax; Dioxus 0.7 provides updated signal primitives (`Signal<T>`, `use_signal`, `use_memo`), clean RSX macro syntax, and explicit platform feature targeting (`web`, `desktop`).
- Rejected: Dioxus 0.7 offers superior RSX ergonomics and component lifecycle for node canvas manipulation.

### React / TypeScript Frontend + Rust WASM module
- Pros: Ecosystem maturity for standard web widgets.
- Cons: Overhead of JS-WASM bridge serialization across borders when streaming simulation metrics back to the UI canvas. Double language maintenance overhead.
- Rejected: Pure Rust WASM with Dioxus eliminates JS-WASM serialization friction and keeps state local to Rust signals.

## Consequences
- Single language stack (Rust) for both engine logic (`starforce.rs`) and UI components (`main.rs`, `components/`).
- Must strictly adhere to Dioxus 0.7 API conventions (signals replacing `cx`/`use_state`).
- Fast rendering and native reactivity with minimum runtime overhead.
