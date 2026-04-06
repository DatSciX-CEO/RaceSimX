use crate::types::CarInput;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

pub fn validate_car_input(car: &CarInput) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    if car.weight_lbs <= 0.0 {
        warnings.push(warn("weight_lbs", "must be positive"));
    }
    if car.hp_peak <= 0.0 {
        warnings.push(warn("hp_peak", "must be positive"));
    }
    if car.weight_lbs > 15_000.0 {
        warnings.push(warn(
            "weight_lbs",
            "unusually high (> 15 000 lbs) — double-check value",
        ));
    }
    if car.hp_peak > 5_000.0 {
        warnings.push(warn(
            "hp_peak",
            "unusually high (> 5 000 HP) — double-check value",
        ));
    }

    if let Some(tq) = car.tq_peak {
        if tq <= 0.0 {
            warnings.push(warn("tq_peak", "must be positive"));
        }
    }
    if let Some(rpm) = car.rpm_hp {
        if rpm < 1000.0 || rpm > 15_000.0 {
            warnings.push(warn("rpm_hp", "expected between 1 000 and 15 000"));
        }
    }
    if let Some(rpm) = car.rpm_tq {
        if rpm < 500.0 || rpm > 12_000.0 {
            warnings.push(warn("rpm_tq", "expected between 500 and 12 000"));
        }
    }
    if let Some(red) = car.redline {
        if red < 2000.0 || red > 20_000.0 {
            warnings.push(warn("redline", "expected between 2 000 and 20 000"));
        }
    }

    if let (Some(rpm_tq), Some(rpm_hp)) = (car.rpm_tq, car.rpm_hp) {
        if rpm_tq >= rpm_hp {
            warnings.push(warn("rpm_tq", "peak torque RPM should be lower than peak HP RPM"));
        }
    }
    if let (Some(redline), Some(rpm_hp)) = (car.redline, car.rpm_hp) {
        if redline <= rpm_hp {
            warnings.push(warn("redline", "redline should be above peak HP RPM"));
        }
    }

    if let Some(gears) = &car.gears {
        if gears.is_empty() {
            warnings.push(warn("gears", "must have at least one gear"));
        }
        for (i, &g) in gears.iter().enumerate() {
            if g <= 0.0 {
                warnings.push(warn("gears", &format!("gear {i} ratio must be positive")));
            }
        }
        for w in gears.windows(2) {
            if w[1] >= w[0] {
                warnings.push(warn(
                    "gears",
                    "ratios should be descending (highest first)",
                ));
                break;
            }
        }
    }
    if let Some(fd) = car.final_drive {
        if fd <= 0.0 {
            warnings.push(warn("final_drive", "must be positive"));
        }
    }

    if let Some(cd) = car.cd {
        if cd < 0.1 || cd > 1.0 {
            warnings.push(warn("cd", "expected between 0.10 and 1.00"));
        }
    }
    if let Some(fa) = car.frontal_area_sqft {
        if fa < 10.0 || fa > 50.0 {
            warnings.push(warn("frontal_area_sqft", "expected between 10 and 50 sq ft"));
        }
    }
    if let Some(td) = car.tire_diameter_in {
        if td < 15.0 || td > 40.0 {
            warnings.push(warn("tire_diameter_in", "expected between 15 and 40 inches"));
        }
    }
    if let Some(mu) = car.tire_mu {
        if mu < 0.3 || mu > 2.5 {
            warnings.push(warn("tire_mu", "expected between 0.3 and 2.5"));
        }
    }
    if let Some(pct) = car.weight_dist_rear_pct {
        if pct < 20.0 || pct > 80.0 {
            warnings.push(warn(
                "weight_dist_rear_pct",
                "expected between 20% and 80%",
            ));
        }
    }
    if let Some(loss) = car.drivetrain_loss_pct {
        if loss < 0.0 || loss > 50.0 {
            warnings.push(warn(
                "drivetrain_loss_pct",
                "expected between 0% and 50%",
            ));
        }
    }

    warnings
}

fn warn(field: &str, message: &str) -> ValidationWarning {
    ValidationWarning {
        field: field.to_string(),
        message: message.to_string(),
    }
}
