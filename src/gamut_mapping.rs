#![allow(clippy::excessive_precision, dead_code)]
#![deny(clippy::shadow_reuse, clippy::shadow_same, clippy::shadow_unrelated)]

// converted to rust from https://bottosson.github.io/posts/gamutclipping/

// Copyright (c) 2021 Björn Ottosson
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[derive(Clone, Copy)]
pub struct OKLab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

#[derive(Clone, Copy)]
pub struct LinearRGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub fn linear_srgb_to_oklab(c: LinearRGB) -> OKLab {
    let l = 0.4122214708f32 * c.r + 0.5363325363f32 * c.g + 0.0514459929f32 * c.b;
    let m = 0.2119034982f32 * c.r + 0.6806995451f32 * c.g + 0.1073969566f32 * c.b;
    let s = 0.0883024619f32 * c.r + 0.2817188376f32 * c.g + 0.6299787005f32 * c.b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    OKLab {
        l: 0.2104542553f32 * l_ + 0.7936177850f32 * m_ - 0.0040720468f32 * s_,
        a: 1.9779984951f32 * l_ - 2.4285922050f32 * m_ + 0.4505937099f32 * s_,
        b: 0.0259040371f32 * l_ + 0.7827717662f32 * m_ - 0.8086757660f32 * s_,
    }
}

pub fn oklab_to_linear_srgb(c: OKLab) -> LinearRGB {
    let l_ = c.l + 0.3963377774f32 * c.a + 0.2158037573f32 * c.b;
    let m_ = c.l - 0.1055613458f32 * c.a - 0.0638541728f32 * c.b;
    let s_ = c.l - 0.0894841775f32 * c.a - 1.2914855480f32 * c.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    LinearRGB {
        r: 4.0767416621f32 * l - 3.3077115913f32 * m + 0.2309699292f32 * s,
        g: -1.2684380046f32 * l + 2.6097574011f32 * m - 0.3413193965f32 * s,
        b: -0.0041960863f32 * l - 0.7034186147f32 * m + 1.7076147010f32 * s,
    }
}

// Finds the maximum saturation possible for a given hue that fits in sRGB
// Saturation here is defined as S = C/L
// a and b must be normalized so a^2 + b^2 == 1
fn compute_max_saturation(a: f32, b: f32) -> f32 {
    debug_assert!((a.powi(2) + b.powi(2) - 1.).abs() < 0.0001);
    // Max saturation will be when one of r, g or b goes below zero.

    // Select different coefficients depending on which component goes below zero first
    let k0;
    let k1;
    let k2;
    let k3;
    let k4;
    let wl;
    let wm;
    let ws;

    if -1.88170328f32 * a - 0.80936493f32 * b > 1. {
        // Red component
        k0 = 1.19086277f32;
        k1 = 1.76576728f32;
        k2 = 0.59662641f32;
        k3 = 0.75515197f32;
        k4 = 0.56771245f32;
        wl = 4.0767416621f32;
        wm = -3.3077115913f32;
        ws = 0.2309699292f32;
    } else if 1.81444104f32 * a - 1.19445276f32 * b > 1. {
        // Green component
        k0 = 0.73956515f32;
        k1 = -0.45954404f32;
        k2 = 0.08285427f32;
        k3 = 0.12541070f32;
        k4 = 0.14503204f32;
        wl = -1.2684380046f32;
        wm = 2.6097574011f32;
        ws = -0.3413193965f32;
    } else {
        // Blue component
        k0 = 1.35733652f32;
        k1 = -0.00915799f32;
        k2 = -1.15130210f32;
        k3 = -0.50559606f32;
        k4 = 0.00692167f32;
        wl = -0.0041960863f32;
        wm = -0.7034186147f32;
        ws = 1.7076147010f32;
    }

    // Approximate max saturation using a polynomial:
    let mut s = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;

    // Do one step Halley's method to get closer
    // this gives an error less than 10e6, except for some blue hues where the dS/dh is close to infinite
    // this should be sufficient for most applications, otherwise do two/three steps

    let k_l = 0.3963377774f32 * a + 0.2158037573f32 * b;
    let k_m = -0.1055613458f32 * a - 0.0638541728f32 * b;
    let k_s = -0.0894841775f32 * a - 1.2914855480f32 * b;

    for _ in 0..3 {
        let l_ = 1. + s * k_l;
        let m_ = 1. + s * k_m;
        let s_ = 1. + s * k_s;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let sh = s_ * s_ * s_;

        let l_ds = 3. * k_l * l_ * l_;
        let m_ds = 3. * k_m * m_ * m_;
        let s_ds = 3. * k_s * s_ * s_;

        let l_ds2 = 6. * k_l * k_l * l_;
        let m_ds2 = 6. * k_m * k_m * m_;
        let s_ds2 = 6. * k_s * k_s * s_;

        let f = wl * l + wm * m + ws * sh;
        let f1 = wl * l_ds + wm * m_ds + ws * s_ds;
        let f2 = wl * l_ds2 + wm * m_ds2 + ws * s_ds2;

        s = s - f * f1 / (f1 * f1 - 0.5f32 * f * f2);
    }

    s
}

// finds L_cusp and C_cusp for a given hue
// a and b must be normalized so a^2 + b^2 == 1
struct LC {
    l: f32,
    c: f32,
}

fn find_cusp(a: f32, b: f32) -> LC {
    debug_assert!((a.powi(2) + b.powi(2) - 1.).abs() < 0.0001);
    // First, find the maximum saturation (saturation S = C/L)
    let s_cusp = compute_max_saturation(a, b);

    // Convert to linear sRGB to find the first point where at least one of r,g or b >= 1:
    let rgb_at_max = oklab_to_linear_srgb(OKLab {
        l: 1.,
        a: s_cusp * a,
        b: s_cusp * b,
    });
    let l_cusp = (1. / rgb_at_max.r.max(rgb_at_max.g.max(rgb_at_max.b))).cbrt();
    let c_cusp = l_cusp * s_cusp;

    LC {
        l: l_cusp,
        c: c_cusp,
    }
}

// Finds intersection of the line defined by
// L = L0 * (1 - t) + t * L1;
// C = t * C1;
// a and b must be normalized so a^2 + b^2 == 1
fn find_gamut_intersection(a: f32, b: f32, l1: f32, c1: f32, l0: f32) -> f32 {
    debug_assert!((a.powi(2) + b.powi(2) - 1.).abs() < 0.0001);
    // Find the cusp of the gamut triangle
    let cusp = find_cusp(a, b);

    // Find the intersection for upper and lower half seprately
    if ((l1 - l0) * cusp.c - (cusp.l - l0) * c1) <= 0. {
        // Lower half

        cusp.c * l0 / (c1 * cusp.l + cusp.c * (l0 - l1))
    } else {
        // Upper half

        // First intersect with triangle
        let mut t = cusp.c * (l0 - 1.) / (c1 * (cusp.l - 1.) + cusp.c * (l0 - l1));

        // Then one step Halley's method
        {
            let dl = l1 - l0;
            let dc = c1;

            let k_l = 0.3963377774f32 * a + 0.2158037573f32 * b;
            let k_m = -0.1055613458f32 * a - 0.0638541728f32 * b;
            let k_s = -0.0894841775f32 * a - 1.2914855480f32 * b;

            let l_dt = dl + dc * k_l;
            let m_dt = dl + dc * k_m;
            let s_dt = dl + dc * k_s;

            // If higher accuracy is required, 2 or 3 iterations of the following block can be used:
            for _ in 0..3 {
                let l = l0 * (1. - t) + t * l1;
                let c = t * c1;

                let l_ = l + c * k_l;
                let m_ = l + c * k_m;
                let s_ = l + c * k_s;

                let lo = l_ * l_ * l_;
                let m = m_ * m_ * m_;
                let s = s_ * s_ * s_;

                let ldt = 3. * l_dt * l_ * l_;
                let mdt = 3. * m_dt * m_ * m_;
                let sdt = 3. * s_dt * s_ * s_;

                let ldt2 = 6. * l_dt * l_dt * l_;
                let mdt2 = 6. * m_dt * m_dt * m_;
                let sdt2 = 6. * s_dt * s_dt * s_;

                let r = 4.0767416621f32 * lo - 3.3077115913f32 * m + 0.2309699292f32 * s - 1.;
                let r1 = 4.0767416621f32 * ldt - 3.3077115913f32 * mdt + 0.2309699292f32 * sdt;
                let r2 = 4.0767416621f32 * ldt2 - 3.3077115913f32 * mdt2 + 0.2309699292f32 * sdt2;

                let u_r = r1 / (r1 * r1 - 0.5f32 * r * r2);
                let mut t_r = -r * u_r;

                let g = -1.2684380046f32 * lo + 2.6097574011f32 * m - 0.3413193965f32 * s - 1.;
                let g1 = -1.2684380046f32 * ldt + 2.6097574011f32 * mdt - 0.3413193965f32 * sdt;
                let g2 = -1.2684380046f32 * ldt2 + 2.6097574011f32 * mdt2 - 0.3413193965f32 * sdt2;

                let u_g = g1 / (g1 * g1 - 0.5f32 * g * g2);
                let mut t_g = -g * u_g;

                let b_ = -0.0041960863f32 * lo - 0.7034186147f32 * m + 1.7076147010f32 * s - 1.;
                let b1 = -0.0041960863f32 * ldt - 0.7034186147f32 * mdt + 1.7076147010f32 * sdt;
                let b2 = -0.0041960863f32 * ldt2 - 0.7034186147f32 * mdt2 + 1.7076147010f32 * sdt2;

                let u_b = b1 / (b1 * b1 - 0.5f32 * b_ * b2);
                let mut t_b = -b_ * u_b;

                t_r = if u_r >= 0. { t_r } else { f32::MAX };
                t_g = if u_g >= 0. { t_g } else { f32::MAX };
                t_b = if u_b >= 0. { t_b } else { f32::MAX };

                t += t_r.min(t_g.min(t_b));
            }
        }
        t
    }
}

/// like f32::signum except for no Nan and returns 0 if x is 0
fn sgn(x: f32) -> f32 {
    ((0. < x) as i8 - (x < 0.) as i8) as f32
}

pub fn gamut_clip_preserve_chroma(rgb: LinearRGB) -> LinearRGB {
    if rgb.r < 1. && rgb.g < 1. && rgb.b < 1. && rgb.r > 0. && rgb.g > 0. && rgb.b > 0. {
        return rgb;
    }

    let lab = linear_srgb_to_oklab(rgb);

    let l = lab.l;
    let eps = 0.00001f32;
    let c = eps.max(f32::sqrt(lab.a * lab.a + lab.b * lab.b));
    let a_ = lab.a / c;
    let b_ = lab.b / c;

    let l0 = l.clamp(0., 1.);

    let t = find_gamut_intersection(a_, b_, l, c, l0);
    let l_clipped = l0 * (1. - t) + t * l;
    let c_clipped = t * c;

    oklab_to_linear_srgb(OKLab {
        l: l_clipped,
        a: c_clipped * a_,
        b: c_clipped * b_,
    })
}

pub fn gamut_clip_project_to_0_5(rgb: LinearRGB) -> LinearRGB {
    if rgb.r < 1. && rgb.g < 1. && rgb.b < 1. && rgb.r > 0. && rgb.g > 0. && rgb.b > 0. {
        return rgb;
    }

    let lab = linear_srgb_to_oklab(rgb);

    let l = lab.l;
    let eps = 0.00001f32;
    let c = eps.max(f32::sqrt(lab.a * lab.a + lab.b * lab.b));
    let a_ = lab.a / c;
    let b_ = lab.b / c;

    let l0 = 0.5;

    let t = find_gamut_intersection(a_, b_, l, c, l0);
    let l_clipped = l0 * (1. - t) + t * l;
    let c_clipped = t * c;

    oklab_to_linear_srgb(OKLab {
        l: l_clipped,
        a: c_clipped * a_,
        b: c_clipped * b_,
    })
}

pub fn gamut_clip_project_to_l_cusp(rgb: LinearRGB) -> LinearRGB {
    if rgb.r < 1. && rgb.g < 1. && rgb.b < 1. && rgb.r > 0. && rgb.g > 0. && rgb.b > 0. {
        return rgb;
    }

    let lab = linear_srgb_to_oklab(rgb);

    let l = lab.l;
    let eps = 0.00001f32;
    let c = eps.max(f32::sqrt(lab.a * lab.a + lab.b * lab.b));
    let a_ = lab.a / c;
    let b_ = lab.b / c;

    // The cusp is computed here and in find_gamut_intersection, an optimized solution would only compute it once.
    let cusp = find_cusp(a_, b_);

    let l0 = cusp.l;

    let t = find_gamut_intersection(a_, b_, l, c, l0);

    let l_clipped = l0 * (1. - t) + t * l;
    let c_clipped = t * c;

    oklab_to_linear_srgb(OKLab {
        l: l_clipped,
        a: c_clipped * a_,
        b: c_clipped * b_,
    })
}

pub fn gamut_clip_adaptive_l0_0_5(rgb: LinearRGB) -> LinearRGB {
    gamut_clip_adaptive_l0_0_5_alpha(rgb, 0.05)
}

pub fn gamut_clip_adaptive_l0_0_5_alpha(rgb: LinearRGB, alpha: f32) -> LinearRGB {
    if rgb.r < 1. && rgb.g < 1. && rgb.b < 1. && rgb.r > 0. && rgb.g > 0. && rgb.b > 0. {
        return rgb;
    }

    let lab = linear_srgb_to_oklab(rgb);

    let l = lab.l;
    let eps = 0.00001f32;
    let c = f32::sqrt(lab.a.powi(2) + lab.b.powi(2));
    if c < eps {
        return oklab_to_linear_srgb(OKLab {
            l: l.clamp(0., 1.),
            a: lab.a,
            b: lab.b,
        });
    }
    let a_ = lab.a / c;
    let b_ = lab.b / c;

    let ld = l - 0.5;
    let e1 = 0.5 + ld.abs() + alpha * c;
    let l0 = 0.5 * (1. + sgn(ld) * (e1 - f32::sqrt(e1 * e1 - 2. * ld.abs())));

    let t = find_gamut_intersection(a_, b_, l, c, l0);
    let l_clipped = l0 * (1. - t) + t * l;
    let c_clipped = t * c;

    oklab_to_linear_srgb(OKLab {
        l: l_clipped,
        a: c_clipped * a_,
        b: c_clipped * b_,
    })
}

pub fn gamut_clip_adaptive_l0_l_cusp(rgb: LinearRGB) -> LinearRGB {
    gamut_clip_adaptive_l0_l_cusp_alpha(rgb, 0.05)
}

pub fn gamut_clip_adaptive_l0_l_cusp_alpha(rgb: LinearRGB, alpha: f32) -> LinearRGB {
    if rgb.r < 1. && rgb.g < 1. && rgb.b < 1. && rgb.r > 0. && rgb.g > 0. && rgb.b > 0. {
        return rgb;
    }

    let lab = linear_srgb_to_oklab(rgb);

    let l = lab.l;
    let eps = 0.00001f32;
    let c = eps.max(f32::sqrt(lab.a * lab.a + lab.b * lab.b));
    let a_ = lab.a / c;
    let b_ = lab.b / c;

    // The cusp is computed here and in find_gamut_intersection, an optimized solution would only compute it once.
    let cusp = find_cusp(a_, b_);

    let ld = l - cusp.l;
    let k = 2. * if ld > 0. { 1. - cusp.l } else { cusp.l };

    let e1 = 0.5 * k + ld.abs() + alpha * c / k;
    let l_0 = cusp.l + 0.5 * (sgn(ld) * (e1 - f32::sqrt(e1 * e1 - 2. * k * ld.abs())));

    let t = find_gamut_intersection(a_, b_, l, c, l_0);
    let l_clipped = l_0 * (1. - t) + t * l;
    let c_clipped = t * c;

    oklab_to_linear_srgb(OKLab {
        l: l_clipped,
        a: c_clipped * a_,
        b: c_clipped * b_,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn origo() {
        let linsrgb = oklab_to_linear_srgb(OKLab {
            l: 0.,
            a: 0.,
            b: 0.,
        });
        let clipped = gamut_clip_adaptive_l0_0_5(linsrgb);
        assert!(clipped.r.abs() < 0.0001);
        assert!(clipped.g.abs() < 0.0001);
        assert!(clipped.b.abs() < 0.0001);
    }
}
