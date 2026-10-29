# entropython

A Python module for efficient calculation of Shannon byte entropy (and the chi-square statistic of the byte
distribution) based on Rust.

## Installation

To install the version from pypi.org run
```shell
pip install entropython
```

To install the latest version directly from GitHub run
```shell
pip git+https://github.com/fkie-cad/entropython
```
(you need to [have rust installed](https://www.rust-lang.org/tools/install))
or check out the repository and run
```shell
pip install .
``` 

## Usage

```python
from entropython import shannon_entropy, metric_entropy, chi_square

byte_str = (
    b'Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod ' 
    b'tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.'
)
print(shannon_entropy(byte_str))
print(metric_entropy(byte_str))
print(chi_square(byte_str))
```

`chi_square` tests the byte distribution against a uniform distribution (255 degrees of freedom).
Encrypted or random data stays close to 255, while compressed data, which has a similarly high entropy,
is usually considerably higher (the difference is only reliable for larger inputs, e.g. from a few hundred KiB).
