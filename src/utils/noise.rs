use libnoise::{Generator, ImprovedPerlin};

pub trait PointGenerator<const N: usize>: Send + Sync {
    /// Sample the generator at the given point
    fn get(&self, point: [f64; N]) -> f64;
}

pub trait PointTransform<const N: usize>: Send + Sync {
    /// Transform a point into another point
    fn transform(&self, point: [f64; N]) -> [f64; N];
}

/// Add the outputs of two generators
/// y = G1(x) + G2(x)
struct Add<const N: usize, G1: PointGenerator<N>, G2: PointGenerator<N>> {
    g1: G1,
    g2: G2,
}

impl<const N: usize, G1: PointGenerator<N>, G2: PointGenerator<N>> PointGenerator<N>
    for Add<N, G1, G2>
{
    fn get(&self, point: [f64; N]) -> f64 {
        self.g1.get(point) + self.g2.get(point)
    }
}

/// Sum the outputs of many generators
/// y = G1(x) + G2(x) + ... + GN(x)
struct Sum<const N: usize, G: PointGenerator<N>>(
    // TODO: box?
    Vec<G>,
);
impl<const N: usize, G: PointGenerator<N>> PointGenerator<N> for Sum<N, G> {
    fn get(&self, point: [f64; N]) -> f64 {
        self.0.iter().map(|g| g.get(point)).sum()
    }
}

/// Multiply the output of two generators
/// y = G1(x) * G2(x)
struct Mul<const N: usize, G1: PointGenerator<N>, G2: PointGenerator<N>> {
    g1: G1,
    g2: G2,
}

impl<const N: usize, G1: PointGenerator<N>, G2: PointGenerator<N>> PointGenerator<N>
    for Mul<N, G1, G2>
{
    fn get(&self, point: [f64; N]) -> f64 {
        self.g1.get(point) * self.g2.get(point)
    }
}

/// Multiply the point by the output of the point passed through the generator
/// y = x * G(x)
struct MulSingle<const N: usize, G: PointGenerator<N>>(G);

impl<const N: usize, G: PointGenerator<N>> PointTransform<N> for MulSingle<N, G> {
    fn transform(&self, point: [f64; N]) -> [f64; N] {
        let mul = self.0.get(point);
        point.map(|x| x * mul)
    }
}

/// Apply a transform to a point before passing it to a generator
/// y = G(T(x))
struct PreTransform<const N: usize, T: PointTransform<N>, G: PointGenerator<N>> {
    t: T,
    g: G,
}

impl<const N: usize, G1: PointTransform<N>, G2: PointGenerator<N>> PointGenerator<N>
    for PreTransform<N, G1, G2>
{
    fn get(&self, point: [f64; N]) -> f64 {
        let point = self.t.transform(point);
        self.g.get(point)
    }
}

/// Apply a transform to a point after passing it to a generator
/// y = T(G(x))
struct PostTransform<const N: usize, T: PointTransform<1>, G: PointGenerator<N>> {
    t: T,
    g: G,
}
impl<const N: usize, T: PointTransform<1>, G: PointGenerator<N>> PointGenerator<N>
    for PostTransform<N, T, G>
{
    fn get(&self, point: [f64; N]) -> f64 {
        self.t.transform([self.g.get(point)])[0]
    }
}

// ==============================================
// Concrete implementations
// ==============================================

struct Perlin {
    source: ImprovedPerlin<2>,
}

impl Perlin {
    pub fn new(seed: u64) -> Self {
        Self {
            source: ImprovedPerlin::new(seed),
        }
    }
}

impl PointGenerator<2> for Perlin {
    fn get(&self, point: [f64; 2]) -> f64 {
        self.source.sample(point.map(|x| x.rem_euclid(256.)))
    }
}

/// Always produces a constant value
struct Constant(f64);

impl<const N: usize> PointGenerator<N> for Constant {
    fn get(&self, _point: [f64; N]) -> f64 {
        self.0
    }
}

/// Create a stack of perlin generators
pub fn perlin_stack(
    seed: u64,
    num_octaves: usize,
    amplitude: f64,
    persistence: f64,
    scale: f64,
) -> impl PointGenerator<2> {
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

    let layers = frequencies
        .into_iter()
        .zip(amplitudes)
        .map(|(f, a)| PostTransform {
            g: PreTransform {
                t: MulSingle(Constant(f)),
                g: Perlin::new(seed),
            },
            t: MulSingle(Constant(a)),
        })
        .collect();

    Sum(layers)
}
