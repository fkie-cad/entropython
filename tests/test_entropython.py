import math
import os
import zlib

import pytest
from entropython import shannon_entropy, metric_entropy, chi_square, chi_square_p_value


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


@pytest.fixture(scope='module')
def compressed():
    return zlib.compress(b"".join(b"%d: %d\n" % (i, i * i) for i in range(200_000)), 9)


def test_chi_square_random_vs_compressed(compressed):
    # 255 degrees of freedom: uniform data is below ~330 with p = 0.999
    assert chi_square(os.urandom(1_000_000)) < 400
    assert chi_square(compressed) > 400


@pytest.mark.parametrize(('input_value', 'expected_output'), [
    (bytes(range(256)) * 5, 1.0),
    (b"0" * 1280, 0.0),
])
def test_chi_square_p_value(input_value, expected_output):
    assert math.isclose(chi_square_p_value(input_value), expected_output, abs_tol=1e-9)


@pytest.mark.parametrize('input_value', [b"", b"00000000", os.urandom(1279)])
def test_chi_square_p_value_too_short(input_value):
    assert math.isnan(chi_square_p_value(input_value))


def test_chi_square_p_value_random_vs_compressed(compressed):
    # a p-value below 1e-9 for random data happens once in a billion runs
    assert chi_square_p_value(os.urandom(1_000_000)) > 1e-9
    assert chi_square_p_value(compressed) < 1e-9
