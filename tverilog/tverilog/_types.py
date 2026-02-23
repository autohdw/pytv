from __future__ import annotations

import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

if sys.version_info >= (3, 10):
    @dataclass(frozen=True, slots=True)
    class GenerateResult:
        input_file: Path
        output_file: Path
        inst_file: Path
        generated_py_file: Path | None
        mode: Literal["inline", "subprocess"]
        python_path_used: str | None
        context_used: bool
else:
    @dataclass(frozen=True)
    class GenerateResult:
        input_file: Path
        output_file: Path
        inst_file: Path
        generated_py_file: Path | None
        mode: Literal["inline", "subprocess"]
        python_path_used: str | None
        context_used: bool
