from __future__ import annotations

import sys
from pathlib import Path

import pytest

from tverilog import (
    ConfigurationError,
    GeneratedExecutionError,
    PytvConversionError,
    generate,
)


def _write(path: Path, content: str) -> Path:
    path.write_text(content, encoding="utf-8")
    return path


def test_inline_happy_path_generation(tmp_path: Path) -> None:
    source = _write(tmp_path / "simple.pytv", "assign out = `value`;\n")
    output = tmp_path / "simple_out.v"

    result = generate(
        source,
        output_file=output,
        use_context=False,
        context={"value": 7},
    )

    assert result.mode == "inline"
    assert result.python_path_used is None
    assert result.context_used is False
    assert result.output_file == output
    assert output.read_text(encoding="utf-8").strip() == "assign out = 7;"


def test_inline_context_default_uses_caller_symbols(tmp_path: Path) -> None:
    source = _write(tmp_path / "ctx_default.pytv", "assign out = `value`;\n")

    def _run() -> str:
        value = 13  # noqa: F841
        result = generate(source, output_file=tmp_path / "ctx_default.v")
        return result.output_file.read_text(encoding="utf-8").strip()

    assert _run() == "assign out = 13;"


def test_use_context_false_disables_caller_scope(tmp_path: Path) -> None:
    source = _write(tmp_path / "ctx_off.pytv", "assign out = `value`;\n")

    def _run() -> None:
        value = 21  # noqa: F841
        generate(source, output_file=tmp_path / "ctx_off.v", use_context=False)

    with pytest.raises(GeneratedExecutionError):
        _run()


def test_explicit_context_overrides_collisions(tmp_path: Path) -> None:
    source = _write(tmp_path / "ctx_override.pytv", "assign out = `value`;\n")

    def _run() -> str:
        value = 3  # noqa: F841
        result = generate(
            source,
            output_file=tmp_path / "ctx_override.v",
            context={"value": 99},
        )
        return result.output_file.read_text(encoding="utf-8").strip()

    assert _run() == "assign out = 99;"


def test_subprocess_mode_works_with_explicit_python_path(tmp_path: Path) -> None:
    source = _write(tmp_path / "subprocess_ok.pytv", "assign out = 1;\n")
    output = tmp_path / "subprocess_ok.v"

    result = generate(source, output_file=output, python_path=sys.executable)

    assert result.mode == "subprocess"
    assert result.python_path_used == sys.executable
    assert result.context_used is False
    assert output.read_text(encoding="utf-8").strip() == "assign out = 1;"


def test_context_is_ignored_in_subprocess_mode(tmp_path: Path) -> None:
    source = _write(tmp_path / "subprocess_ctx_ignore.pytv", "assign out = `value`;\n")

    with pytest.raises(GeneratedExecutionError):
        generate(
            source,
            output_file=tmp_path / "subprocess_ctx_ignore.v",
            python_path=sys.executable,
            context={"value": 5},
            use_context=True,
        )


def test_conversion_failure_maps_to_typed_error(tmp_path: Path) -> None:
    missing = tmp_path / "missing_file.pytv"
    with pytest.raises(PytvConversionError):
        generate(missing)


def test_runtime_failure_maps_to_typed_error(tmp_path: Path) -> None:
    source = _write(tmp_path / "runtime_fail.pytv", "assign out = `missing_symbol`;\n")

    with pytest.raises(GeneratedExecutionError):
        generate(source, output_file=tmp_path / "runtime_fail.v", use_context=False)


def test_syntax_failure_maps_to_typed_error(tmp_path: Path) -> None:
    source = _write(
        tmp_path / "syntax_fail.pytv",
        "/*!\nif True:\nprint('bad indent')\n*/\n",
    )

    with pytest.raises(GeneratedExecutionError):
        generate(source, output_file=tmp_path / "syntax_fail.v", use_context=False)


def test_cleanup_success_removes_generated_python_by_default(tmp_path: Path) -> None:
    source = _write(tmp_path / "cleanup_default.pytv", "assign out = 1;\n")

    result = generate(source, output_file=tmp_path / "cleanup_default.v")

    assert result.generated_py_file is None
    assert not (tmp_path / "cleanup_default.v.py").exists()


def test_cleanup_success_keeps_generated_python_when_requested(tmp_path: Path) -> None:
    source = _write(tmp_path / "cleanup_keep.pytv", "assign out = 1;\n")

    result = generate(
        source,
        output_file=tmp_path / "cleanup_keep.v",
        keep_generated_py=True,
    )

    assert result.generated_py_file == tmp_path / "cleanup_keep.v.py"
    assert result.generated_py_file.exists()


def test_cleanup_failure_path_is_preserved_on_execution_failure(tmp_path: Path) -> None:
    source = _write(tmp_path / "cleanup_fail.pytv", "assign out = `missing_value`;\n")
    output = tmp_path / "cleanup_fail.v"

    with pytest.raises(GeneratedExecutionError) as exc_info:
        generate(source, output_file=output, use_context=False, keep_generated_py=False)

    script_path = output.with_suffix(output.suffix + ".py")
    assert script_path.exists()
    assert exc_info.value.script_path == script_path


def test_invalid_configuration_types_raise_configuration_error(tmp_path: Path) -> None:
    source = _write(tmp_path / "config.pytv", "assign out = 1;\n")
    with pytest.raises(ConfigurationError):
        generate(source, vars={"x": 1})  # type: ignore[arg-type]
