Simulation engine (crates/sim-core/) -- a standalone Rust library with:

Data schemas (types.rs) for partial car inputs, fully-resolved cars with source tracking, race presets, and comparison results
Estimation pipeline (estimation.rs) that fills 16 optional fields from a layered profile system -- vehicle class, drivetrain layout, transmission type, then global defaults -- all loaded from config/profiles/defaults.toml, nothing hardcoded
Time-step solver (solver.rs) implementing the exact force-balance integration from your build.md: synthesised torque curves, power-vs-grip traction limiting, dynamic weight transfer, aero drag (with wall detection), rolling resistance, rotational inertia, gear shifting with off-power time, launch control, roll-race kickdown, and sub-step finish interpolation
Confidence scoring (confidence.rs) that weights each field by its physics impact (critical/high/medium/low) and reports a 0-100% score per car
Validation (validation.rs) for range checking and contradiction detection
19 passing integration tests covering config loading, fallback resolution, torque curves, quarter-mile times, roll races, aero-wall DNF, confidence, and validation
External config (config/) -- all tunable values in editable TOML:

profiles/defaults.toml -- drivetrain loss, gear ratios, aero defaults, torque curve synthesis constants, simulation step size
races/standard.toml -- 5 race presets (1/8 mi dig, 1/4 mi dig, 1 mi dig, 40-150 roll, 60-150 roll)
environment/standard.toml -- air density
cars/ -- two example cars (fully-specified muscle car, minimal AWD sedan)
Desktop app (Tauri v2 + vanilla JS):

Two side-by-side car editors with grouped optional fields
Preset car loading from saved TOML configs
Race event selection via toggleable chips
Results table with winner highlighting and time margins
Confidence bars and per-field assumption breakdowns with impact badges
Dark racing-themed UI
To launch the app: npm run tauri dev from c:\race.