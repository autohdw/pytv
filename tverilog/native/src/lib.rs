use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use pytv::{Config, Convert, FileOptions};
use std::path::{Path, PathBuf};

#[pyclass(module = "tverilog._core", frozen)]
#[derive(Debug, Clone)]
struct ConversionArtifacts {
    #[pyo3(get)]
    script: String,
    #[pyo3(get)]
    verilog_file: String,
    #[pyo3(get)]
    python_script_file: String,
    #[pyo3(get)]
    inst_file: String,
}

#[pyfunction]
#[pyo3(signature = (
    input_file,
    output_file=None,
    magic="!",
    tab_size=4,
    vars=None,
    preamble=None
))]
fn convert_template(
    input_file: String,
    output_file: Option<String>,
    magic: &str,
    tab_size: u32,
    vars: Option<Vec<(String, String)>>,
    preamble: Option<String>,
) -> PyResult<ConversionArtifacts> {
    let config = Config::new(
        magic.to_string(),
        Config::default_template_re(),
        false,
        false,
        tab_size,
    );

    let file_options = FileOptions {
        input: input_file.clone(),
        output: output_file.clone(),
    };

    let convert = Convert::new(config, file_options, vars, preamble);
    let script = convert
        .render_python_script()
        .map_err(|err| PyRuntimeError::new_err(format!("conversion failed: {err}")))?;

    let input_path = PathBuf::from(input_file);
    let output_path = output_file.as_ref().map(PathBuf::from);
    let output_paths = Convert::output_paths(Path::new(&input_path), output_path.as_deref());

    Ok(ConversionArtifacts {
        script,
        verilog_file: output_paths.verilog_file.to_string_lossy().to_string(),
        python_script_file: output_paths
            .python_script_file
            .to_string_lossy()
            .to_string(),
        inst_file: output_paths.inst_file.to_string_lossy().to_string(),
    })
}

#[pymodule]
fn _core(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ConversionArtifacts>()?;
    module.add_function(wrap_pyfunction!(convert_template, module)?)?;
    Ok(())
}
