# ADR-002: Monte Carlo Simulation Engine and Binned Metrics Storage

## Status
Accepted

## Date
2026-07-19

## Context
MapleStory Starforce enhancement features probabilistic outcomes (success, failure, destruction/boom) with cascading star drops upon failure. Calculating exact analytical probability distributions across multi-stage enhancements (especially with safeguards, Star Catch, and non-standard enhancement levels 15-21) becomes combinatorially complex.

Monte Carlo simulation allows evaluating millions of trials in milliseconds. However:
- Storing individual trial costs in raw arrays requires hundreds of megabytes of memory per run (e.g. 10 million `u64` values = 80 MB).
- Dynamic vector reallocation during rapid simulation loops introduces memory fragmentation and allocation latency.

## Decision
Implement a binned metrics collector (`FastMetrics`) utilizing `rand::rngs::SmallRng` for random number generation and fixed binning with `BIN_SIZE = 100_000_000` (100M mesos per bin).

Key implementation choices:
1. `SmallRng`: High throughput non-cryptographic PRNG suitable for simulation runs.
2. Binned Histograms: `cost_histogram` pre-allocated with 10,000 bins (covering up to 1 Trillion mesos) to prevent reallocations.
3. Joint Histograms: `joint_histogram` tracks joint probability of cost bin and boom count (`[u32; 100]`).
4. Per-star friction tracking: Measures accumulated cost, boom count, and total attempts spent at each specific star rank (0–30).

## Alternatives Considered

### Storing Raw Trial Arrays (`Vec<u64>`)
- Pros: Simple to sort and extract exact percentiles.
- Cons: High memory bandwidth usage; sorting 1M+ elements per run causes noticeable frame drops on the WASM main thread.
- Rejected: Sorting large vectors is too expensive for interactive UI updates.

### Analytical Markov Chains / Transition Matrices
- Pros: Exact mathematical solution without variance.
- Cons: Dynamic rules (e.g., custom per-star level modes 15-21, conditional safeguard, SSF event modifiers) require dynamic matrix recalculation and inversion, making interactive parameter tweaking complex and less flexible than Monte Carlo.
- Rejected: Monte Carlo with binned sampling provides sub-percentile accuracy with arbitrary rule flexibility.

## Consequences
- Percentiles (e.g., median, 75th, 90th, 95th, 99th) are estimated via cumulative bin counts using `find_percentile`, yielding ultra-fast percentile computation ($O(\text{bins})$).
- Near zero memory allocation overhead during simulation loops.
