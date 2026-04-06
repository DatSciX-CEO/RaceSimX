use crate::config::SimulationDefaults;
use crate::types::*;
use crate::units;
use std::f64::consts::PI;

impl SimCar {
    /// Synthesise torque (Nm) at a given RPM from peak-HP / peak-TQ metadata.
    /// Three-segment piecewise linear curve identical to the build.md reference.
    pub fn torque_nm_at_rpm(&self, rpm: f64) -> f64 {
        let tq_at_hp_rpm =
            units::lbft_to_nm(units::hp_rpm_to_tq_lbft(self.hp_peak, self.rpm_hp));

        if rpm <= self.rpm_tq {
            let idle_tq = self.tq_peak_nm * self.idle_torque_fraction;
            let span = (self.rpm_tq - self.idle_rpm).max(1.0);
            let slope = (self.tq_peak_nm - idle_tq) / span;
            idle_tq + slope * (rpm - self.idle_rpm).max(0.0)
        } else if rpm <= self.rpm_hp {
            let span = (self.rpm_hp - self.rpm_tq).max(1.0);
            let slope = (tq_at_hp_rpm - self.tq_peak_nm) / span;
            self.tq_peak_nm + slope * (rpm - self.rpm_tq)
        } else {
            let overrev_tq = tq_at_hp_rpm * self.overrev_torque_fraction;
            let span = (self.redline - self.rpm_hp).max(1.0);
            let slope = (overrev_tq - tq_at_hp_rpm) / span;
            tq_at_hp_rpm + slope * (rpm - self.rpm_hp)
        }
    }
}

pub fn simulate_race(
    car: &SimCar,
    preset: &RacePreset,
    sim_cfg: &SimulationDefaults,
) -> RaceResult {
    let dt = sim_cfg.time_step_s;
    let max_t = sim_cfg.max_time_s;

    let mut v = units::mph_to_ms(preset.start_mph);
    let target_v = match &preset.target {
        RaceTarget::Speed { mph } => Some(units::mph_to_ms(*mph)),
        _ => None,
    };
    let target_d = match &preset.target {
        RaceTarget::Distance { feet } => Some(units::ft_to_m(*feet)),
        _ => None,
    };

    let mut d: f64 = 0.0;
    let mut t: f64 = 0.0;
    let mut accel: f64 = 0.0;
    let mut current_gear: usize = 0;
    let mut shift_timer: f64 = 0.0;
    let mut is_shifting = false;

    // Roll-race kickdown: drop to the lowest gear that doesn't over-rev.
    if v > 0.0 {
        for (i, &gear) in car.gears.iter().enumerate() {
            let wheel_rps = v / (2.0 * PI * car.tire_radius_m);
            let rpm = wheel_rps * 60.0 * gear * car.final_drive;
            if rpm < car.redline - 800.0 {
                current_gear = i;
                break;
            }
        }
    }

    loop {
        // --- Check finish conditions with sub-step interpolation ---
        if let Some(td) = target_d {
            if d >= td {
                let dt_over = if v > 0.001 { (d - td) / v } else { 0.0 };
                let t_exact = (t - dt_over).max(0.0);
                return RaceResult {
                    time_s: t_exact,
                    final_speed_mph: units::ms_to_mph(v),
                    completed: true,
                    dnf_reason: None,
                };
            }
        }
        if let Some(tv) = target_v {
            if v >= tv {
                let dt_over = if accel > 0.001 {
                    (v - tv) / accel
                } else {
                    0.0
                };
                let t_exact = (t - dt_over).max(0.0);
                return RaceResult {
                    time_s: t_exact,
                    final_speed_mph: units::ms_to_mph(tv),
                    completed: true,
                    dnf_reason: None,
                };
            }
        }

        if current_gear >= car.gears.len() {
            return RaceResult {
                time_s: f64::INFINITY,
                final_speed_mph: units::ms_to_mph(v),
                completed: false,
                dnf_reason: Some("Ran out of gears (top speed reached)".into()),
            };
        }

        let overall_ratio = car.gears[current_gear] * car.final_drive;
        let wheel_rps = v / (2.0 * PI * car.tire_radius_m);
        let mut engine_rpm = wheel_rps * 60.0 * overall_ratio;

        // Launch control: hold engine at launch RPM from standstill in first gear
        if engine_rpm < car.launch_rpm && preset.start_mph == 0.0 && current_gear == 0 {
            engine_rpm = car.launch_rpm;
        }

        // Shift when hitting redline
        if engine_rpm >= car.redline {
            is_shifting = true;
            shift_timer = car.shift_time_s;
            current_gear += 1;
            if current_gear >= car.gears.len() {
                continue;
            }
        }

        let f_tractive;
        let mut was_off_power = false;
        if is_shifting {
            f_tractive = 0.0;
            shift_timer -= dt;
            if shift_timer <= 0.0 {
                is_shifting = false;
            }
            was_off_power = true;
        } else {
            let engine_tq = car.torque_nm_at_rpm(engine_rpm);
            let f_engine = (engine_tq * overall_ratio * car.efficiency) / car.tire_radius_m;

            // Dynamic weight transfer
            let weight_transfer =
                (car.mass_kg * accel * car.cg_height_m) / car.wheelbase_m;

            let w_driven = match car.driven_wheels {
                DriveLayout::RWD => (car.weight_n * car.rear_weight_bias) + weight_transfer,
                DriveLayout::FWD => {
                    (car.weight_n * (1.0 - car.rear_weight_bias)) - weight_transfer
                }
                DriveLayout::AWD => car.weight_n,
            };

            let f_grip = w_driven * car.tire_mu;
            f_tractive = f_engine.min(f_grip.max(0.0));
        }

        let f_drag = 0.5 * car.rho * car.cd * car.frontal_area_m2 * v * v;
        let f_roll = car.crr * car.weight_n;
        let mass_eff = car.mass_kg
            * (1.0
                + car.rotational_inertia_base
                + car.rotational_inertia_gear_factor * overall_ratio * overall_ratio);

        accel = (f_tractive - f_drag - f_roll) / mass_eff;

        // Aero wall: drag prevents further acceleration
        if accel <= sim_cfg.aero_wall_accel_threshold
            && v > sim_cfg.aero_wall_min_speed_ms
            && !is_shifting
            && !was_off_power
        {
            return RaceResult {
                time_s: f64::INFINITY,
                final_speed_mph: units::ms_to_mph(v),
                completed: false,
                dnf_reason: Some(format!(
                    "Aero wall at {:.1} mph",
                    units::ms_to_mph(v)
                )),
            };
        }

        v += accel * dt;
        if v < 0.0 {
            v = 0.0;
        }
        d += v * dt;
        t += dt;

        if t > max_t {
            return RaceResult {
                time_s: f64::INFINITY,
                final_speed_mph: units::ms_to_mph(v),
                completed: false,
                dnf_reason: Some(format!("Timeout > {max_t}s")),
            };
        }
    }
}

/// Run a full comparison: resolve both cars, simulate every race preset, and
/// return the combined result.
pub fn run_comparison(
    car_a: &ResolvedCar,
    car_b: &ResolvedCar,
    env: &EnvironmentInput,
    races: &[RacePreset],
    profiles: &crate::config::ProfileConfig,
) -> ComparisonResult {
    let sim_a = crate::estimation::build_sim_car(car_a, env, profiles);
    let sim_b = crate::estimation::build_sim_car(car_b, env, profiles);
    let sim_cfg = &profiles.simulation;

    let race_results: Vec<RaceComparison> = races
        .iter()
        .map(|preset| {
            let ra = simulate_race(&sim_a, preset, sim_cfg);
            let rb = simulate_race(&sim_b, preset, sim_cfg);

            let (winner, margin) = match (ra.completed, rb.completed) {
                (true, true) => {
                    let diff = ra.time_s - rb.time_s;
                    if diff.abs() < 0.001 {
                        (None, 0.0)
                    } else if diff < 0.0 {
                        (Some(car_a.name.clone()), diff.abs())
                    } else {
                        (Some(car_b.name.clone()), diff.abs())
                    }
                }
                (true, false) => (Some(car_a.name.clone()), 0.0),
                (false, true) => (Some(car_b.name.clone()), 0.0),
                (false, false) => (None, 0.0),
            };

            RaceComparison {
                race_name: preset.name.clone(),
                result_a: ra,
                result_b: rb,
                winner,
                margin_s: margin,
            }
        })
        .collect();

    let confidence_a = crate::confidence::car_confidence(car_a);
    let confidence_b = crate::confidence::car_confidence(car_b);
    let details_a = crate::confidence::field_details(car_a);
    let details_b = crate::confidence::field_details(car_b);

    ComparisonResult {
        car_a: car_a.clone(),
        car_b: car_b.clone(),
        races: race_results,
        confidence_a,
        confidence_b,
        details_a,
        details_b,
    }
}
