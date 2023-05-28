pub trait Arithmetic:
    std::ops::Mul<Output=Self> +
    std::ops::Div<Output=Self> +
    std::ops::Add<Output=Self> +
    std::ops::Sub<Output=Self> +
    std::cmp::Eq +
    std::cmp::PartialOrd +
    std::convert::From<f32> +
    std::marker::Copy
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

