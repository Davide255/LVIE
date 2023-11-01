use std::ops::RangeInclusive;

pub fn norm_range_f32(r: RangeInclusive<f32>, value: f32) -> f32 {
    if r.start() <= &value && &value <= r.end() {
        return value;
    } else if &value < r.start() {
        return *r.start();
    } else {
        return *r.end();
    }
}
