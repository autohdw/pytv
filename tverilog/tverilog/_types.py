from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Literal


@dataclass(frozen=True, slots=True)
class GenerateResult:
    input_file: Path
    output_file: Path
    inst_file: Path
    generated_py_file: Path | None
    mode: Literal["inline", "subprocess"]
    python_path_used: str | None
    context_used: bool
