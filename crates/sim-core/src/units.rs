pub const GRAVITY_MS2: f64 = 9.81;

pub fn lbs_to_kg(lbs: f64) -> f64 {
    lbs * 0.453592
}

pub fn inches_to_meters(inches: f64) -> f64 {
    inches * 0.0254
}

pub fn mph_to_ms(mph: f64) -> f64 {
    mph * 0.44704
}

pub fn ms_to_mph(ms: f64) -> f64 {
    ms / 0.44704
}

pub fn sqft_to_sqm(sqft: f64) -> f64 {
    sqft * 0.092903
}

pub fn ft_to_m(ft: f64) -> f64 {
    ft * 0.3048
}

pub fn lbft_to_nm(lbft: f64) -> f64 {
    lbft * 1.355818
}

/// HP = Torque(lb-ft) * RPM / 5252
pub fn hp_rpm_to_tq_lbft(hp: f64, rpm: f64) -> f64 {
    if rpm <= 0.0 {
        return 0.0;
    }
    hp * 5252.0 / rpm
}
