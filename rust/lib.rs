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
    let mut byte_count = [0u64; 256];
    for byte in bytes {
        byte_count[*byte as usize] += 1;
    }
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

/// This module implements functions to compute the Shannon byte entropy of a byte array.
/// The computation itself is implemented in Rust for faster runtime.
#[pymodule]
fn libentropython(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(shannon_entropy, m)?)?;
    m.add_function(wrap_pyfunction!(metric_entropy, m)?)?;
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
}