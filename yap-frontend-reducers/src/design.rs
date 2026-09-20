//! Design colors for both frontends. The palette in `palette.rs` is the single
//! source of truth: iOS reads `design_palette()` through the Swift bindings, and
//! the web imports the gitignored `yap-frontend/src/tokens.css`, which
//! `build.rs` rewrites from the same list whenever this crate builds.

use crate::palette::Oklch;

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl From<Oklch> for Rgba {
    fn from(value: Oklch) -> Self {
        let [r, g, b] = value.srgb();
        Self {
            r,
            g,
            b,
            a: value.alpha.clamp(0.0, 1.0),
        }
    }
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub struct DynamicColor {
    pub light: Rgba,
    pub dark: Rgba,
}

macro_rules! bridged_palette {
    ($($field:ident $name:literal: $light:expr, $dark:expr;)*) => {
        #[bridgerton::bridge(transparent)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct Palette {
            $(pub $field: DynamicColor,)*
        }

        #[bridgerton::bridge]
        pub fn design_palette() -> Palette {
            use crate::palette::oklch;
            Palette { $($field: DynamicColor { light: ($light).into(), dark: ($dark).into() },)* }
        }
    };
}
with_palette!(bridged_palette);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{css_tokens, oklch, render_css};

    #[test]
    fn css_names_are_unique_and_valid() {
        let mut names = std::collections::HashSet::new();
        for token in css_tokens() {
            assert!(names.insert(token.name));
            let name = token.name.strip_prefix("--").unwrap();
            assert!(name.as_bytes()[0].is_ascii_lowercase());
            assert!(name.split('-').all(|part| {
                !part.is_empty()
                    && part
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            }));
        }
        assert_eq!(names.len(), 31);
    }

    #[test]
    fn conversions_and_css() {
        let white = Rgba::from(oklch(1.0, 0.0, 0.0));
        for c in [white.r, white.g, white.b, white.a] {
            assert!((c - 1.0).abs() < 1e-6);
        }
        let gray = Rgba::from(oklch(0.145, 0.0, 0.0));
        for c in [gray.r, gray.g, gray.b] {
            assert!((c - 0.039388235).abs() < 1e-6);
        }
        let accent = Rgba::from(oklch(0.3176, 0.1987, 328.6));
        assert!(accent.r > 0.3 && accent.g == 0.0 && accent.b > 0.3);
        let css = render_css();
        assert!(css.contains("--accent-foreground: oklch(0.3176 0.1987 328.6);"));
        assert!(css.contains(".dark {"));
        assert!(css.contains("--border: oklch(1 0 0 / 0.1);"));
    }
}
