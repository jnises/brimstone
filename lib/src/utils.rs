use palette::{Srgb, FromComponent, Component};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

pub(crate) fn render_par<F, T>(size: (usize, usize), buf: &mut [Srgb<T>], func: F)
where
    F: Fn(f32, f32) -> Srgb + Sync,
    T: Default + Copy + Send + FromComponent<f32> + Component,
{
    assert!(buf.len() == size.0 * size.1);
    buf.par_chunks_exact_mut(size.0)
        .enumerate()
        .for_each(|(y, row)| {
            let normy = y as f32 / size.1 as f32;
            row.iter_mut().enumerate().for_each(|(x, pixel)| {
                let normx = x as f32 / size.0 as f32;
                let p: Srgb<T> = func(normx, normy).into_format();
                *pixel = p;
            });
        });
}
