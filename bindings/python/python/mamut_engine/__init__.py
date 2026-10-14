"""Scaffold local de MAMUT Engine: solo la versión del núcleo Rust."""

from ._native import version as version

__version__: str = version()
__all__ = ["version", "__version__"]
