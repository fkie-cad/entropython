import math
import os
import zlib

import pytest
from entropython import shannon_entropy, metric_entropy, chi_square


@pytest.mark.parametrize(('input_value', 'expected_output'), [
    (b"00000000", 0.0),
    (os.urandom(10_000), 8.0),
])
def test_shannon_entropy(input_value, expected_output):
    assert math.isclose(
        shannon_entropy(input_value),
        expected_output,
        abs_tol=0.05,
    )


@pytest.mark.parametrize(('input_value', 'expected_output'), [
    (b"00000000", 0.0),
    (os.urandom(10_000), 1.0),
])
def test_metric_entropy(input_value, expected_output):
    assert math.isclose(
        metric_entropy(input_value),
        expected_output,
        abs_tol=0.05,
    )


@pytest.mark.parametrize(('input_value', 'expected_output'), [
    (b"", 0.0),
    (bytes(range(256)) * 4, 0.0),
    (b"00000000", 255 * 8),
])
def test_chi_square(input_value, expected_output):
    assert math.isclose(chi_square(input_value), expected_output, abs_tol=1e-6)


def test_chi_square_random_vs_compressed():
    # 255 degrees of freedom: uniform data is below ~330 with p = 0.999
    assert chi_square(os.urandom(1_000_000)) < 400
    compressed = zlib.compress(b"".join(b"%d: %d\n" % (i, i * i) for i in range(200_000)), 9)
    assert chi_square(compressed) > 400
