# entropython

A Python module for efficient calculation of Shannon byte entropy based on Rust.

## Installation

Just run
```shell
pip install entropython
```

## Usage

```python
from entropython import shannon_entropy, metric_entropy

bytes = 'Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.'.encode('utf-8')
print(shannon_entropy(bytes))
print(metric_entropy(bytes))
```

## Build from Source

For building the binary from source, [Rust](https://www.rust-lang.org/) needs to be installed.

Run
```shell
cargo build --release
mv target/release/libentropython.so entropython.so # The renaming is necessary for Python to find the module
# Optional: Remove debug symbols from the binary to dramatically reduce its size.
strip entropython.so
```

The built `entropython.so` itself has no dependencies.