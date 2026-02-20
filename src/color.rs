fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

pub fn temperature_to_rgb(kelvin: u32) -> [f64; 3] {
    let temp = (kelvin as f64) / 100.0;

    let (red, green, blue) = if temp <= 66.0 {
        let red = 255.0;
        let green = clamp(99.4708025861 * temp.ln() - 161.1195681661, 0.0, 255.0);
        let blue = if temp <= 19.0 {
            0.0
        } else {
            clamp(
                138.5177312231 * (temp - 10.0).ln() - 305.0447927307,
                0.0,
                255.0,
            )
        };
        (red, green, blue)
    } else {
        let red = clamp(
            329.698727446 * (temp - 60.0).powf(-0.1332047592),
            0.0,
            255.0,
        );
        let green = clamp(
            288.1221695283 * (temp - 60.0).powf(-0.0755148492),
            0.0,
            255.0,
        );
        let blue = 255.0;
        (red, green, blue)
    };

    [red / 255.0, green / 255.0, blue / 255.0]
}

pub fn channel_multipliers(temperature_k: u32, gamma_pct: f64, identity: bool) -> [f64; 3] {
    let gamma = gamma_pct / 100.0;

    if identity {
        [gamma, gamma, gamma]
    } else {
        let tint = temperature_to_rgb(temperature_k);
        [tint[0] * gamma, tint[1] * gamma, tint[2] * gamma]
    }
}

pub fn ctm_matrix(temperature_k: u32, gamma_pct: f64, identity: bool) -> [f64; 9] {
    let ch = channel_multipliers(temperature_k, gamma_pct, identity);

    [ch[0], 0.0, 0.0, 0.0, ch[1], 0.0, 0.0, 0.0, ch[2]]
}

pub fn identity_matrix() -> [f64; 9] {
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
}

pub fn build_gamma_lut(size: usize, multipliers: [f64; 3]) -> Vec<u8> {
    if size == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(size * 3 * 2);

    for channel in multipliers {
        for i in 0..size {
            let base = if size == 1 {
                1.0
            } else {
                i as f64 / (size as f64 - 1.0)
            };

            let value = clamp((base * 65535.0 * channel).round(), 0.0, 65535.0) as u16;
            out.extend_from_slice(&value.to_ne_bytes());
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_known_points() {
        let warm = temperature_to_rgb(1000);
        assert!((warm[0] - 1.0).abs() < 1e-6);
        assert!(warm[1] < 0.3);
        assert!(warm[2] <= 0.01);

        let neutral = temperature_to_rgb(6500);
        assert!((neutral[0] - 1.0).abs() < 0.01);
        assert!((neutral[1] - 0.99).abs() < 0.05);
        assert!((neutral[2] - 0.98).abs() < 0.08);

        let cold = temperature_to_rgb(20000);
        assert!(cold[0] < 0.8);
        assert!(cold[1] < 0.9);
        assert!((cold[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ctm_identity_behavior() {
        let matrix = ctm_matrix(1000, 120.0, true);
        assert!((matrix[0] - 1.2).abs() < 1e-9);
        assert!((matrix[4] - 1.2).abs() < 1e-9);
        assert!((matrix[8] - 1.2).abs() < 1e-9);
        assert_eq!(matrix[1], 0.0);
    }

    #[test]
    fn gamma_table_shape() {
        let lut = build_gamma_lut(256, [1.0, 0.5, 0.0]);
        assert_eq!(lut.len(), 256 * 3 * 2);

        let first_red = u16::from_ne_bytes([lut[0], lut[1]]);
        let last_red = u16::from_ne_bytes([lut[510], lut[511]]);
        assert_eq!(first_red, 0);
        assert_eq!(last_red, 65535);

        let green_start = 512;
        let green_end = green_start + 510;
        let last_green = u16::from_ne_bytes([lut[green_end], lut[green_end + 1]]);
        assert!((32760..=32768).contains(&last_green));

        let blue_start = 1024;
        let blue_end = blue_start + 510;
        let last_blue = u16::from_ne_bytes([lut[blue_end], lut[blue_end + 1]]);
        assert_eq!(last_blue, 0);
    }
}
