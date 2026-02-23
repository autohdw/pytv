from __future__ import annotations

import builtins
import subprocess
from contextlib import redirect_stdout
from pathlib import Path
from typing import Any, Mapping

from ._errors import ConfigurationError, GeneratedExecutionError


def _line_from_traceback(exc: BaseException, script_path: Path) -> int | None:
    tb = exc.__traceback__
    script_str = str(script_path)
    while tb is not None:
        if tb.tb_frame.f_code.co_filename == script_str:
            return tb.tb_lineno
        tb = tb.tb_next
    return None


def run_inline(
    *,
    script_text: str,
    script_path: Path,
    output_file: Path,
    use_context: bool,
    caller_globals: Mapping[str, Any] | None,
    caller_locals: Mapping[str, Any] | None,
    explicit_context: Mapping[str, Any] | None,
) -> None:
    try:
        code_obj = compile(script_text, str(script_path), "exec")
    except SyntaxError as exc:
        detail = f"Generated Python has syntax error: {exc.msg}"
        if exc.lineno is not None:
            detail += f" (line {exc.lineno})"
        raise GeneratedExecutionError(
            detail,
            script_path=script_path,
            line=exc.lineno,
            mode="inline",
        ) from exc

    exec_namespace: dict[str, Any] = {
        "__builtins__": builtins.__dict__,
        "__name__": "__main__",
        "__file__": str(script_path),
    }
    if use_context:
        if caller_globals:
            exec_namespace.update(caller_globals)
        if caller_locals:
            exec_namespace.update(caller_locals)
    if explicit_context:
        exec_namespace.update(dict(explicit_context))

    try:
        with output_file.open("w", encoding="utf-8", newline="\n") as stream:
            with redirect_stdout(stream):
                exec(code_obj, exec_namespace, exec_namespace)
    except Exception as exc:  # pragma: no cover - narrow except is not practical here
        line = _line_from_traceback(exc, script_path)
        detail = f"Generated Python execution failed: {exc}"
        if line is not None:
            detail += f" (line {line})"
        raise GeneratedExecutionError(
            detail,
            script_path=script_path,
            line=line,
            mode="inline",
        ) from exc


def run_subprocess(*, script_path: Path, output_file: Path, python_path: str) -> None:
    try:
        with output_file.open("w", encoding="utf-8", newline="\n") as stream:
            completed = subprocess.run(
                [python_path, str(script_path)],
                stdout=stream,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )
    except FileNotFoundError as exc:
        raise ConfigurationError(f"Python interpreter not found: {python_path}") from exc
    except OSError as exc:
        raise GeneratedExecutionError(
            f"Failed to execute generated script in subprocess: {exc}",
            script_path=script_path,
            mode="subprocess",
            python_path=python_path,
        ) from exc

    if completed.returncode != 0:
        detail = f"Generated Python subprocess failed with exit code {completed.returncode}"
        stderr = (completed.stderr or "").strip()
        if stderr:
            detail += f": {stderr}"
        raise GeneratedExecutionError(
            detail,
            script_path=script_path,
            mode="subprocess",
            python_path=python_path,
        )
