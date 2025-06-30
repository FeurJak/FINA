use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
    prelude::StdRng,
};

pub use rand;

pub trait UniformRand: Sized {
    fn rand<R: Rng + ?Sized>(rng: &mut R) -> Self;
}

impl<T> UniformRand for T
where
    StandardUniform: Distribution<T>,
{
    #[inline]
    fn rand<R: Rng + ?Sized>(rng: &mut R) -> Self {
        rng.sample(StandardUniform)
    }
}

fn test_rng_helper() -> StdRng {
    use rand::SeedableRng;
    let seed = [
        1, 0, 0, 0, 23, 0, 0, 0, 200, 1, 0, 0, 210, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0,
    ];
    StdRng::from_seed(seed)
}

pub fn test_rng() -> impl Rng {
    test_rng_helper()
}
