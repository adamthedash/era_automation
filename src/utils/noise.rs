use libnoise::{Generator, ImprovedPerlin};

// ==============================================
// Libnoise extensions
// ==============================================

pub trait MyGenerator<const D: usize>: Generator<D> + Send + Sync {}

impl<const D: usize, G> MyGenerator<D> for G where G: Generator<D> + Send + Sync {}

/// Sum the outputs of many generators
struct SumMany<const D: usize, G>(Vec<G>)
where
    G: Generator<D>;

impl<const D: usize, G> Generator<D> for SumMany<D, G>
where
    G: Generator<D>,
{
    fn sample(&self, point: [f64; D]) -> f64 {
        self.0.iter().map(|g| g.sample(point)).sum()
    }
}

impl<const D: usize, G> FromIterator<G> for SumMany<D, G>
where
    G: Generator<D>,
{
    fn from_iter<T: IntoIterator<Item = G>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Create a stack of perlin generators
pub fn perlin_stack(
    seed: u64,
    num_octaves: usize,
    amplitude: f64,
    persistence: f64,
    scale: f64,
) -> impl MyGenerator<2> {
    assert!(num_octaves > 0);
    assert!(amplitude > 0.);
    assert!(persistence > 0.);

    // Pre-generate octave values
    let mut amplitudes = vec![1.];
    let mut frequencies = vec![1. * scale];
    for i in 0..num_octaves - 1 {
        amplitudes.push(amplitudes[i] * amplitude);
        frequencies.push(frequencies[i] * persistence);
    }

    // To preserve variance, amplitudes must satisty unit circle constraint
    let divisor = amplitudes.iter().map(|a| a * a).sum::<f64>().sqrt();
    amplitudes.iter_mut().for_each(|a| *a /= divisor);

    frequencies
        .into_iter()
        .zip(amplitudes)
        .map(|(f, a)| ImprovedPerlin::<2>::new(seed).scale([f; 2]).mul(a))
        .collect::<SumMany<_, _>>()
        .clamp(-1., 1.)
}
