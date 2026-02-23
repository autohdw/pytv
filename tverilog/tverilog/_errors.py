from __future__ import annotations

from pathlib import Path
from typing import Literal


class TVerilogError(RuntimeError):
    """Base exception for all tverilog errors."""


class PytvConversionError(TVerilogError):
    """Raised when PyTV template conversion fails."""


class ConfigurationError(TVerilogError):
    """Raised when API arguments are invalid."""


class GeneratedExecutionError(TVerilogError):
    """Raised when executing generated Python fails."""

    def __init__(
        self,
        message: str,
        *,
        script_path: Path | None = None,
        line: int | None = None,
        mode: Literal["inline", "subprocess"] | None = None,
        python_path: str | None = None,
    ) -> None:
        super().__init__(message)
        self.script_path = script_path
        self.line = line
        self.mode = mode
        self.python_path = python_path
