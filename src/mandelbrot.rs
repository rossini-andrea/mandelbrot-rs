use tokio_util::sync::CancellationToken;

pub trait Arithmetic:
    std::ops::Mul<Output=Self> +
    std::ops::Div<Output=Self> +
    std::ops::Add<Output=Self> +
    std::ops::Sub<Output=Self> +
    std::cmp::PartialOrd<Self> +
    std::convert::From<f32> +
    std::convert::From<i32> +
    std::convert::From<u32> +
    std::marker::Copy
{

}

impl <T:
    std::ops::Mul<Output=Self> +
    std::ops::Div<Output=Self> +
    std::ops::Add<Output=Self> +
    std::ops::Sub<Output=Self> +
    std::cmp::PartialOrd<Self> +
    std::convert::From<f32> +
    std::convert::From<i32> +
    std::convert::From<u32> +
    std::marker::Copy> Arithmetic for T
{

}

pub fn bounded<Real: Arithmetic>((a, b): (Real, Real), maxiter: usize) -> (bool, usize) {
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


pub fn mandelbrot_set_inner<Real: Arithmetic>(x_left: Real, y_bottom: Real, scale: Real, w: usize, h: usize, maxiter: usize, ct: &CancellationToken) -> Option<(Vec<(bool, usize)>, Vec<usize>)> {
    let mut set = vec![(false, 0usize); usize::try_from(w * h).unwrap()];
    let mut hist = vec![0usize; maxiter + 1];

    for y in 0..h {
        for x in 0..w {
            if ct.is_cancelled() {
                return None;
            }

            let pixel_index = w * y + x;
            let result = bounded((Real::from(x as f32) * scale + x_left, y_bottom + Real::from(y as f32) * scale), maxiter);
            hist[result.1] += 1;
            set[pixel_index] = result;
        }
    }

    Some((set, hist))
}

pub fn mandelbrot_set<Real: Arithmetic>(x_left: Real, y_bottom: Real, scale: Real, w: usize, h: usize, maxiter: usize, palette: &Vec<(u8, u8, u8)>, ct: CancellationToken) -> Option<Vec<(u8, u8, u8)>> {
    let (set, hist) = match mandelbrot_set_inner(x_left, y_bottom, scale, w, h, maxiter, &ct) {
        Some(r) => r,
        None => { return None }
    };
    let mut color_remap = vec![0usize; maxiter + 1];
    let pixel_count = w * h;
    let buf: Vec<(u8, u8, u8)> = set.into_iter().map(|(b, i)| {
        let color_index = if b {
            pixel_count 
        } else if color_remap[i] == 0 {
            let c = hist.clone().into_iter().take(i).fold(0, |a, b| a + b);
            color_remap[i] = c;
            c
        } else {
            color_remap[i]
        } * (palette.len() - 1) / pixel_count;

        palette[color_index]
    }).collect();

    Some(buf)
}
