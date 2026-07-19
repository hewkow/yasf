# ADR-003: Node Canvas Graph Propagation for Equipment Chains

## Status
Accepted

## Date
2026-07-19

## Context
MapleStory progression often involves cascading equipment upgrades (e.g., enhancing a fodder item to 21/22 stars, transferring stars via Transfer Hammer, and continuing enhancement on a target item). Players need to simulate and visualize full gear progression chains on a visual canvas graph.

Key Requirements:
- Represent equipment as nodes (`CanvasItem`) with attributes (start star, target star, level, safeguard, event toggles).
- Represent transfer dependencies as directed edges (`connections`).
- Propagate simulation results sequentially or hierarchically along connected paths.

## Decision
Use an in-memory graph representation with explicit topology propagation in `propagate_canvas_items`.

- Node model (`CanvasItem`): Encapsulates item metadata, enhancement configuration, position on canvas, and calculated simulation results (`SimResult`).
- Edge model (`connections`): Stores index pairs `(source_index, target_index)` representing transfer/dependency flows.
- Propagation algorithm: Sequentially updates target node starting parameters based on source node outcomes (e.g., target starting star derived from source final star minus transfer loss).

## Alternatives Considered

### Independent Standalone Simulators
- Pros: Simpler architecture.
- Cons: Cannot model multi-gear progression paths or Transfer Hammer cost aggregation.
- Rejected: Fails user requirement for canvas-based visual workflow planning.

### Full Graph Database / Complex DAG Library (e.g., petgraph)
- Pros: Standardized graph traversal algorithms.
- Cons: Additional crate dependency overhead for small WASM bundle size; simple index-based adjacency array suffices for visual canvas chains.
- Rejected: Unnecessary dependency weight for lightweight WASM app.

## Consequences
- Clean UI integration with node manipulation controls in Dioxus.
- Linear execution path for canvas propagation that stays deterministic and easily testable.
