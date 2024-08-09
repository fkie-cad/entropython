import math
import os

import pytest
from entropython import shannon_entropy, metric_entropy


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
