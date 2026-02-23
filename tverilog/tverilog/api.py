from __future__ import annotations

import importlib
import inspect
import os
from pathlib import Path
from typing import Any, Mapping

from ._errors import (
    ConfigurationError,
    GeneratedExecutionError,
    PytvConversionError,
)
from ._runner import run_inline, run_subprocess
from ._types import GenerateResult


def _load_core() -> Any:
    try:
        return importlib.import_module("tverilog._core")
    except ImportError as exc:
        raise ConfigurationError(
            "tverilog._core is not available. Build/install the native extension first."
        ) from exc


def _normalize_vars(vars_map: Mapping[str, str] | None) -> list[tuple[str, str]] | None:
    if vars_map is None:
        return None
    if not isinstance(vars_map, Mapping):
        raise ConfigurationError("'vars' must be a mapping from str to str.")
    pairs: list[tuple[str, str]] = []
    for key, value in vars_map.items():
        if not isinstance(key, str):
            raise ConfigurationError("'vars' keys must be str.")
        if not isinstance(value, str):
            raise ConfigurationError(
                "'vars' values must be str Python expressions (for example: \"2 + 3\")."
            )
        pairs.append((key, value))
    return pairs


def _normalize_context(context: Mapping[str, Any] | None) -> dict[str, Any] | None:
    if context is None:
        return None
    if not isinstance(context, Mapping):
        raise ConfigurationError("'context' must be a mapping.")
    return dict(context)


def _ensure_parent(path: Path) -> None:
    if path.parent != Path("") and str(path.parent) != ".":
        path.parent.mkdir(parents=True, exist_ok=True)


def generate(
    input_file: str | os.PathLike[str],
    *,
    output_file: str | os.PathLike[str] | None = None,
    magic: str = "!",
    tab_size: int = 4,
    vars: Mapping[str, str] | None = None,
    preamble: str | os.PathLike[str] | None = None,
    python_path: str | os.PathLike[str] | None = None,
    use_context: bool = True,
    context: Mapping[str, Any] | None = None,
    keep_generated_py: bool = False,
) -> GenerateResult:
    """Generate Verilog from a PyTV template using the native conversion bridge."""
    if not isinstance(magic, str):
        raise ConfigurationError("'magic' must be a string.")
    if not isinstance(tab_size, int) or tab_size <= 0:
        raise ConfigurationError("'tab_size' must be a positive integer.")
    if not isinstance(use_context, bool):
        raise ConfigurationError("'use_context' must be bool.")
    if not isinstance(keep_generated_py, bool):
        raise ConfigurationError("'keep_generated_py' must be bool.")

    input_path = Path(os.fspath(input_file))
    if str(input_path) == "":
        raise ConfigurationError("'input_file' cannot be empty.")

    output_path = None if output_file is None else Path(os.fspath(output_file))
    preamble_path = None if preamble is None else Path(os.fspath(preamble))

    python_path_used: str | None
    if python_path is None:
        python_path_used = None
    else:
        python_path_used = os.fspath(python_path)
        if python_path_used.strip() == "":
            raise ConfigurationError("'python_path' cannot be empty when provided.")

    normalized_vars = _normalize_vars(vars)
    explicit_context = _normalize_context(context)

    core = _load_core()
    try:
        converted = core.convert_template(
            str(input_path),
            None if output_path is None else str(output_path),
            magic,
            tab_size,
            normalized_vars,
            None if preamble_path is None else str(preamble_path),
        )
    except Exception as exc:
        raise PytvConversionError(
            f"PyTV conversion failed for '{input_path}': {exc}"
        ) from exc

    output_verilog_path = Path(converted.verilog_file)
    inst_path = Path(converted.inst_file)
    generated_script_path = Path(converted.python_script_file)
    script_text = str(converted.script)

    _ensure_parent(output_verilog_path)
    _ensure_parent(inst_path)
    _ensure_parent(generated_script_path)

    try:
        generated_script_path.write_text(script_text, encoding="utf-8")
    except OSError as exc:
        raise GeneratedExecutionError(
            f"Failed to write generated Python script: {exc}",
            script_path=generated_script_path,
            mode="inline" if python_path_used is None else "subprocess",
            python_path=python_path_used,
        ) from exc

    mode = "inline" if python_path_used is None else "subprocess"
    caller_globals: dict[str, Any] | None = None
    caller_locals: dict[str, Any] | None = None

    if mode == "inline" and use_context:
        frame = inspect.currentframe()
        try:
            caller = frame.f_back if frame is not None else None
            if caller is not None:
                caller_globals = dict(caller.f_globals)
                caller_locals = dict(caller.f_locals)
        finally:
            del frame

    if mode == "inline":
        run_inline(
            script_text=script_text,
            script_path=generated_script_path,
            output_file=output_verilog_path,
            use_context=use_context,
            caller_globals=caller_globals,
            caller_locals=caller_locals,
            explicit_context=explicit_context,
        )
    else:
        run_subprocess(
            script_path=generated_script_path,
            output_file=output_verilog_path,
            python_path=python_path_used,
        )

    generated_py_file: Path | None = generated_script_path
    if not keep_generated_py:
        try:
            generated_script_path.unlink(missing_ok=True)
            generated_py_file = None
        except OSError as exc:
            raise GeneratedExecutionError(
                f"Generated script cleanup failed: {exc}",
                script_path=generated_script_path,
                mode=mode,
                python_path=python_path_used,
            ) from exc

    return GenerateResult(
        input_file=input_path,
        output_file=output_verilog_path,
        inst_file=inst_path,
        generated_py_file=generated_py_file,
        mode=mode,
        python_path_used=python_path_used,
        context_used=(mode == "inline" and use_context),
    )
