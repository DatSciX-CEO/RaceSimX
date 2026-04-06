use crate::types::{ResolvedCar, ValueSource};

/// Weighted importance of each optional field for simulation accuracy.  These
/// do not live in external config because they are intrinsic to the physics
/// model, not tuneable assumptions.
fn field_weight(name: &str) -> f64 {
    match name {
        "gears" | "final_drive" => 10.0,
        "tq_peak" | "rpm_hp" => 8.0,
        "cd" | "frontal_area_sqft" | "tire_mu" => 7.0,
        "drivetrain_loss_pct" => 6.0,
        "tire_diameter_in" | "redline" => 5.0,
        "rpm_tq" | "shift_time_s" => 4.0,
        "weight_dist_rear_pct" | "wheelbase_in" | "cg_height_in" => 3.0,
        "launch_rpm" => 2.0,
        _ => 1.0,
    }
}

/// Returns a 0.0–1.0 confidence score for a resolved car.  1.0 means every
/// optional field was explicitly provided by the user; 0.0 means everything
/// was estimated.
pub fn car_confidence(car: &ResolvedCar) -> f64 {
    let fields = car.sources.all_fields();
    let mut provided_weight = 0.0;
    let mut total_weight = 0.0;

    for (name, source) in &fields {
        let w = field_weight(name);
        total_weight += w;
        if matches!(source, ValueSource::Provided) {
            provided_weight += w;
        }
    }

    // The three required fields (weight, hp, driven_wheels) are always
    // provided, so give them credit too.
    let required_weight = 10.0 + 10.0 + 5.0; // weight_lbs, hp_peak, driven_wheels
    provided_weight += required_weight;
    total_weight += required_weight;

    if total_weight <= 0.0 {
        return 0.0;
    }
    provided_weight / total_weight
}

/// Per-field breakdown for the UI.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldDetail {
    pub field: String,
    pub source: String,
    pub impact: String,
    pub provided: bool,
}

pub fn field_details(car: &ResolvedCar) -> Vec<FieldDetail> {
    let mut out = Vec::new();
    for (name, source) in car.sources.all_fields() {
        let w = field_weight(name);
        let impact = if w >= 8.0 {
            "critical"
        } else if w >= 6.0 {
            "high"
        } else if w >= 4.0 {
            "medium"
        } else {
            "low"
        };
        out.push(FieldDetail {
            field: name.to_string(),
            source: source.to_string(),
            impact: impact.to_string(),
            provided: matches!(source, ValueSource::Provided),
        });
    }
    out
}
