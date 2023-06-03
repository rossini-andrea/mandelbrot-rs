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

pub fn mandelbrot_set<Real: Arithmetic>(x_left: Real, y_bottom: Real, scale: Real, w: u32, h: u32, ct: CancellationToken) -> Option<Vec<u8>> {
    let mut buf = vec![0u8; usize::try_from(w * h * 3).unwrap()];

    for y in 0..h {
        for x in 0..w {
            if ct.is_cancelled() {
                return None;
            }

            let pixel_index = usize::try_from((w * y + x) * 3).unwrap();
            (
                buf[pixel_index],
                buf[pixel_index + 1],
                buf[pixel_index + 2]
            ) = match bounded((Real::from(x) * scale + x_left, y_bottom + Real::from(y) * scale), 1000) {
                (true, ..) => (0, 0, 0),
                _ => (255, 255, 255)
            }
        }
    }

    Some(buf)
}
