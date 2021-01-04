# entropython

Efficient calculation of Shannon byte entropy based on Rust.

## Installation

For building the binary, [Rust](https://www.rust-lang.org/) needs to be installed.

Run
```shell
cargo build --release
mv target/release/libentropython.so entropython.so # The renaming is necessary for Python to find the module
# Optional: Remove debug symbols from the binary to dramatically reduce its size.
strip entropython.so
```

The built `entropython.so` itself has no dependencies.

## Usage

```python
from entropython import shannon_entropy, metric_entropy

bytes = 'Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.'.encode('utf-8')
print(shannon_entropy(bytes))
print(metric_entropy(bytes))
```