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
