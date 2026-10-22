"""Estadísticos de las medidas. Hyndman y Fan tipo 6, sin ajuste manual."""

import math


def quantile(samples, probability):
    """Cuantil tipo 6: h = (n + 1) p, con interpolación lineal entre estadísticos de orden."""
    if not samples:
        raise ValueError("no hay muestras")
    if not 0.0 <= probability <= 1.0:
        raise ValueError("la probabilidad está fuera de [0, 1]")
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
    """Mediana, rango intercuartil, mínimo y máximo de una muestra no vacía."""
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
