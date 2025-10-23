pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    assert!((0_f64..=1_f64).contains(&t));

    a * (1. - t) + t * b
}

pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    assert!((0_f32..=1_f32).contains(&t));

    a * (1. - t) + t * b
}
