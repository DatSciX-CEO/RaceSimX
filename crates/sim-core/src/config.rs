use crate::types::{CarInput, DriveLayout, EnvironmentInput, RacePreset, RaceTarget};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileConfig {
    pub simulation: SimulationDefaults,
    pub engine_defaults: EngineDefaults,
    pub chassis_defaults: ChassisDefaults,
    pub physics_constants: PhysicsConstants,
    pub drivetrain: HashMap<String, DrivetrainProfile>,
    pub transmission: HashMap<String, TransmissionProfile>,
    pub vehicle_class: Option<HashMap<String, VehicleClassProfile>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimulationDefaults {
    pub time_step_s: f64,
    pub max_time_s: f64,
    pub aero_wall_accel_threshold: f64,
    pub aero_wall_min_speed_ms: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineDefaults {
    pub idle_rpm: f64,
    pub idle_torque_fraction: f64,
    pub overrev_torque_fraction: f64,
    pub default_rpm_hp: f64,
    pub rpm_tq_offset_from_hp: f64,
    pub min_rpm_tq: f64,
    pub redline_offset_from_hp: f64,
    pub tq_from_hp_multiplier: f64,
    pub default_launch_rpm: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChassisDefaults {
    pub cd: f64,
    pub frontal_area_sqft: f64,
    pub tire_diameter_in: f64,
    pub cg_height_in: f64,
    pub wheelbase_in: f64,
    pub tire_mu: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhysicsConstants {
    pub rolling_resistance_crr: f64,
    pub rotational_inertia_base: f64,
    pub rotational_inertia_gear_factor: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrivetrainProfile {
    pub loss_pct: f64,
    pub weight_dist_rear_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransmissionProfile {
    pub gears: Vec<f64>,
    pub final_drive: f64,
    pub shift_time_s: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VehicleClassProfile {
    pub cd: Option<f64>,
    pub frontal_area_sqft: Option<f64>,
    pub cg_height_in: Option<f64>,
    pub wheelbase_in: Option<f64>,
    pub tire_mu: Option<f64>,
    pub drivetrain_loss_pct: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct RacePresetFile {
    race: Vec<RacePresetEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct RacePresetEntry {
    name: String,
    start_mph: f64,
    target_distance_ft: Option<f64>,
    target_speed_mph: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct EnvironmentFile {
    name: Option<String>,
    air_density_kgm3: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct CarFile {
    name: Option<String>,
    weight_lbs: f64,
    hp_peak: f64,
    driven_wheels: DriveLayout,
    tq_peak: Option<f64>,
    rpm_hp: Option<f64>,
    rpm_tq: Option<f64>,
    redline: Option<f64>,
    transmission_type: Option<String>,
    gears: Option<Vec<f64>>,
    final_drive: Option<f64>,
    shift_time_s: Option<f64>,
    vehicle_class: Option<String>,
    cd: Option<f64>,
    frontal_area_sqft: Option<f64>,
    tire_diameter_in: Option<f64>,
    cg_height_in: Option<f64>,
    wheelbase_in: Option<f64>,
    weight_dist_rear_pct: Option<f64>,
    tire_mu: Option<f64>,
    drivetrain_loss_pct: Option<f64>,
    launch_rpm: Option<f64>,
}

pub fn load_profiles(path: &Path) -> Result<ProfileConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read profile config at {}: {e}", path.display()))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse profile config at {}: {e}", path.display()))
}

pub fn load_race_presets(path: &Path) -> Result<Vec<RacePreset>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read race presets at {}: {e}", path.display()))?;
    let file: RacePresetFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse race presets at {}: {e}", path.display()))?;

    file.race
        .into_iter()
        .map(|entry| {
            let target = match (entry.target_distance_ft, entry.target_speed_mph) {
                (Some(ft), None) => RaceTarget::Distance { feet: ft },
                (None, Some(mph)) => RaceTarget::Speed { mph },
                (Some(_), Some(_)) => {
                    return Err(format!(
                        "Race '{}': specify either target_distance_ft or target_speed_mph, not both",
                        entry.name
                    ))
                }
                (None, None) => {
                    return Err(format!(
                        "Race '{}': must specify target_distance_ft or target_speed_mph",
                        entry.name
                    ))
                }
            };
            Ok(RacePreset {
                name: entry.name,
                start_mph: entry.start_mph,
                target,
            })
        })
        .collect()
}

pub fn load_environment(path: &Path) -> Result<EnvironmentInput, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read environment at {}: {e}", path.display()))?;
    let file: EnvironmentFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse environment at {}: {e}", path.display()))?;
    Ok(EnvironmentInput {
        name: file.name,
        air_density_kgm3: file.air_density_kgm3,
    })
}

pub fn load_car(path: &Path) -> Result<CarInput, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read car config at {}: {e}", path.display()))?;
    let file: CarFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse car config at {}: {e}", path.display()))?;
    Ok(CarInput {
        name: file.name,
        weight_lbs: file.weight_lbs,
        hp_peak: file.hp_peak,
        driven_wheels: file.driven_wheels,
        tq_peak: file.tq_peak,
        rpm_hp: file.rpm_hp,
        rpm_tq: file.rpm_tq,
        redline: file.redline,
        transmission_type: file.transmission_type,
        gears: file.gears,
        final_drive: file.final_drive,
        shift_time_s: file.shift_time_s,
        vehicle_class: file.vehicle_class,
        cd: file.cd,
        frontal_area_sqft: file.frontal_area_sqft,
        tire_diameter_in: file.tire_diameter_in,
        cg_height_in: file.cg_height_in,
        wheelbase_in: file.wheelbase_in,
        weight_dist_rear_pct: file.weight_dist_rear_pct,
        tire_mu: file.tire_mu,
        drivetrain_loss_pct: file.drivetrain_loss_pct,
        launch_rpm: file.launch_rpm,
    })
}

pub fn list_car_files(dir: &Path) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read car directory {}: {e}", dir.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}
