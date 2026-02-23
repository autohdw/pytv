"""Internal bridge module that re-exports native functions from the installed extension."""

from __future__ import annotations

from _core import ConversionArtifacts, convert_template

__all__ = ["ConversionArtifacts", "convert_template"]
