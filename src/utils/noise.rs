use libnoise::{Generator, ImprovedPerlin};

// ==============================================
// Libnoise extensions
// ==============================================

/// Libnoise Generator that's thread safe
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

/// Applies the modulo operator to the inputs
struct ModInput<const D: usize, G>
where
    G: Generator<D>,
{
    generator: G,
    modulo: [f64; D],
}

impl<const D: usize, G> Generator<D> for ModInput<D, G>
where
    G: Generator<D>,
{
    fn sample(&self, point: [f64; D]) -> f64 {
        let mut transformed_point = [0.; D];
        for (i, (x, m)) in point.into_iter().zip(self.modulo).enumerate() {
            transformed_point[i] = x.rem_euclid(m);
        }
        self.generator.sample(transformed_point)
    }
}

/// Extension trait with my adapters
trait GenAdapters<const D: usize>: Generator<D> + Sized {
    fn mod_input(self, modulo: [f64; D]) -> ModInput<D, Self> {
        ModInput {
            generator: self,
            modulo,
        }
    }
}
impl<const D: usize, G> GenAdapters<D> for G where G: Generator<D> + Sized {}

/// Create a stack of perlin generators
///
/// num_octaves: Number of noise layers to use
/// amplitude: Influence multiplier for each subsequent noise layer. Should usually be 0-1
/// persistence: Granularity multiplier for each subsequent noise layer. Should usually be 0-1
/// scale: Overall granularity multiplier, Bigger == more granular
///
pub fn perlin_stack(
    seed: u64,
    num_octaves: usize,
    amplitude: f64,
    persistence: f64,
    scale: f64,
    offset: f64,
) -> impl MyGenerator<2> {
    assert!(num_octaves > 0);
    assert!(amplitude > 0.);
    assert!(persistence > 0.);

    // Pre-generate octave values
    let mut amplitudes = vec![1.];
    let mut frequencies = vec![1. * scale];
    let mut offsets = vec![0.];
    for i in 0..num_octaves - 1 {
        amplitudes.push(amplitudes[i] * amplitude);
        frequencies.push(frequencies[i] * persistence);
        offsets.push(offsets[i] + offset);
    }

    // To preserve variance, amplitudes must satisty unit circle constraint
    let divisor = amplitudes.iter().map(|a| a * a).sum::<f64>().sqrt();
    amplitudes.iter_mut().for_each(|a| *a /= divisor);

    frequencies
        .into_iter()
        .zip(amplitudes)
        .zip(offsets)
        .map(|((f, a), o)| {
            ImprovedPerlin::<2>::new(seed)
                .mod_input([256.; _])
                .scale([f; _])
                .translate([o; _])
                .mul(a)
        })
        .collect::<SumMany<_, _>>()
        .clamp(-1., 1.)
}
