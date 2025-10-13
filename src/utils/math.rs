pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    assert!((0_f64..=1_f64).contains(&t));

    a * (1. - t) + t * b
}
