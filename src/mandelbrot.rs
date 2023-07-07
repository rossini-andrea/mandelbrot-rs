use std::vec::Vec;
use sdl2::rect::Rect;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub trait Arithmetic:
    'static +
    std::ops::Mul<Output=Self> +
    std::ops::Div<Output=Self> +
    std::ops::Add<Output=Self> +
    std::ops::Sub<Output=Self> +
    std::cmp::PartialOrd<Self> +
    std::convert::From<f32> +
    std::convert::From<i32> +
    std::convert::From<u32> +
    std::marker::Copy +
    std::marker::Send
{

}

impl <T:
    'static +
    std::ops::Mul<Output=Self> +
    std::ops::Div<Output=Self> +
    std::ops::Add<Output=Self> +
    std::ops::Sub<Output=Self> +
    std::cmp::PartialOrd<Self> +
    std::convert::From<f32> +
    std::convert::From<i32> +
    std::convert::From<u32> +
    std::marker::Copy +
    std::marker::Send
    > Arithmetic for T
{

}

#[derive(Clone, Copy, Debug)]
pub struct Sector<Real: Arithmetic> {
    left: Real,
    bottom: Real,
    scale: Real,
    w: usize,
    h: usize,
}

impl<Real: Arithmetic> Sector<Real> {
    pub fn new(left: Real, bottom: Real, scale: Real, w: usize, h: usize) -> Self {
        Self { left, bottom, scale, w, h }
    }

    pub async fn compute(self, maxiter: usize, palette: &Vec<(u8, u8, u8)>, ct: CancellationToken) -> Option<Vec<(u8, u8, u8)>> {
        let Some((set, hist)) = compute_set_inner(self.left, self.bottom, self.scale, self.w, self.h, maxiter, ct.clone())
        .await else {
            return None;
        };
        let mut color_remap = vec![0usize; maxiter + 1];
        let pixel_count = self.w * self.h;

        if ct.is_cancelled() {
            return None;
        }

        let buf: Vec<(u8, u8, u8)> = set.into_iter().map(|(b, i)| {
            let color_index = if b {
                pixel_count 
            } else if color_remap[i] == 0 {
                let c = hist.clone().into_iter().take(i).sum();
                color_remap[i] = c;
                c
            } else {
                color_remap[i]
            } * (palette.len() - 1) / pixel_count;

            palette[palette.len() - color_index - 1]
        }).collect();

        if ct.is_cancelled() {
            return None;
        }

        Some(buf)
    }

    pub fn zoom_to_selection(&self, selection: Rect) -> Self {
        Self::new(
            self.left + Real::from(selection.left()) * self.scale,
            self.bottom + Real::from(selection.top()) * self.scale,
            self.scale * Real::from(selection.width()) / (self.w as u32).into(),
            self.w,
            self.h
        )
    }

    pub fn fit_size(&self, w: usize, h: usize) -> Self {
        Self::new(
            self.left - Real::from(w as i32 - self.w as i32) * self.scale / 2.0f32.into(),
            self.bottom - Real::from(h as i32- self.h as i32) * self.scale / 2.0f32.into(),
            self.scale,
            w,
            h
        )
    }
}

fn bounded<Real: Arithmetic>((a, b): (Real, Real), maxiter: usize) -> (bool, usize) {
    let mut z: (Real, Real) = (0f32.into(), 0f32.into());
    let mut i: usize = 0;

    while i < maxiter {
        let z0 = (z.0 * z.0 - z.1 * z.1) + a;
        z.1 = Real::from(2f32) * z.0 * z.1 + b;
        z.0 = z0;
        i += 1;

        if z.0 * z.0 + z.1 * z.1 >= Real::from(4f32) {
            return (false, i);
        }
    }

    (true, i)
}


async fn compute_set_inner<Real: Arithmetic>(x_left: Real, y_bottom: Real, scale: Real, w: usize, h: usize, maxiter: usize, ct: CancellationToken) -> Option<(Vec<(bool, usize)>, Vec<usize>)> {
    let mut set = vec![(false, 0usize); w * h];
    let mut hist = vec![0usize; maxiter + 1];

    let mut tasks = Vec::<JoinHandle<(bool, usize)>>::with_capacity(w * h);

    for y in 0..h {
        for x in 0..w {
            if ct.is_cancelled() {
                return None;
            }

            let max = maxiter;
            tasks.push(tokio::spawn(async move {
                bounded((Real::from(x as f32) * scale + x_left, y_bottom + Real::from(y as f32) * scale), max)
            }));
        }
    }

    for (t, pixel_index) in tasks.iter_mut().zip(0..) {
        let result = t.await.ok()?;
        hist[result.1] += 1;
        set[pixel_index] = result;
    }

    Some((set, hist))
}

