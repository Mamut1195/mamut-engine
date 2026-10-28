"""Computes measurement statistics using Hyndman and Fan type 6, without manual adjustment."""

import math


def quantile(samples, probability):
    """Computes a type 6 quantile: h = (n + 1) p, with linear interpolation between order statistics."""
    if not samples:
        raise ValueError("there are no samples")
    if not 0.0 <= probability <= 1.0:
        raise ValueError("the probability is outside [0, 1]")
    ordered = sorted(samples)
    count = len(ordered)
    position = (count + 1) * probability
    if position <= 1:
        return ordered[0]
    if position >= count:
        return ordered[-1]
    lower = math.floor(position)
    fraction = position - lower
    left = ordered[lower - 1]
    right = ordered[lower]
    return (1.0 - fraction) * left + fraction * right


def summarize(samples):
    """Computes the median, interquartile range, minimum, and maximum of a nonempty sample."""
    ordered = sorted(samples)
    q1 = quantile(ordered, 0.25)
    q3 = quantile(ordered, 0.75)
    return {
        "n": len(ordered),
        "median": quantile(ordered, 0.5),
        "iqr": q3 - q1,
        "min": ordered[0],
        "max": ordered[-1],
    }
