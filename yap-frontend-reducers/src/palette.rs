//! The color palette shared by the web and iOS apps, plus the OKLCH math.
//!
//! This file is compiled twice: as a module of the library, and by `build.rs`
//! (via `#[path]`) so it can write `yap-frontend/src/tokens.css` whenever the
//! workspace builds. Keep it free of dependencies for that reason; the
//! bridged types live in `design.rs`.

use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
    pub alpha: f64,
}

pub const fn oklch(l: f64, c: f64, h: f64) -> Oklch {
    Oklch {
        l,
        c,
        h,
        alpha: 1.0,
    }
}

impl Oklch {
    pub const fn with_alpha(self, alpha: f64) -> Self {
        Self { alpha, ..self }
    }

    /// Gamma-encoded sRGB, clamped to the sRGB gamut.
    pub fn srgb(self) -> [f64; 3] {
        let a = self.c * self.h.to_radians().cos();
        let b = self.c * self.h.to_radians().sin();
        let l = (self.l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
        let m = (self.l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
        let s = (self.l - 0.0894841775 * a - 1.2914855480 * b).powi(3);
        [
            4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        ]
        .map(|c| {
            let c = if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            };
            c.clamp(0.0, 1.0)
        })
    }
}

pub struct CssToken {
    pub name: &'static str,
    pub light: Oklch,
    pub dark: Oklch,
}

/// Invokes `$callback!` with the palette, so the list below is written once
/// and expanded into both the CSS token list and the bridged `Palette` struct.
macro_rules! with_palette {
    ($callback:ident) => {
        $callback! {
            background "--background": oklch(1.0, 0.0, 0.0), oklch(0.145, 0.0, 0.0);
            foreground "--foreground": oklch(0.2176, 0.0987, 328.6), oklch(0.9102, 0.0375, 328.6);
            card "--card": oklch(1.0, 0.0, 0.0), oklch(0.205, 0.0, 0.0);
            card_foreground "--card-foreground": oklch(0.2176, 0.0987, 328.6), oklch(0.9102, 0.0375, 328.6);
            popover "--popover": oklch(1.0, 0.0, 0.0), oklch(0.205, 0.0, 0.0);
            popover_foreground "--popover-foreground": oklch(0.2176, 0.0987, 328.6), oklch(0.9102, 0.0375, 328.6);
            primary "--primary": oklch(0.2176, 0.0987, 328.6), oklch(0.9102, 0.0375, 328.6);
            primary_foreground "--primary-foreground": oklch(0.985, 0.0, 0.0), oklch(0.205, 0.0, 0.0);
            secondary "--secondary": oklch(0.97, 0.0, 0.0), oklch(0.269, 0.0, 0.0);
            secondary_foreground "--secondary-foreground": oklch(0.2176, 0.0987, 328.6), oklch(0.9102, 0.0375, 328.6);
            muted "--muted": oklch(0.97, 0.0, 0.0), oklch(0.269, 0.0, 0.0);
            muted_foreground "--muted-foreground": oklch(0.2906, 0.0738, 328.6), oklch(0.749, 0.0206, 328.6);
            accent "--accent": oklch(0.97, 0.0, 0.0), oklch(0.269, 0.0, 0.0);
            accent_foreground "--accent-foreground": oklch(0.3176, 0.1987, 328.6), oklch(0.8102, 0.1375, 328.6);
            destructive "--destructive": oklch(0.577, 0.245, 27.325), oklch(0.704, 0.191, 22.216);
            border "--border": oklch(0.8933, 0.0752, 0.0), oklch(1.0, 0.0, 0.0).with_alpha(0.1);
            input "--input": oklch(0.8933, 0.0752, 0.0), oklch(1.0, 0.0, 0.0).with_alpha(0.15);
            ring "--ring": oklch(0.708, 0.0, 0.0), oklch(0.556, 0.0, 0.0);
            chart_1 "--chart-1": oklch(0.646, 0.222, 41.116), oklch(0.488, 0.243, 264.376);
            chart_2 "--chart-2": oklch(0.6, 0.118, 184.704), oklch(0.696, 0.17, 162.48);
            chart_3 "--chart-3": oklch(0.398, 0.07, 227.392), oklch(0.769, 0.188, 70.08);
            chart_4 "--chart-4": oklch(0.828, 0.189, 84.429), oklch(0.627, 0.265, 303.9);
            chart_5 "--chart-5": oklch(0.769, 0.188, 70.08), oklch(0.645, 0.246, 16.439);
            sidebar "--sidebar": oklch(0.985, 0.0, 0.0), oklch(0.205, 0.0, 0.0);
            sidebar_foreground "--sidebar-foreground": oklch(0.2176, 0.0987, 328.6), oklch(0.9102, 0.0375, 328.6);
            sidebar_primary "--sidebar-primary": oklch(0.205, 0.0, 0.0), oklch(0.488, 0.243, 264.376);
            sidebar_primary_foreground "--sidebar-primary-foreground": oklch(0.985, 0.0, 0.0), oklch(0.9102, 0.0375, 328.6);
            sidebar_accent "--sidebar-accent": oklch(0.97, 0.0, 0.0), oklch(0.269, 0.0, 0.0);
            sidebar_accent_foreground "--sidebar-accent-foreground": oklch(0.2176, 0.0987, 328.6), oklch(0.9102, 0.0375, 328.6);
            sidebar_border "--sidebar-border": oklch(0.8933, 0.0752, 0.0), oklch(1.0, 0.0, 0.0).with_alpha(0.1);
            sidebar_ring "--sidebar-ring": oklch(0.708, 0.0, 0.0), oklch(0.556, 0.0, 0.0);
        }
    };
}

macro_rules! css_token_list {
    ($($field:ident $name:literal: $light:expr, $dark:expr;)*) => {
        pub fn css_tokens() -> Vec<CssToken> {
            vec![$(CssToken { name: $name, light: $light, dark: $dark },)*]
        }
    };
}
with_palette!(css_token_list);

pub fn render_css() -> String {
    let mut css = String::from(
        "/* Generated by yap-frontend-reducers/build.rs from src/palette.rs. Do not edit. */\n",
    );
    let tokens = css_tokens();
    for (selector, dark) in [(":root", false), (".dark", true)] {
        writeln!(css, "\n{selector} {{").unwrap();
        for token in &tokens {
            let Oklch { l, c, h, alpha } = if dark { token.dark } else { token.light };
            write!(css, "  {}: oklch({l} {c} {h}", token.name).unwrap();
            if alpha != 1.0 {
                write!(css, " / {alpha}").unwrap();
            }
            css.push_str(");\n");
        }
        css.push_str("}\n");
    }
    css
}
