use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DriveLayout {
    FWD,
    RWD,
    AWD,
}

impl fmt::Display for DriveLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FWD => write!(f, "FWD"),
            Self::RWD => write!(f, "RWD"),
            Self::AWD => write!(f, "AWD"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValueSource {
    Provided,
    DerivedFromProvided { description: String },
    ProfileDefault { profile_name: String },
    GlobalDefault,
}

impl fmt::Display for ValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provided => write!(f, "provided"),
            Self::DerivedFromProvided { description } => write!(f, "derived: {description}"),
            Self::ProfileDefault { profile_name } => write!(f, "profile: {profile_name}"),
            Self::GlobalDefault => write!(f, "global default"),
        }
    }
}

/// User-facing car configuration. Only `weight_lbs`, `hp_peak`, and
/// `driven_wheels` are required -- every other field is optional and will be
/// filled by the estimation pipeline when missing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarInput {
    pub name: Option<String>,
    pub weight_lbs: f64,
    pub hp_peak: f64,
    pub driven_wheels: DriveLayout,

    pub tq_peak: Option<f64>,
    pub rpm_hp: Option<f64>,
    pub rpm_tq: Option<f64>,
    pub redline: Option<f64>,

    pub transmission_type: Option<String>,
    pub gears: Option<Vec<f64>>,
    pub final_drive: Option<f64>,
    pub shift_time_s: Option<f64>,

    pub vehicle_class: Option<String>,
    pub cd: Option<f64>,
    pub frontal_area_sqft: Option<f64>,
    pub tire_diameter_in: Option<f64>,
    pub cg_height_in: Option<f64>,
    pub wheelbase_in: Option<f64>,
    pub weight_dist_rear_pct: Option<f64>,
    pub tire_mu: Option<f64>,
    pub drivetrain_loss_pct: Option<f64>,

    pub launch_rpm: Option<f64>,
}

/// A fully-resolved car where every field is populated and tagged with its
/// origin so the UI can report which values are real vs estimated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCar {
    pub name: String,
    pub weight_lbs: f64,
    pub hp_peak: f64,
    pub driven_wheels: DriveLayout,
    pub tq_peak: f64,
    pub rpm_hp: f64,
    pub rpm_tq: f64,
    pub redline: f64,
    pub gears: Vec<f64>,
    pub final_drive: f64,
    pub cd: f64,
    pub frontal_area_sqft: f64,
    pub tire_diameter_in: f64,
    pub shift_time_s: f64,
    pub launch_rpm: f64,
    pub cg_height_in: f64,
    pub wheelbase_in: f64,
    pub weight_dist_rear_pct: f64,
    pub tire_mu: f64,
    pub drivetrain_loss_pct: f64,
    pub sources: FieldSources,
}

/// Tracks where each resolved value came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSources {
    pub tq_peak: ValueSource,
    pub rpm_hp: ValueSource,
    pub rpm_tq: ValueSource,
    pub redline: ValueSource,
    pub gears: ValueSource,
    pub final_drive: ValueSource,
    pub shift_time_s: ValueSource,
    pub cd: ValueSource,
    pub frontal_area_sqft: ValueSource,
    pub tire_diameter_in: ValueSource,
    pub cg_height_in: ValueSource,
    pub wheelbase_in: ValueSource,
    pub weight_dist_rear_pct: ValueSource,
    pub tire_mu: ValueSource,
    pub drivetrain_loss_pct: ValueSource,
    pub launch_rpm: ValueSource,
}

impl FieldSources {
    pub fn all_fields(&self) -> Vec<(&str, &ValueSource)> {
        vec![
            ("tq_peak", &self.tq_peak),
            ("rpm_hp", &self.rpm_hp),
            ("rpm_tq", &self.rpm_tq),
            ("redline", &self.redline),
            ("gears", &self.gears),
            ("final_drive", &self.final_drive),
            ("shift_time_s", &self.shift_time_s),
            ("cd", &self.cd),
            ("frontal_area_sqft", &self.frontal_area_sqft),
            ("tire_diameter_in", &self.tire_diameter_in),
            ("cg_height_in", &self.cg_height_in),
            ("wheelbase_in", &self.wheelbase_in),
            ("weight_dist_rear_pct", &self.weight_dist_rear_pct),
            ("tire_mu", &self.tire_mu),
            ("drivetrain_loss_pct", &self.drivetrain_loss_pct),
            ("launch_rpm", &self.launch_rpm),
        ]
    }
}

/// Internal representation with SI units, ready for the physics solver.
pub struct SimCar {
    pub name: String,
    pub mass_kg: f64,
    pub weight_n: f64,
    pub tq_peak_nm: f64,
    pub rpm_hp: f64,
    pub rpm_tq: f64,
    pub redline: f64,
    pub hp_peak: f64,
    pub gears: Vec<f64>,
    pub final_drive: f64,
    pub efficiency: f64,
    pub cd: f64,
    pub frontal_area_m2: f64,
    pub tire_radius_m: f64,
    pub shift_time_s: f64,
    pub launch_rpm: f64,
    pub cg_height_m: f64,
    pub wheelbase_m: f64,
    pub rear_weight_bias: f64,
    pub tire_mu: f64,
    pub driven_wheels: DriveLayout,
    pub rho: f64,
    pub crr: f64,
    pub rotational_inertia_base: f64,
    pub rotational_inertia_gear_factor: f64,
    pub idle_rpm: f64,
    pub idle_torque_fraction: f64,
    pub overrev_torque_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RaceTarget {
    Distance { feet: f64 },
    Speed { mph: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RacePreset {
    pub name: String,
    pub start_mph: f64,
    pub target: RaceTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResult {
    pub time_s: f64,
    pub final_speed_mph: f64,
    pub completed: bool,
    pub dnf_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceComparison {
    pub race_name: String,
    pub result_a: RaceResult,
    pub result_b: RaceResult,
    pub winner: Option<String>,
    pub margin_s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub car_a: ResolvedCar,
    pub car_b: ResolvedCar,
    pub races: Vec<RaceComparison>,
    pub confidence_a: f64,
    pub confidence_b: f64,
    pub details_a: Vec<crate::confidence::FieldDetail>,
    pub details_b: Vec<crate::confidence::FieldDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInput {
    pub name: Option<String>,
    pub air_density_kgm3: Option<f64>,
}
