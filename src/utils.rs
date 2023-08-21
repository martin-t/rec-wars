/// Maps values between the src and dest range: `src_min` to `dest_min`, `src_max` to `dest_max`, values between them
/// to the corresponding values between and extrapolates for values outside the range.
///
/// `src_min` and `src_max` must not be the same or division by zero occurs.
///
/// `dest_max` can be smaller than `dest_min` if you want the resulting range to be inverted, all values can be negative.
pub fn lerp_ranges(src_min: f64, src_max: f64, dest_min: f64, dest_max: f64, value: f64) -> f64 {
    let src_diff = src_max - src_min;
    let dest_diff = dest_max - dest_min;
    let ratio = (value - src_min) / src_diff;
    dest_min + dest_diff * ratio
}
