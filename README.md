# RaceSimX

**Physics-based drag and roll race simulation.** Feed it two cars -- from bare specs to full dyno sheets -- and let the engine settle the argument.

Built with a Rust simulation core and a Tauri v2 desktop UI.

---

## Features

- **5 ms kinematic time-step integration** -- not a single algebraic guess, but thousands of force-balance calculations per race
- **Partial input support** -- provide as little as weight, peak HP, and drivetrain layout; the estimation pipeline fills the rest from layered profiles
- **Smart defaults cascade** -- user value > vehicle class > drivetrain profile > transmission profile > global defaults, all loaded from editable TOML, nothing hardcoded
- **Confidence scoring** -- every comparison shows a 0--100% score per car, with per-field source tags (provided, derived, profile, default) and impact badges (critical / high / medium / low)
- **Aero-wall detection** -- if drag overwhelms thrust before the target speed, the sim declares a DNF instead of looping forever
- **Dark racing-themed UI** -- side-by-side car editors, toggleable race chips, results table with winner highlighting, and assumption breakdowns
- **Basic / Advanced toggle** -- show only the three required fields when you have limited data, or flip to Advanced to dial in every variable
- **Save & load configs** -- create, edit, and save car profiles directly from the UI without touching files
- **Environment panel** -- adjust air density and other conditions on the fly

## Prerequisites

| Tool | Version |
|------|---------|
| [Node.js](https://nodejs.org/) | 18+ |
| [Rust](https://www.rust-lang.org/tools/install) | 1.70+ (stable) |
| [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) | WebView2 (Windows), webkit2gtk (Linux) |

## Quick Start

```bash
# Clone the repo
git clone https://github.com/your-org/RaceSimX.git
cd RaceSimX

# Install frontend dependencies
npm install

# Launch in development mode (starts Vite + Tauri together)
npm run tauri dev
```

To create a production build:

```bash
npm run tauri build
```

The installer/executable will appear in `src-tauri/target/release/bundle/`.

## How It Works

At every 5 ms step the solver computes:

1. **Engine torque** from a synthesised curve (idle --> peak torque --> peak HP --> overrev)
2. **Tractive force** = min(power-limited, grip-limited) with dynamic weight transfer
3. **Aerodynamic drag** = 0.5 * rho * Cd * A * v^2
4. **Rolling resistance** = mass * g * Crr
5. **Effective mass** with rotational inertia correction
6. **Net acceleration**, integrated into velocity and distance

Gear shifts introduce a configurable off-power window. Launch control holds RPM at a target for standing starts. Roll races kick down to the optimal gear before the clock starts. Sub-step interpolation pins the finish to < 1 ms accuracy.

### Estimation Pipeline

Only three fields are required per car: **weight**, **peak HP**, and **driven wheels**. All 16 remaining variables are resolved through a priority cascade:

| Priority | Source | Example |
|----------|--------|---------|
| 1 | User-provided value | `tq_peak = 420` in the TOML or form |
| 2 | Vehicle class profile | `muscle_car` --> Cd 0.36, CG 20 in |
| 3 | Drivetrain profile | `RWD` --> 15% loss, 50% rear weight |
| 4 | Transmission profile | `dual_clutch` --> 7-speed ratios, 0.05 s shift |
| 5 | Global defaults | Cd 0.32, tire mu 1.0, etc. |

Every resolved value is tagged with its origin so you always know what's real and what's estimated.

### Race Events (default)

| Event | Type |
|-------|------|
| 1/8 Mile Dig | Standing start, 660 ft |
| 1/4 Mile Dig | Standing start, 1320 ft |
| 1 Mile Dig | Standing start, 5280 ft |
| 40--150 mph Roll | Rolling start |
| 60--150 mph Roll | Rolling start |

Race presets are fully editable in `config/races/standard.toml`.

## Adding Cars

### From the UI

1. Fill in the car form (minimum: Weight, Peak HP, Driven Wheels).
2. Toggle **Advanced** to add engine, transmission, chassis, and tire data as available.
3. Type a config name and click **Save** -- the car appears in the preset dropdown instantly.

### From a TOML file

Create a file in `config/cars/`. Minimal example:

```toml
name = "My Car"
weight_lbs = 3500.0
hp_peak = 400.0
driven_wheels = "AWD"
```

Full example with every optional field: see `config/cars/example_muscle_car.toml`.

## Editing Defaults

All estimation constants live in `config/profiles/defaults.toml`. You can tune:

- **Drivetrain profiles** -- loss percentages and weight distribution per layout (FWD / RWD / AWD)
- **Transmission profiles** -- gear ratios, final drive, and shift times for auto, manual, dual-clutch, single-speed
- **Vehicle class profiles** -- aero, chassis, and tire defaults for sports car, sedan, muscle car, SUV, EV
- **Engine defaults** -- torque curve synthesis parameters, idle RPM, launch RPM
- **Chassis defaults** -- Cd, frontal area, tire diameter, CG height, wheelbase, tire mu
- **Physics constants** -- rolling resistance, rotational inertia factors
- **Simulation settings** -- time step, max duration, aero-wall thresholds

No code changes required -- edit a TOML value, restart the app, done.

## Environment Settings

Air density and other environmental conditions are managed from the **Environment** panel in the UI, or directly in `config/environment/standard.toml`:

```toml
name = "Standard Sea Level"
air_density_kgm3 = 1.225
```

## Project Layout

```
config/                        editable TOML configuration
  profiles/defaults.toml         estimation profiles & physics constants
  races/standard.toml            race presets (dig + roll events)
  environment/standard.toml      air density & conditions
  cars/                          saved car configs

crates/sim-core/               standalone Rust physics library
  src/types.rs                   data schemas with source tracking
  src/estimation.rs              layered fallback resolution pipeline
  src/solver.rs                  time-step race simulator
  src/confidence.rs              confidence scoring
  src/validation.rs              input range checking & contradiction detection
  src/config.rs                  TOML loading
  src/units.rs                   unit conversions

src-tauri/                     Tauri v2 desktop backend
src/                           frontend (vanilla JS + CSS)
```

## Tests

```bash
cargo test -p sim-core
```

19 integration tests cover config loading, estimation fallbacks, torque curve synthesis, quarter-mile and roll-race simulation bounds, aero-wall DNF behaviour, confidence scoring, and input validation.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Physics engine | Rust (standalone crate, no framework deps) |
| Desktop shell | Tauri v2 |
| Frontend | Vanilla JS + CSS (no framework) |
| Build tooling | Vite 6, Cargo |
| Configuration | TOML |

## License

See [LICENSE](LICENSE) for details.
