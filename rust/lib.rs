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

fn count_bytes(bytes: &[u8]) -> [u64; 256] {
    let mut byte_count = [0u64; 256];
    for byte in bytes {
        byte_count[*byte as usize] += 1;
    }
    byte_count
}

/// This module implements functions to compute the Shannon byte entropy of a byte array.
/// The computation itself is implemented in Rust for faster runtime.
#[pymodule(gil_used = false)]
fn libentropython(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(shannon_entropy, m)?)?;
    m.add_function(wrap_pyfunction!(metric_entropy, m)?)?;
    m.add_function(wrap_pyfunction!(chi_square, m)?)?;
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
}