//! This is not a real example, just a tool for comparing custom vs Quake friction code.
//!
//! I tried to make friction framerate independent.
//! If you run 30 fps, you get the same resulting velocity (ignoring float rounding errors)
//! as 60 fps because the time delta (dt) changes too and using powf assures friction applies proportionally.
//! Unfortunately I didn't realize that when integrating it to get distance, the step size still matters
//! and therefore the final position is affected by framerate.
//!
//! Quake friction uses a different formula which takes dt into account in such a way
//! that even velocity differs based on framerate so there are two places where differences accumulate.
//! Quake code: https://github.com/id-Software/Quake-III-Arena/blob/master/code/game/bg_pmove.c
//!
//! This example shows that the effect, however, is much smaller than I expected.
//! The upper table shows that I have chosen parameters for which custom and Quake give almost the same results at 60 FPS.
//! Then compare it to the lower table - it shows what happens at 15 FPS.
//! With my algorithm, the speed at a given time (compare rows with the same `i`) is the same regardless of FPS.
//! However, the total distance trvavelled at the end is different.
//! The error in Quake physics is only about twice bigger than in mine though.
fn main() {
    // Custom friction uses values between 0 and 1,
    // Quake friction can increase to infinity.
    let friction_cus: f64 = 0.97;
    let friction_q3: f64 = 3.41; // chosen to match custom
    let v0 = 100.0;
    let dt = 0.016;
    let steps = 40;

    {
        let mut v_cus = v0;
        let mut v_q3 = v0;
        let mut dist_cus = 0.0;
        let mut dist_q3 = 0.0;
        for i in (0..=steps).skip(1) {
            v_cus *= (1.0 - friction_cus).powf(dt);
            dist_cus += v_cus * dt;

            let drop = v_q3 * friction_q3 * dt;
            v_q3 = (v_q3 - drop).max(0.0);
            dist_q3 += v_q3 * dt;

            println!(
                "i: {}\tcustom: {}\tquake: {} (drop {})",
                i, v_cus, v_q3, drop
            );
        }
        println!(
            "Distance travelled: custom: {} quake: {}",
            dist_cus, dist_q3
        );
    }

    println!();

    // The longer the interval, the fewer the steps.
    let multiplier = 4;
    {
        let dt = dt * multiplier as f64;

        let mut v_cus = v0;
        let mut v_q3 = v0;
        let mut dist_cus = 0.0;
        let mut dist_q3 = 0.0;
        for i in (0..=steps).step_by(multiplier).skip(1) {
            v_cus *= (1.0 - friction_cus).powf(dt);
            dist_cus += v_cus * dt;

            let drop = v_q3 * friction_q3 * dt;
            v_q3 = (v_q3 - drop).max(0.0);
            dist_q3 += v_q3 * dt;

            println!(
                "i: {}\tcustom: {}\tquake: {} (drop {})",
                i, v_cus, v_q3, drop
            );
        }
        println!(
            "Distance travelled: custom: {} quake: {}",
            dist_cus, dist_q3
        );
    }
}
