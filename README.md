# HFT Simulator

A multi-threaded, lock-free high-frequency trading simulator written in Rust. The project models a realistic financial market with a central exchange, multiple types of market participants, and a live price visualization frontend.

**Author:** npersily3  
**Contact:** nrpersily@gmail.com

---

## Project Goals

This project is a learning sandbox for several interconnected topics:

- **Rust** — systems programming, ownership, concurrency primitives, lock-free atomics, and high-performance data structures
- **Market microstructure and HFT** — limit order books, bid-ask spreads, market vs. limit orders, price discovery, noise traders vs. fundamental traders
- **GPU computing** *(planned)* — offloading simulation workloads to the GPU for massively parallel participant modeling
- **Cloud and distributed systems** *(in progress)* — containerizing each layer with Docker and Docker Compose, with a path toward running components across separate machines

---

## Architecture: Three Layers

The simulation is organized into three distinct layers, each running in its own thread(s) and communicating via lock-free channels.

### Layer 1 — Exchange (`src/exchange.rs`)

The lowest and most performance-critical layer. This is the central matching engine.

- Maintains a **limit order book** implemented as a flat price-indexed array of bid/ask queues (`Book`)
- Processes incoming `MarketOrder`s from a crossbeam channel
- Matches bids against asks and executes trades atomically using `Arc<AtomicU64>` money balances — no locks required
- Supports market orders and limit orders
- In `gui` mode, emits a `TICK:` line per tick with best bid, best ask, mid price, and true price — consumed by the visualizer

### Layer 2 — Market (`src/market.rs` + `src/corporation.rs`)

The middle layer simulates the population of traders and the underlying asset.

**Noise traders** (`market::noise`) — random participants that submit market orders with log-normally distributed quantities and exponentially distributed inter-arrival times. They represent irrational or uninformed flow.

**Fundamentalist traders** (`market::fundamentalist`) — informed participants that observe the current mid-price and the true asset value. When the spread between them exceeds a threshold, they submit limit orders at the true price, pushing the market back toward fair value.

**Corporation** (`corporation::set_true_price`) — models the underlying asset's true value as a random walk, stepping up or down each tick. This gives fundamentalists a moving target to track and creates realistic price dynamics.

### Layer 3 — Personal / Strategy (`src/main.rs`, `NUM_PERSONAL_THREADS`)

The top layer is reserved for user-defined trading strategies — the "you" in the simulation. Currently empty (`NUM_PERSONAL_THREADS = 0`), this is where a personal HFT strategy would run: reacting to order book state, submitting and canceling orders, and competing against the noise and fundamentalist bots. This is the primary area for future development.

---

## Concurrency Model

All threads synchronize through a `TickBarrier` (`src/utils.rs`). Each tick:

1. All trader threads (noise + fundamental + personal) submit their orders and then wait on the barrier
2. The exchange thread detects that all traders have checked in, drains the order queue, updates the book, and emits market data
3. The barrier releases everyone for the next tick

This gives the simulation a discrete, reproducible time step while still using real OS threads.

Two feature flags control timing:
- `tick` (default) — barrier-synchronized discrete ticks
- `time` — real-time sleeping between steps

---

## Visualization

`frontend/visualizer.py` is a Python/matplotlib frontend that:

1. Compiles the Rust binary with the `gui` feature
2. Spawns it as a subprocess and reads its stdout
3. Plots mid price, true price, bid-ask band, and spread in a live-updating window with pan and zoom sliders

Run it directly:
```bash
python frontend/visualizer.py
```

Requires: `matplotlib`

---

## Running


**Headless with Tick (no GUI output):**
```bash
cargo run --features tick
```

**Headless with Real Time (no GUI output):**
```bash
cargo run --features time
```


**With Docker (in progress, do not run):**
```bash
docker compose up --build
```

The `Dockerfile` uses a multi-stage Alpine build for a minimal release image. The `frontend/` directory has its own `Dockerfile` and `compose.yaml` for the Python visualizer.

---

## Roadmap

- [ ] Personal strategy layer (Layer 3) — implement a basic market-making or momentum strategy
- [ ] Order cancellation — allow traders to cancel resting orders
- [ ] Trade history — re-enable the `HistoryEntry` ring buffer and expose it via the visualizer
- [ ] GPU acceleration — port the participant simulation (especially noise traders) to run on the GPU using CUDA or wgpu
- [ ] Cloud deployment — run the exchange and market layers on separate machines communicating over the network via Docker Swarm or Kubernetes

---

## Dependencies

| Crate | Purpose |
|---|---|
| `crossbeam` | Lock-free channels and queues for inter-thread communication |
| `rand` / `rand_distr` | Log-normal quantity sampling, exponential sleep times |
| `windows-sys` | Windows debug break for development assertions |
