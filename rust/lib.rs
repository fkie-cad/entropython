use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// Compute the Shannon byte entropy of a byte array.
///
/// See https://en.wikipedia.org/wiki/Entropy_(information_theory)
/// for more information about Shannon entropy.
///
/// :param bytes: The input byte array.
/// :returns: The Shannon byte entropy of the input array.
#[pyfunction]
fn shannon_entropy(bytes: &[u8]) -> f64 {
    let byte_count = count_bytes(bytes);
    let mut entropy = 0f64;
    for counted_num in byte_count.iter().filter(|num| **num > 0u64) {
        let byte_probability = *counted_num as f64 / (bytes.len() as f64);
        entropy -= byte_probability * byte_probability.log2();
    }

    entropy
}

/// Compute the metric entropy of a byte array.
///
/// Metric entropy = Shannon entropy / 8
///
/// :param bytes: The input byte array.
/// :returns: The Shannon byte entropy of the input array.
#[pyfunction]
fn metric_entropy(bytes: &[u8]) -> f64 {
    shannon_entropy(bytes) / 8f64
}

/// Compute the chi-square statistic of the byte distribution of a byte array
/// against a uniform distribution.
///
/// Uniformly distributed (e.g. encrypted or random) data has a value close to 255
/// (the degrees of freedom), while compressed data with the same entropy is usually
/// considerably higher. The difference only becomes reliable with larger inputs.
///
/// See https://en.wikipedia.org/wiki/Pearson%27s_chi-squared_test
/// for more information about the chi-square test.
///
/// :param bytes: The input byte array.
/// :returns: The chi-square statistic of the input array (0.0 for an empty input).
#[pyfunction]
fn chi_square(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0f64;
    }
    let expected = bytes.len() as f64 / 256f64;
    count_bytes(bytes)
        .iter()
        .map(|count| {
            let difference = *count as f64 - expected;
            difference * difference / expected
        })
        .sum()
}

/// Compute the p-value of the chi-square test of a byte array against a uniform distribution.
///
/// The p-value is the probability that uniformly distributed data of the same length has a
/// chi-square statistic at least as high as the input's (255 degrees of freedom). A value close
/// to 0.0 means the bytes are not uniformly distributed (e.g. compressed data, given a large
/// enough input). For uniform (e.g. encrypted or random) data, the p-value is itself uniformly
/// distributed between 0.0 and 1.0, so a value close to 1.0 does not mean "more random".
///
/// The p-value uses the chi-square distribution, which is only a good approximation if each
/// byte value is expected at least 5 times. For shorter inputs (less than 1280 bytes), it would
/// overstate the non-uniformity (e.g. 10% instead of 1% of random 8-byte inputs would have a
/// p-value below 0.01), so the result is NaN instead.
///
/// :param bytes: The input byte array.
/// :returns: The p-value of the input array (NaN for inputs shorter than 1280 bytes).
#[pyfunction]
fn chi_square_p_value(bytes: &[u8]) -> f64 {
    if bytes.len() < CHI_SQUARE_P_VALUE_MIN_LENGTH {
        return f64::NAN;
    }
    upper_regularized_gamma(255f64 / 2f64, chi_square(bytes) / 2f64)
}

/// 5 expected occurrences per byte value
const CHI_SQUARE_P_VALUE_MIN_LENGTH: usize = 5 * 256;

const GAMMA_EPSILON: f64 = 1e-15;
const GAMMA_MAX_ITERATIONS: usize = 1000;

/// The regularized upper incomplete gamma function Q(a, x) = Γ(a, x) / Γ(a).
/// The chi-square survival function with k degrees of freedom is Q(k / 2, x / 2).
/// Uses a series for x < a + 1 and a continued fraction (modified Lentz) otherwise,
/// see Numerical Recipes, chapter 6.2.
fn upper_regularized_gamma(a: f64, x: f64) -> f64 {
    if x <= 0f64 {
        return 1f64;
    }
    let prefactor = (a * x.ln() - x - ln_gamma(a)).exp();
    if x < a + 1f64 {
        let mut term = 1f64 / a;
        let mut sum = term;
        let mut converged = false;
        for n in 1..GAMMA_MAX_ITERATIONS {
            term *= x / (a + n as f64);
            sum += term;
            if term.abs() < sum.abs() * GAMMA_EPSILON {
                converged = true;
                break;
            }
        }
        debug_assert!(converged, "gamma series did not converge for a = {a}, x = {x}");
        (1f64 - sum * prefactor).max(0f64)
    } else {
        let tiny = f64::MIN_POSITIVE / GAMMA_EPSILON;
        let mut b = x + 1f64 - a;
        let mut c = 1f64 / tiny;
        let mut d = 1f64 / b;
        let mut fraction = d;
        let mut converged = false;
        for i in 1..GAMMA_MAX_ITERATIONS {
            let an = -(i as f64) * (i as f64 - a);
            b += 2f64;
            d = an * d + b;
            if d.abs() < tiny {
                d = tiny;
            }
            c = b + an / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1f64 / d;
            let delta = d * c;
            fraction *= delta;
            if (delta - 1f64).abs() < GAMMA_EPSILON {
                converged = true;
                break;
            }
        }
        debug_assert!(converged, "gamma continued fraction did not converge for a = {a}, x = {x}");
        fraction * prefactor
    }
}

/// ln(Γ(x)) for x > 0 with the Lanczos approximation (g = 7, n = 9).
fn ln_gamma(x: f64) -> f64 {
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let x = x - 1f64;
    let t = x + 7.5f64;
    let series: f64 = COEFFICIENTS[0]
        + COEFFICIENTS[1..]
            .iter()
            .enumerate()
            .map(|(i, c)| c / (x + i as f64 + 1f64))
            .sum::<f64>();
    0.5f64 * (2f64 * std::f64::consts::PI).ln() + (x + 0.5f64) * t.ln() - t + series.ln()
}

fn count_bytes(bytes: &[u8]) -> [u64; 256] {
    let mut byte_count = [0u64; 256];
    for byte in bytes {
        byte_count[*byte as usize] += 1;
    }
    byte_count
}

/// This module implements functions to compute the Shannon byte entropy of a byte array
/// and the chi-square test of its byte distribution against a uniform distribution.
/// The computation itself is implemented in Rust for faster runtime.
#[pymodule(gil_used = false)]
fn libentropython(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(shannon_entropy, m)?)?;
    m.add_function(wrap_pyfunction!(metric_entropy, m)?)?;
    m.add_function(wrap_pyfunction!(chi_square, m)?)?;
    m.add_function(wrap_pyfunction!(chi_square_p_value, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shannon() {
        let mut bytes = [0u8; 1000];
        println!("shannon_entropy: {}", shannon_entropy(&bytes));
        assert_eq!(shannon_entropy(&bytes), 0f64);

        for i in 0..bytes.len() {
            bytes[i] = i as u8;
        }
        println!("shannon_entropy: {}", shannon_entropy(&bytes));
        assert!(shannon_entropy(&bytes) > 7.99f64);
        assert!(shannon_entropy(&bytes) <= 8f64);
    }

    #[test]
    fn chi_square_uniform_and_constant() {
        assert_eq!(chi_square(&[]), 0f64);

        // every byte value exactly equally often: perfectly uniform
        let uniform: Vec<u8> = (0..1024).map(|i| i as u8).collect();
        assert_eq!(chi_square(&uniform), 0f64);

        // a single byte value: (n - n/256)² / (n/256) + 255 * n/256 = 255 * n
        let constant = [0u8; 1000];
        assert!((chi_square(&constant) - 255_000f64).abs() < 1e-6);
    }

    #[test]
    fn ln_gamma_known_values() {
        // Γ(n) = (n - 1)!, Γ(0.5) = √π
        assert!((ln_gamma(1f64)).abs() < 1e-12);
        assert!((ln_gamma(5f64) - 24f64.ln()).abs() < 1e-12);
        assert!((ln_gamma(0.5f64) - std::f64::consts::PI.sqrt().ln()).abs() < 1e-12);
    }

    #[test]
    fn chi_square_survival_function() {
        // reference values: scipy.stats.chi2.sf(x, 255)
        let cases = [
            (0f64, 1f64),
            (255f64, 0.48822252177040637),
            (310.45738821990585, 0.01),
            (330.51974363400586, 0.001),
            (200f64, 0.9954254445419519),
        ];
        for (x, expected) in cases {
            let p_value = upper_regularized_gamma(127.5f64, x / 2f64);
            assert!((p_value - expected).abs() < 1e-10, "x = {x}: {p_value} != {expected}");
        }
        assert!(upper_regularized_gamma(127.5f64, 1e6f64) < 1e-300);
    }

    #[test]
    fn chi_square_p_value_bounds() {
        let uniform: Vec<u8> = (0..1280).map(|i| i as u8).collect();
        assert_eq!(chi_square_p_value(&uniform), 1f64);
        assert_eq!(chi_square_p_value(&[0u8; 1280]), 0f64);
    }

    #[test]
    fn chi_square_p_value_too_short() {
        assert!(chi_square_p_value(&[]).is_nan());
        assert!(chi_square_p_value(&[0u8; 8]).is_nan());
        let uniform: Vec<u8> = (0..1279).map(|i| i as u8).collect();
        assert!(chi_square_p_value(&uniform).is_nan());
    }
}