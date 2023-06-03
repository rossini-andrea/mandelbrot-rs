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


pub fn mandelbrot_set_inner<Real: Arithmetic>(x_left: Real, y_bottom: Real, scale: Real, w: u32, h: u32, maxiter: usize, ct: &CancellationToken) -> Option<(Vec<(bool, usize)>, Vec<usize>)> {
    let mut set = vec![(false, 0usize); usize::try_from(w * h).unwrap()];
    let mut hist = vec![0usize; maxiter + 1];

    for y in 0..h {
        for x in 0..w {
            if ct.is_cancelled() {
                return None;
            }

            let pixel_index = usize::try_from(w * y + x).unwrap();
            let result = bounded((Real::from(x) * scale + x_left, y_bottom + Real::from(y) * scale), maxiter);
            hist[result.1] += 1;
            set[pixel_index] = result;
        }
    }

    Some((set, hist))
}

pub fn mandelbrot_set<Real: Arithmetic>(x_left: Real, y_bottom: Real, scale: Real, w: u32, h: u32, maxiter: usize, ct: CancellationToken) -> Option<Vec<(u8, u8, u8)>> {
    let set = match mandelbrot_set_inner(x_left, y_bottom, scale, w, h, maxiter, &ct) {
        Some((set, _)) => set,
        None => { return None }
    };
    let mut buf = vec![(0u8, 0u8, 0u8); usize::try_from(w * h).unwrap()];

    for ((bounded, _), color) in set.iter().zip(buf.iter_mut()) {
        if ct.is_cancelled() {
            return None;
        }

        *color = if *bounded {
            (0, 0, 0)
        } else {
            (255, 255, 255)
        };
    
    }

    Some(buf)
}
