use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::{ParallelSlice, ParallelSliceMut},
};
use std::ops;

/// approximation of gaussian blur
pub fn gaussian_blur<T>(buf: &mut [T], w: usize, h: usize, sigma: f32)
where
    T: Copy
        + ops::Sub<Output = T>
        + ops::AddAssign
        + ops::SubAssign
        + ops::Div<f32, Output = T>
        + Default
        + Sync
        + Send,
{
    debug_assert!(buf.len() == w * h);
    // TODO use non-in-place transpose to allow non-square
    assert!(w == h, "we only handle square input currently");
    for fw in box_widths_for_gauss_3(sigma) {
        average_filter_x(buf, w, h, fw);
    }
    transpose_square(buf, w);
    for fw in box_widths_for_gauss_3(sigma) {
        average_filter_x(buf, w, h, fw);
    }
    transpose_square(buf, w);
}

fn transpose_square<T>(buf: &mut [T], size: usize) {
    debug_assert!(buf.len() == size.pow(2));
    for y in 0..(size - 1) {
        for x in (y + 1)..size {
            // TODO unchecked?
            buf.swap(y * size + x, x * size + y);
        }
    }
}

fn box_widths_for_gauss_3(sigma: f32) -> [u32; 3] {
    // https://www.peterkovesi.com/papers/FastGaussianSmoothing.pdf
    const N: f32 = 3.;
    let ideal_width = (12. * sigma.powi(2) / N + 1.).sqrt();
    // ideal is a real value, but the filter with should be odd whole numbers. so we use two different widths to approximate
    let wl = {
        let w = ideal_width as u32;
        if w % 2 == 0 {
            w - 1
        } else {
            w
        }
    };
    let wu = wl + 2;
    let m = ((12. * sigma.powi(2) - (N * wl.pow(2) as f32 - 4. * N * wl as f32 - 3. * N))
        / (-4. * wl as f32 - 4.))
        .round() as u32;
    [
        // TODO can the first one ever be wu?
        if 0 < m { wl } else { wu },
        if 1 < m { wl } else { wu },
        // TODO can the last one ever be wl?
        if 2 < m { wl } else { wu },
    ]
}

fn average_filter_x<T>(buf: &mut [T], w: usize, h: usize, filter_width: u32)
where
    T: Copy
        + ops::Sub<Output = T>
        + ops::AddAssign
        + ops::SubAssign
        + ops::Div<f32, Output = T>
        + Default
        + Sync
        + Send,
{
    debug_assert!(buf.len() == w * h);
    assert!(filter_width % 2 == 1);
    if filter_width == 1 {
        return;
    }
    let rd = (filter_width as usize - 1) / 2;
    let mut tmp = vec![];
    tmp.extend_from_slice(buf);
    tmp.par_chunks_exact(w)
        .zip(buf.par_chunks_exact_mut(w))
        .for_each(|(inp, out)| {
            let mut acc = T::default();
            for _ in 0..rd {
                acc += inp[0];
            }
            for i in &inp[0..=rd] {
                acc += *i;
            }
            for x in 0..w {
                out[x] = acc / filter_width as f32;
                acc += if x >= w - rd - 1 {
                    *inp.last().unwrap()
                } else {
                    inp[x + rd + 1]
                };
                acc -= if x < rd { inp[0] } else { inp[x - rd] };
            }
        });
}
