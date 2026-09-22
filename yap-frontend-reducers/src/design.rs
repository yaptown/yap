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
        assert_eq!(names.len(), 57);
    }

    #[test]
    fn feedback_roles_match_css_and_preserve_mode_and_alpha() {
        let palette = design_palette();
        let tokens = css_tokens();
        let rgba = |color: Rgba| [color.r, color.g, color.b, color.a];
        for (family, roles) in [
            (
                "positive",
                [
                    palette.positive,
                    palette.positive_foreground,
                    palette.positive_surface,
                    palette.positive_border,
                    palette.positive_field,
                ],
            ),
            (
                "caution",
                [
                    palette.caution,
                    palette.caution_foreground,
                    palette.caution_surface,
                    palette.caution_border,
                    palette.caution_field,
                ],
            ),
            (
                "warning",
                [
                    palette.warning,
                    palette.warning_foreground,
                    palette.warning_surface,
                    palette.warning_border,
                    palette.warning_field,
                ],
            ),
            (
                "negative",
                [
                    palette.negative,
                    palette.negative_foreground,
                    palette.negative_surface,
                    palette.negative_border,
                    palette.negative_field,
                ],
            ),
            (
                "info",
                [
                    palette.info,
                    palette.info_foreground,
                    palette.info_surface,
                    palette.info_border,
                    palette.info_field,
                ],
            ),
        ] {
            for (suffix, color) in ["", "-foreground", "-surface", "-border", "-field"]
                .into_iter()
                .zip(roles)
            {
                let name = format!("--{family}{suffix}");
                let token = tokens.iter().find(|token| token.name == name).unwrap();
                assert_eq!(rgba(color.light), rgba(token.light.into()), "{name} light");
                assert_eq!(rgba(color.dark), rgba(token.dark.into()), "{name} dark");
                let expected_alpha = match suffix {
                    "-surface" => 0.1,
                    "-border" => 0.2,
                    _ => 1.0,
                };
                assert_eq!(color.light.a, expected_alpha, "{name} light alpha");
                assert_eq!(color.dark.a, expected_alpha, "{name} dark alpha");
            }
            let [solid, _, surface, border, _] = roles;
            assert_eq!(rgba(solid.light), rgba(solid.dark), "{family} solid");
            for (color, alpha) in [(surface, 0.1), (border, 0.2)] {
                let expected = [solid.light.r, solid.light.g, solid.light.b, alpha];
                assert_eq!(rgba(color.light), expected, "{family} light tint");
                assert_eq!(rgba(color.dark), expected, "{family} dark tint");
            }
            let foreground = tokens
                .iter()
                .find(|t| t.name == format!("--{family}-foreground"))
                .unwrap();
            let field = tokens
                .iter()
                .find(|t| t.name == format!("--{family}-field"))
                .unwrap();
            assert!(
                foreground.light.l < foreground.dark.l,
                "{family} foreground adapts"
            );
            assert!(field.light.l > field.dark.l, "{family} field adapts");
        }
    }

    #[test]
    fn destructive_foreground_is_always_white() {
        let color = design_palette().destructive_foreground;
        for mode in [color.light, color.dark] {
            for channel in [mode.r, mode.g, mode.b, mode.a] {
                assert!((channel - 1.0).abs() < 1e-6);
            }
        }
        assert_eq!(
            render_css()
                .matches("--destructive-foreground: oklch(1 0 0);")
                .count(),
            2
        );
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
