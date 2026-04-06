use crate::config::ProfileConfig;
use crate::types::*;
use crate::units;

/// Resolves a partial `CarInput` into a fully-populated `ResolvedCar` by
/// filling missing fields from profiles and derivation rules.  Every resolved
/// value is tagged with its `ValueSource` so the UI can show assumptions.
pub fn resolve_car(input: &CarInput, profiles: &ProfileConfig) -> Result<ResolvedCar, String> {
    let layout_key = input.driven_wheels.to_string();
    let dt_profile = profiles.drivetrain.get(&layout_key);
    let trans_key = input
        .transmission_type
        .clone()
        .unwrap_or_else(|| "auto".to_string());
    let trans_profile = profiles.transmission.get(&trans_key);
    let class_profile = input
        .vehicle_class
        .as_ref()
        .and_then(|c| profiles.vehicle_class.as_ref()?.get(c));

    let eng = &profiles.engine_defaults;
    let chassis = &profiles.chassis_defaults;

    let (tq_peak, src_tq) = resolve_f64(
        input.tq_peak,
        || {
            class_profile.and_then(|_| None) // no class-level tq default
        },
        || Some(input.hp_peak * eng.tq_from_hp_multiplier),
        "derived from hp_peak * tq_from_hp_multiplier",
    );

    let (rpm_hp, src_rpm_hp) = resolve_f64(
        input.rpm_hp,
        || None,
        || Some(eng.default_rpm_hp),
        "engine_defaults.default_rpm_hp",
    );

    let (rpm_tq, src_rpm_tq) = resolve_f64(
        input.rpm_tq,
        || None,
        || {
            let derived = (rpm_hp - eng.rpm_tq_offset_from_hp).max(eng.min_rpm_tq);
            Some(derived)
        },
        "derived from rpm_hp - offset",
    );

    let (redline, src_redline) = resolve_f64(
        input.redline,
        || None,
        || Some(rpm_hp + eng.redline_offset_from_hp),
        "derived from rpm_hp + offset",
    );

    let (gears, src_gears) = match &input.gears {
        Some(g) => (g.clone(), ValueSource::Provided),
        None => match trans_profile {
            Some(tp) => (
                tp.gears.clone(),
                ValueSource::ProfileDefault {
                    profile_name: format!("transmission.{trans_key}"),
                },
            ),
            None => {
                let fallback = profiles
                    .transmission
                    .get("auto")
                    .map(|t| t.gears.clone())
                    .unwrap_or_else(|| vec![4.17, 2.34, 1.52, 1.14, 0.87, 0.69]);
                (fallback, ValueSource::GlobalDefault)
            }
        },
    };

    let (final_drive, src_fd) = match input.final_drive {
        Some(v) => (v, ValueSource::Provided),
        None => match trans_profile {
            Some(tp) => (
                tp.final_drive,
                ValueSource::ProfileDefault {
                    profile_name: format!("transmission.{trans_key}"),
                },
            ),
            None => {
                let fallback = profiles
                    .transmission
                    .get("auto")
                    .map(|t| t.final_drive)
                    .unwrap_or(3.42);
                (fallback, ValueSource::GlobalDefault)
            }
        },
    };

    let (shift_time_s, src_shift) = match input.shift_time_s {
        Some(v) => (v, ValueSource::Provided),
        None => match trans_profile {
            Some(tp) => (
                tp.shift_time_s,
                ValueSource::ProfileDefault {
                    profile_name: format!("transmission.{trans_key}"),
                },
            ),
            None => {
                let fallback = profiles
                    .transmission
                    .get("auto")
                    .map(|t| t.shift_time_s)
                    .unwrap_or(0.15);
                (fallback, ValueSource::GlobalDefault)
            }
        },
    };

    let (cd, src_cd) = resolve_with_class(
        input.cd,
        class_profile.and_then(|c| c.cd),
        &input.vehicle_class,
        chassis.cd,
        "cd",
    );

    let (frontal_area_sqft, src_fa) = resolve_with_class(
        input.frontal_area_sqft,
        class_profile.and_then(|c| c.frontal_area_sqft),
        &input.vehicle_class,
        chassis.frontal_area_sqft,
        "frontal_area_sqft",
    );

    let (tire_diameter_in, src_tire) = resolve_f64(
        input.tire_diameter_in,
        || None,
        || Some(chassis.tire_diameter_in),
        "chassis_defaults.tire_diameter_in",
    );

    let (cg_height_in, src_cg) = resolve_with_class(
        input.cg_height_in,
        class_profile.and_then(|c| c.cg_height_in),
        &input.vehicle_class,
        chassis.cg_height_in,
        "cg_height_in",
    );

    let (wheelbase_in, src_wb) = resolve_with_class(
        input.wheelbase_in,
        class_profile.and_then(|c| c.wheelbase_in),
        &input.vehicle_class,
        chassis.wheelbase_in,
        "wheelbase_in",
    );

    let (tire_mu, src_mu) = resolve_with_class(
        input.tire_mu,
        class_profile.and_then(|c| c.tire_mu),
        &input.vehicle_class,
        chassis.tire_mu,
        "tire_mu",
    );

    let (weight_dist_rear_pct, src_wd) = match input.weight_dist_rear_pct {
        Some(v) => (v, ValueSource::Provided),
        None => match dt_profile {
            Some(dp) => (
                dp.weight_dist_rear_pct,
                ValueSource::ProfileDefault {
                    profile_name: format!("drivetrain.{layout_key}"),
                },
            ),
            None => (50.0, ValueSource::GlobalDefault),
        },
    };

    let (drivetrain_loss_pct, src_dl) = match input.drivetrain_loss_pct {
        Some(v) => (v, ValueSource::Provided),
        None => {
            if let Some(cp) = class_profile.and_then(|c| c.drivetrain_loss_pct) {
                (
                    cp,
                    ValueSource::ProfileDefault {
                        profile_name: format!(
                            "vehicle_class.{}",
                            input.vehicle_class.as_deref().unwrap_or("?")
                        ),
                    },
                )
            } else if let Some(dp) = dt_profile {
                (
                    dp.loss_pct,
                    ValueSource::ProfileDefault {
                        profile_name: format!("drivetrain.{layout_key}"),
                    },
                )
            } else {
                (15.0, ValueSource::GlobalDefault)
            }
        }
    };

    let (launch_rpm, src_launch) = resolve_f64(
        input.launch_rpm,
        || None,
        || Some(eng.default_launch_rpm),
        "engine_defaults.default_launch_rpm",
    );

    Ok(ResolvedCar {
        name: input
            .name
            .clone()
            .unwrap_or_else(|| format!("{:.0} HP {} car", input.hp_peak, input.driven_wheels)),
        weight_lbs: input.weight_lbs,
        hp_peak: input.hp_peak,
        driven_wheels: input.driven_wheels,
        tq_peak,
        rpm_hp,
        rpm_tq,
        redline,
        gears,
        final_drive,
        cd,
        frontal_area_sqft,
        tire_diameter_in,
        shift_time_s,
        launch_rpm,
        cg_height_in,
        wheelbase_in,
        weight_dist_rear_pct,
        tire_mu,
        drivetrain_loss_pct,
        sources: FieldSources {
            tq_peak: src_tq,
            rpm_hp: src_rpm_hp,
            rpm_tq: src_rpm_tq,
            redline: src_redline,
            gears: src_gears,
            final_drive: src_fd,
            shift_time_s: src_shift,
            cd: src_cd,
            frontal_area_sqft: src_fa,
            tire_diameter_in: src_tire,
            cg_height_in: src_cg,
            wheelbase_in: src_wb,
            weight_dist_rear_pct: src_wd,
            tire_mu: src_mu,
            drivetrain_loss_pct: src_dl,
            launch_rpm: src_launch,
        },
    })
}

/// Convert a `ResolvedCar` + environment into the SI-unit `SimCar` the solver
/// consumes.
pub fn build_sim_car(
    car: &ResolvedCar,
    env: &EnvironmentInput,
    profiles: &ProfileConfig,
) -> SimCar {
    let phys = &profiles.physics_constants;
    let eng = &profiles.engine_defaults;

    SimCar {
        name: car.name.clone(),
        mass_kg: units::lbs_to_kg(car.weight_lbs),
        weight_n: units::lbs_to_kg(car.weight_lbs) * units::GRAVITY_MS2,
        tq_peak_nm: units::lbft_to_nm(car.tq_peak),
        rpm_hp: car.rpm_hp,
        rpm_tq: car.rpm_tq,
        redline: car.redline,
        hp_peak: car.hp_peak,
        gears: car.gears.clone(),
        final_drive: car.final_drive,
        efficiency: 1.0 - (car.drivetrain_loss_pct / 100.0),
        cd: car.cd,
        frontal_area_m2: units::sqft_to_sqm(car.frontal_area_sqft),
        tire_radius_m: units::inches_to_meters(car.tire_diameter_in / 2.0),
        shift_time_s: car.shift_time_s,
        launch_rpm: car.launch_rpm,
        cg_height_m: units::inches_to_meters(car.cg_height_in),
        wheelbase_m: units::inches_to_meters(car.wheelbase_in),
        rear_weight_bias: car.weight_dist_rear_pct / 100.0,
        tire_mu: car.tire_mu,
        driven_wheels: car.driven_wheels,
        rho: env.air_density_kgm3.unwrap_or(1.225),
        crr: phys.rolling_resistance_crr,
        rotational_inertia_base: phys.rotational_inertia_base,
        rotational_inertia_gear_factor: phys.rotational_inertia_gear_factor,
        idle_rpm: eng.idle_rpm,
        idle_torque_fraction: eng.idle_torque_fraction,
        overrev_torque_fraction: eng.overrev_torque_fraction,
    }
}

fn resolve_f64(
    user: Option<f64>,
    class_fn: impl FnOnce() -> Option<f64>,
    global_fn: impl FnOnce() -> Option<f64>,
    global_label: &str,
) -> (f64, ValueSource) {
    if let Some(v) = user {
        return (v, ValueSource::Provided);
    }
    if let Some(v) = class_fn() {
        return (
            v,
            ValueSource::DerivedFromProvided {
                description: global_label.to_string(),
            },
        );
    }
    if let Some(v) = global_fn() {
        return (v, ValueSource::GlobalDefault);
    }
    (0.0, ValueSource::GlobalDefault)
}

fn resolve_with_class(
    user: Option<f64>,
    class_val: Option<f64>,
    class_name: &Option<String>,
    global: f64,
    field: &str,
) -> (f64, ValueSource) {
    if let Some(v) = user {
        return (v, ValueSource::Provided);
    }
    if let Some(v) = class_val {
        return (
            v,
            ValueSource::ProfileDefault {
                profile_name: format!(
                    "vehicle_class.{}.{field}",
                    class_name.as_deref().unwrap_or("?")
                ),
            },
        );
    }
    (global, ValueSource::GlobalDefault)
}
