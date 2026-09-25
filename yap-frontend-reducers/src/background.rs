//! Background band colors and static fallback, shared by both renderers.
use crate::Rgba;

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub enum BackgroundTheme {
    Light,
    Dark,
    Oled,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BackgroundPalette {
    /// Packed sRGB triples, suitable for SwiftUI's floatArray and worker messages.
    pub colors: Vec<f64>,
    pub num_bands: u32,
    pub fallback: Rgba,
}

#[bridgerton::bridge]
pub fn background_palette(theme: BackgroundTheme) -> BackgroundPalette {
    let (lightness, chroma, shift, hue_start, hue_range) = match theme {
        BackgroundTheme::Dark => (15.0, 9.0, 7.0, 5.2, 3.0),
        BackgroundTheme::Oled => (5.0, 0.0, 15.0, 3.2, -3.0),
        BackgroundTheme::Light => (78.0, 35.0, 12.0, 3.2, -3.0),
    };
    let num_bands = 6;
    let colors: Vec<_> = (0..num_bands)
        .flat_map(|i| {
            let band = f64::from(i) / f64::from(num_bands);
            let hue = (hue_start + band * hue_range).rem_euclid(std::f64::consts::TAU);
            lch_to_rgb(lightness + (band - 0.5) * shift, chroma, hue)
        })
        .collect();
    let [r, g, b] = match theme {
        BackgroundTheme::Dark => [6.0 / 255.0, 3.0 / 255.0, 7.0 / 255.0],
        BackgroundTheme::Oled => [0.0; 3],
        BackgroundTheme::Light => [colors[3], colors[4], colors[5]],
    };
    BackgroundPalette {
        colors,
        num_bands,
        fallback: Rgba { r, g, b, a: 1.0 },
    }
}

// This is CIE LCh (D65), not the OKLCH design-token color space.
fn lch_to_rgb(l: f64, c: f64, h: f64) -> [f64; 3] {
    let a = c * h.cos();
    let b = c * h.sin();
    let lab_f_inv = |t: f64| {
        if t > 0.206893 {
            t * t * t
        } else {
            (t - 16.0 / 116.0) / 7.787
        }
    };
    let fy = (l + 16.0) / 116.0;
    let x = 0.95047 * lab_f_inv(a / 500.0 + fy);
    let y = lab_f_inv(fy);
    let z = 1.08883 * lab_f_inv(fy - b / 200.0);
    [
        3.2404542 * x - 1.5371385 * y - 0.4985314 * z,
        -0.969266 * x + 1.8760108 * y + 0.041556 * z,
        0.0556434 * x - 0.2040259 * y + 1.0572252 * z,
    ]
    .map(|c| {
        let srgb = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        srgb.clamp(0.0, 1.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // Captured by running the original shader-colors.ts before its removal.
    #[test]
    fn matches_original_typescript_palettes() {
        let palette = background_palette(BackgroundTheme::Dark);
        assert_eq!(palette.num_bands, 6);
        for (actual, expected) in palette.colors.iter().zip([
            0.12090034227076699,
            0.11393758157404854,
            0.1620363211900731,
            0.15751776482797916,
            0.11565147400521314,
            0.15601163855666073,
            0.18403846931377218,
            0.12065189477298785,
            0.14318831450607092,
            0.19839602707706783,
            0.1307167266442017,
            0.128812312430648,
            0.20073797387962994,
            0.14543778502362104,
            0.11855066026468128,
            0.19297016945869364,
            0.16276966547781666,
            0.11743724612474266,
        ]) {
            assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
        }
        let expected = [
            0.023529411764705882,
            0.011764705882352941,
            0.027450980392156862,
        ];
        for (actual, expected) in [palette.fallback.r, palette.fallback.g, palette.fallback.b]
            .into_iter()
            .zip(expected)
        {
            assert!((actual - expected).abs() < 1e-12);
        }
        let palette = background_palette(BackgroundTheme::Light);
        assert_eq!(palette.num_bands, 6);
        for (actual, expected) in palette.colors.iter().zip([
            0.3290747510015879,
            0.7602454822987482,
            0.7023600120532683,
            0.49500915620639113,
            0.7726045806330549,
            0.6016726722095254,
            0.6593589899770746,
            0.7715839258041745,
            0.5262991599279953,
            0.813332623585101,
            0.758533669847431,
            0.5020161321916833,
            0.9456183365504319,
            0.7386018845997597,
            0.5419130280102092,
            1.0,
            0.7227061432027628,
            0.6397979220674587,
        ]) {
            assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
        }
        let expected = [0.49500915620639113, 0.7726045806330549, 0.6016726722095254];
        for (actual, expected) in [palette.fallback.r, palette.fallback.g, palette.fallback.b]
            .into_iter()
            .zip(expected)
        {
            assert!((actual - expected).abs() < 1e-12);
        }
        let palette = background_palette(BackgroundTheme::Oled);
        assert_eq!(palette.num_bands, 6);
        for (actual, expected) in palette.colors.iter().zip([
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.03575809584558219,
            0.035758092457431286,
            0.03575809361792445,
            0.06603052129387621,
            0.06603051651559136,
            0.06603051815222657,
            0.08830662291990568,
            0.08830661726216035,
            0.08830661920002414,
            0.10770341510896142,
            0.1077034086854308,
            0.10770341088558769,
        ]) {
            assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
        }
        let expected = [0.0, 0.0, 0.0];
        for (actual, expected) in [palette.fallback.r, palette.fallback.g, palette.fallback.b]
            .into_iter()
            .zip(expected)
        {
            assert!((actual - expected).abs() < 1e-12);
        }
    }
}
