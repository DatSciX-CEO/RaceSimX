use sim_core::*;
use std::path::PathBuf;

fn config_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("config")
}

fn load_test_profiles() -> config::ProfileConfig {
    load_profiles(&config_root().join("profiles").join("defaults.toml")).unwrap()
}

fn load_test_env() -> EnvironmentInput {
    load_environment(&config_root().join("environment").join("standard.toml")).unwrap()
}

fn load_test_races() -> Vec<RacePreset> {
    load_race_presets(&config_root().join("races").join("standard.toml")).unwrap()
}

// ─── Config Loading ─────────────────────────────────────────────────

#[test]
fn profiles_load_successfully() {
    let p = load_test_profiles();
    assert!(p.simulation.time_step_s > 0.0);
    assert!(p.drivetrain.contains_key("RWD"));
    assert!(p.drivetrain.contains_key("FWD"));
    assert!(p.drivetrain.contains_key("AWD"));
    assert!(p.transmission.contains_key("auto"));
    assert!(p.transmission.contains_key("manual"));
}

#[test]
fn race_presets_load_all_five() {
    let races = load_test_races();
    assert_eq!(races.len(), 5);
    assert_eq!(races[0].name, "1/8 Mile Dig");
}

#[test]
fn car_files_load() {
    let muscle = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    assert_eq!(muscle.weight_lbs, 3950.0);
    assert_eq!(muscle.hp_peak, 480.0);
    assert_eq!(muscle.driven_wheels, DriveLayout::RWD);
    assert!(muscle.gears.is_some());

    let sedan = load_car(&config_root().join("cars").join("example_awd_sedan.toml")).unwrap();
    assert_eq!(sedan.driven_wheels, DriveLayout::AWD);
    assert!(sedan.gears.is_none());
}

#[test]
fn list_car_files_finds_examples() {
    let cars = list_car_files(&config_root().join("cars")).unwrap();
    assert!(cars.len() >= 2);
    assert!(cars.contains(&"example_muscle_car".to_string()));
}

// ─── Estimation / Resolution ────────────────────────────────────────

#[test]
fn resolve_full_car_uses_provided_values() {
    let profiles = load_test_profiles();
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let resolved = resolve_car(&input, &profiles).unwrap();

    assert_eq!(resolved.tq_peak, 420.0);
    assert_eq!(resolved.gears.len(), 6);
    assert_eq!(resolved.cd, 0.35);
    assert!(matches!(resolved.sources.tq_peak, ValueSource::Provided));
    assert!(matches!(resolved.sources.gears, ValueSource::Provided));
    assert!(matches!(resolved.sources.cd, ValueSource::Provided));
}

#[test]
fn resolve_minimal_car_fills_defaults() {
    let profiles = load_test_profiles();
    let input = load_car(&config_root().join("cars").join("example_awd_sedan.toml")).unwrap();
    let resolved = resolve_car(&input, &profiles).unwrap();

    assert!(resolved.tq_peak > 0.0, "torque should be estimated");
    assert!(!resolved.gears.is_empty(), "gears should be filled in");
    assert!(resolved.final_drive > 0.0);
    assert!(resolved.cd > 0.0);

    assert!(
        !matches!(resolved.sources.tq_peak, ValueSource::Provided),
        "tq_peak should NOT be Provided for minimal car"
    );
    assert!(
        !matches!(resolved.sources.gears, ValueSource::Provided),
        "gears should NOT be Provided for minimal car"
    );

    assert_eq!(resolved.drivetrain_loss_pct, 18.0, "AWD = 18% loss from profile");
}

#[test]
fn resolve_respects_vehicle_class_profile() {
    let profiles = load_test_profiles();
    let input = CarInput {
        name: Some("Sports test".into()),
        weight_lbs: 3200.0,
        hp_peak: 350.0,
        driven_wheels: DriveLayout::RWD,
        vehicle_class: Some("sports_car".into()),
        tq_peak: None,
        rpm_hp: None,
        rpm_tq: None,
        redline: None,
        transmission_type: None,
        gears: None,
        final_drive: None,
        shift_time_s: None,
        cd: None,
        frontal_area_sqft: None,
        tire_diameter_in: None,
        cg_height_in: None,
        wheelbase_in: None,
        weight_dist_rear_pct: None,
        tire_mu: None,
        drivetrain_loss_pct: None,
        launch_rpm: None,
    };
    let resolved = resolve_car(&input, &profiles).unwrap();
    assert!((resolved.cd - 0.30).abs() < 0.001, "should pick sports_car cd=0.30");
    assert!((resolved.tire_mu - 1.1).abs() < 0.001, "should pick sports_car tire_mu=1.1");
}

// ─── Solver Physics ─────────────────────────────────────────────────

fn make_sim_car(input: &CarInput) -> (SimCar, config::ProfileConfig) {
    let profiles = load_test_profiles();
    let env = load_test_env();
    let resolved = resolve_car(input, &profiles).unwrap();
    let sim = build_sim_car(&resolved, &env, &profiles);
    (sim, profiles)
}

#[test]
fn torque_curve_peaks_at_rpm_tq() {
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let (sim, _) = make_sim_car(&input);

    let tq_at_peak = sim.torque_nm_at_rpm(sim.rpm_tq);
    let tq_below = sim.torque_nm_at_rpm(sim.rpm_tq - 500.0);
    let tq_above = sim.torque_nm_at_rpm(sim.rpm_tq + 500.0);

    assert!(tq_at_peak >= tq_below, "torque should peak at rpm_tq");
    assert!(tq_at_peak >= tq_above, "torque should peak at rpm_tq");
    assert!(tq_at_peak > 0.0);
}

#[test]
fn torque_curve_never_negative() {
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let (sim, _) = make_sim_car(&input);
    for rpm in (1000..=8000).step_by(100) {
        let tq = sim.torque_nm_at_rpm(rpm as f64);
        assert!(tq >= 0.0, "torque should not be negative at RPM {rpm}");
    }
}

#[test]
fn quarter_mile_produces_reasonable_time() {
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let (sim, profiles) = make_sim_car(&input);
    let preset = RacePreset {
        name: "1/4 Mile".into(),
        start_mph: 0.0,
        target: RaceTarget::Distance { feet: 1320.0 },
    };
    let result = simulate_race(&sim, &preset, &profiles.simulation);
    assert!(result.completed, "480HP RWD should finish 1/4 mile");
    assert!(
        result.time_s > 9.0 && result.time_s < 16.0,
        "1/4 mile time {:.2}s should be between 9 and 16 for a 480HP ~4000lb car",
        result.time_s
    );
    assert!(
        result.final_speed_mph > 90.0 && result.final_speed_mph < 160.0,
        "trap speed {:.1} mph should be reasonable",
        result.final_speed_mph
    );
}

#[test]
fn eighth_mile_faster_than_quarter() {
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let (sim, profiles) = make_sim_car(&input);

    let eighth = simulate_race(
        &sim,
        &RacePreset {
            name: "1/8".into(),
            start_mph: 0.0,
            target: RaceTarget::Distance { feet: 660.0 },
        },
        &profiles.simulation,
    );
    let quarter = simulate_race(
        &sim,
        &RacePreset {
            name: "1/4".into(),
            start_mph: 0.0,
            target: RaceTarget::Distance { feet: 1320.0 },
        },
        &profiles.simulation,
    );

    assert!(eighth.completed);
    assert!(quarter.completed);
    assert!(eighth.time_s < quarter.time_s);
    assert!(eighth.final_speed_mph < quarter.final_speed_mph);
}

#[test]
fn roll_race_starts_from_speed() {
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let (sim, profiles) = make_sim_car(&input);

    let roll = simulate_race(
        &sim,
        &RacePreset {
            name: "40-150".into(),
            start_mph: 40.0,
            target: RaceTarget::Speed { mph: 150.0 },
        },
        &profiles.simulation,
    );

    assert!(roll.completed, "480HP car should reach 150 mph");
    assert!(roll.time_s > 1.0 && roll.time_s < 40.0,
        "40-150 time {:.2}s should be reasonable", roll.time_s);
}

#[test]
fn low_power_car_hits_aero_wall() {
    let profiles = load_test_profiles();
    let env = load_test_env();
    let input = CarInput {
        name: Some("Econobox".into()),
        weight_lbs: 3000.0,
        hp_peak: 100.0,
        driven_wheels: DriveLayout::FWD,
        tq_peak: None,
        rpm_hp: None,
        rpm_tq: None,
        redline: None,
        transmission_type: None,
        gears: None,
        final_drive: None,
        shift_time_s: None,
        vehicle_class: None,
        cd: None,
        frontal_area_sqft: None,
        tire_diameter_in: None,
        cg_height_in: None,
        wheelbase_in: None,
        weight_dist_rear_pct: None,
        tire_mu: None,
        drivetrain_loss_pct: None,
        launch_rpm: None,
    };
    let resolved = resolve_car(&input, &profiles).unwrap();
    let sim = build_sim_car(&resolved, &env, &profiles);

    let result = simulate_race(
        &sim,
        &RacePreset {
            name: "60-150".into(),
            start_mph: 60.0,
            target: RaceTarget::Speed { mph: 150.0 },
        },
        &profiles.simulation,
    );

    assert!(!result.completed, "100HP car should not reach 150 mph");
    assert!(result.dnf_reason.is_some());
}

#[test]
fn comparison_produces_winner() {
    let profiles = load_test_profiles();
    let env = load_test_env();
    let races = load_test_races();

    let muscle = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let sedan = load_car(&config_root().join("cars").join("example_awd_sedan.toml")).unwrap();

    let car_a = resolve_car(&muscle, &profiles).unwrap();
    let car_b = resolve_car(&sedan, &profiles).unwrap();

    let result = run_comparison(&car_a, &car_b, &env, &races, &profiles);

    assert_eq!(result.races.len(), 5);
    for race in &result.races {
        if race.result_a.completed && race.result_b.completed {
            assert!(race.winner.is_some(), "completed race should have a winner");
        }
    }
    assert!(result.confidence_a > result.confidence_b,
        "fully-specified car should have higher confidence than minimal car");
}

// ─── Confidence ─────────────────────────────────────────────────────

#[test]
fn full_car_high_confidence() {
    let profiles = load_test_profiles();
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let resolved = resolve_car(&input, &profiles).unwrap();
    let score = car_confidence(&resolved);
    assert!(
        score > 0.85,
        "fully-specified car confidence {score:.2} should be > 0.85"
    );
}

#[test]
fn minimal_car_lower_confidence() {
    let profiles = load_test_profiles();
    let input = load_car(&config_root().join("cars").join("example_awd_sedan.toml")).unwrap();
    let resolved = resolve_car(&input, &profiles).unwrap();
    let score = car_confidence(&resolved);
    assert!(
        score < 0.5,
        "minimal car confidence {score:.2} should be < 0.5"
    );
}

#[test]
fn field_details_lists_all_fields() {
    let profiles = load_test_profiles();
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let resolved = resolve_car(&input, &profiles).unwrap();
    let details = field_details(&resolved);
    assert!(!details.is_empty());
    assert!(details.iter().all(|d| !d.field.is_empty()));
}

// ─── Validation ─────────────────────────────────────────────────────

#[test]
fn valid_car_produces_no_warnings() {
    let input = load_car(&config_root().join("cars").join("example_muscle_car.toml")).unwrap();
    let warnings = validate_car_input(&input);
    assert!(
        warnings.is_empty(),
        "well-formed car should produce no warnings, got: {:?}",
        warnings.iter().map(|w| &w.message).collect::<Vec<_>>()
    );
}

#[test]
fn bad_gear_order_warns() {
    let input = CarInput {
        name: None,
        weight_lbs: 3000.0,
        hp_peak: 300.0,
        driven_wheels: DriveLayout::RWD,
        gears: Some(vec![1.0, 2.0, 3.0]),
        tq_peak: None,
        rpm_hp: None,
        rpm_tq: None,
        redline: None,
        transmission_type: None,
        final_drive: None,
        shift_time_s: None,
        vehicle_class: None,
        cd: None,
        frontal_area_sqft: None,
        tire_diameter_in: None,
        cg_height_in: None,
        wheelbase_in: None,
        weight_dist_rear_pct: None,
        tire_mu: None,
        drivetrain_loss_pct: None,
        launch_rpm: None,
    };
    let warnings = validate_car_input(&input);
    assert!(
        warnings.iter().any(|w| w.field == "gears"),
        "ascending gear ratios should trigger a warning"
    );
}
