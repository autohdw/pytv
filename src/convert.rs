use crate::Config;
use crate::FileOptions;
use std::error::Error;
use std::io::{Result as IoResult, Write};
use std::path;
use std::result::Result;

/// Represents a converter that converts PyTV script to Python script to generate Verilog.
///
/// It is also possible to run the Python script after conversion and optionally delete it after running it.
/// It contains methods for converting code and managing input/output files.
#[derive(Debug, Default)]
pub struct Convert {
    config: Config,
    file_options: FileOptions,
}

#[derive(Debug, PartialEq)]
enum LineType {
    Verilog,
    PythonInline,
    PythonBlock(bool), // 'false' if in first line ('/*!'), 'true' otherwise
    None,
}

impl Default for LineType {
    fn default() -> Self {
        Self::None
    }
}

impl Convert {
    /// Creates a new `Convert` instance with the given configuration and file options.
    pub fn new(config: Config, file_options: FileOptions) -> Convert {
        Convert {
            config,
            file_options,
        }
    }

    /// Creates a new `Convert` instance by parsing command line arguments.
    pub fn from_args() -> Convert {
        let (config, file_options) = Config::from_args();
        Convert::new(config, file_options)
    }

    /// Opens the input file and reads its contents as a string.
    fn open_input(&self) -> IoResult<String> {
        std::fs::read_to_string(&self.file_options.input)
    }

    /// Opens the output Python file and returns a file handle.
    ///
    /// Note: This will overwrite the existing file.
    pub fn open_output(&self) -> IoResult<std::fs::File> {
        std::fs::File::create(&self.output_python_file_name())
    }

    /// Generates the output file name based on the input file name and configuration.
    fn output_file_name(&self) -> String {
        self.file_options.output.clone().unwrap_or_else(|| {
            if self.file_options.output.is_some() {
                return self.file_options.input.clone();
            }
            let mut output = self.file_options.input.clone();
            // change extension to .v if there is extension
            let ext = path::Path::new(&output).extension().unwrap_or_default();
            if ext.is_empty() {
                output.push_str(".v");
            } else {
                output = output.replace(ext.to_str().unwrap(), "v");
            }
            output
        })
    }

    /// Generates the output Python file name based on the input file name and configuration.
    fn output_python_file_name(&self) -> String {
        self.output_file_name() + ".py"
    }

    /// Generates the output instantiation file name based on the input file name and configuration.
    fn output_inst_file_name(&self) -> String {
        self.output_file_name() + ".inst"
    }

    fn switch_line_type(&self, line_type: &mut LineType, line: &str) {
        let trimmed_line = line.trim_start();
        *line_type = match line_type {
            LineType::PythonBlock(_not_first_line) => {
                if trimmed_line == "*/" {
                    LineType::None // end of PythonBlock does nothing
                } else {
                    LineType::PythonBlock(true)
                }
            }
            _ => {
                if trimmed_line.starts_with(&format!("/*{}", self.config.magic_comment_str)) {
                    LineType::PythonBlock(false)
                } else if trimmed_line.starts_with(&format!("//{}", self.config.magic_comment_str))
                {
                    LineType::PythonInline
                } else {
                    LineType::Verilog
                }
            }
        }
    }

    /// Pre-processes a line of code by trimming trailing whitespace and replacing tabs with spaces.
    fn pre_process_line(&self, line: &str) -> String {
        line.trim_end().replace(
            "\t",
            str::repeat(" ", self.config.tab_size as usize).as_str(),
        )
    }

    /// Escapes special characters in a line of Verilog code.
    fn escape_verilog(&self, line: &str) -> String {
        let mut escaped_line = String::with_capacity(line.len());
        for c in line.chars() {
            match c {
                '\'' => escaped_line.push_str("\\'"), // we use single quote for print
                '{' => escaped_line.push_str("{{"),
                '}' => escaped_line.push_str("}}"),
                _ => escaped_line.push(c),
            }
        }
        escaped_line
    }

    /// Applies a regular expression to a line of code based on the template regex in the configuration.
    fn apply_verilog_regex(&self, line: &str) -> String {
        self.config
            .template_re
            .replace_all(line, format!("{{$1}}").as_str())
            .to_string()
    }

    pub(crate) fn apply_protected_verilog_regex(&self, line: &str) -> String {
        self.config
            .template_re
            .replace_all(
                line,
                format!("__LEFT_BRACKET__{{$1}}__RIGHT_BRACKET__").as_str(),
            )
            .to_string()
    }

    /// Runs the Python code to generate verilog.
    ///
    /// The command `python3` should be available to call.
    pub fn run_python(&self) -> IoResult<()> {
        let py_file = self.output_python_file_name();
        let v_file = self.output_file_name();
        let v_file_f = std::fs::File::create(&v_file)?;
        let output = std::process::Command::new("python3")
            .arg(&py_file)
            .stdout(v_file_f)
            .output()?;
        dbg!(&output);
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Python script failed with exit code: {}\n{}",
                    output.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        } else {
            if self.config.delete_python {
                std::fs::remove_file(&py_file)?;
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "inst"))]
    fn process_python_line<W: Write>(
        &self,
        line: &str,
        py_indent_space: usize,
        stream: &mut W,
    ) -> Result<()> {
        writeln!(stream, "{}", utf8_slice::from(&line, py_indent_space))
    }

    /// Converts the code and writes the converted code to the given stream.
    pub fn convert<W: Write>(&self, mut stream: W) -> Result<(), Box<dyn Error>> {
        let mut first_py_line = false;
        let mut py_indent_space = 0usize;
        let magic_string_len = 2 + self.config.magic_comment_str.len();
        #[cfg(feature = "inst")]
        let mut within_inst = false;
        let mut inst_str = String::new();
        #[cfg(feature = "inst")]
        writeln!(
            stream,
            concat!(
                "_inst_file = open('{}', 'w')\n",
                "def _inst_var_map(tuples):\n",
                "    s = ['%s: %s\\n' % tuple for tuple in tuples]\n",
                "    return '    '.join(s)\n\n",
                "def _verilog_ports_var_map(tuples, first_port):\n",
                "    s = ['  .%s(%s)' % tuple for tuple in tuples]\n",
                "    return ('' if first_port else ',\\n') + ',\\n'.join(s)\n\n",
                "def _verilog_vparams_var_map(tuples, first_vparam):\n",
                "    s = ['\\n  parameter %s = %s' % tuple for tuple in tuples]\n",
                "    return ('#(' if first_vparam else ',') + ','.join(s)\n\n",
            ),
            self.output_inst_file_name()
        )?;
        let mut line_type = LineType::default();
        // parse line by line
        for line in self.open_input()?.lines() {
            let line = self.pre_process_line(&line);
            self.switch_line_type(&mut line_type, line.as_str());
            match line_type {
                LineType::PythonBlock(true) => {
                    #[cfg(feature = "inst")]
                    self.process_python_line(
                        &line,
                        0,
                        &mut stream,
                        &mut within_inst,
                        &mut inst_str,
                    )?;
                    #[cfg(not(feature = "inst"))]
                    self.process_python_line(&line, 0, &mut stream)?;
                }
                LineType::PythonInline => {
                    let line = utf8_slice::from(line.trim_start(), magic_string_len);
                    if !first_py_line && !line.is_empty() {
                        first_py_line = true;
                        py_indent_space =
                            line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
                    }
                    if !utf8_slice::till(&line, py_indent_space).trim().is_empty() {
                        Err(format!(
                            "Python line should start with {} spaces.\nUnexpected line: {}",
                            py_indent_space, &line
                        ))?;
                    }
                    #[cfg(feature = "inst")]
                    self.process_python_line(
                        &line,
                        py_indent_space,
                        &mut stream,
                        &mut within_inst,
                        &mut inst_str,
                    )?;
                    #[cfg(not(feature = "inst"))]
                    self.process_python_line(&line, py_indent_space, &mut stream)?;
                }
                LineType::Verilog => {
                    let line = self.apply_verilog_regex(self.escape_verilog(&line).as_str());
                    writeln!(stream, "print(f'{line}')")?;
                }
                _ => {}
            }
        }
        #[cfg(feature = "inst")]
        writeln!(stream, "_inst_file.close()")?;
        Ok(())
    }

    /// Converts the code and writes the converted code to a file.
    ///
    /// With default `Config`, the output will be a Python file.
    pub fn convert_to_file(&self) -> Result<(), Box<dyn Error>> {
        let out_f = self.open_output()?;
        self.convert(out_f)?;
        if self.config.run_python {
            self.run_python()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_process_line() {
        let convert = Convert::default();
        dbg!(convert.config.tab_size);
        assert_eq!(convert.pre_process_line("hello\they"), "hello    hey");
    }

    #[test]
    fn test_escape_verilog() {
        let convert = Convert::default();
        assert_eq!(convert.escape_verilog("hello'world"), "hello\\'world");
        assert_eq!(convert.escape_verilog("string {foo}"), "string {{foo}}");
        assert_eq!(
            convert.escape_verilog("string {{bar}}"),
            "string {{{{bar}}}}"
        );
        assert_eq!(convert.escape_verilog("\"em"), "\"em");
    }

    #[test]
    fn test_apply_verilog_regex() {
        let convert = Convert::default();
        assert_eq!(
            convert.apply_verilog_regex("hello `world`"),
            "hello {world}"
        );
        assert_eq!(
            convert.apply_verilog_regex("hello `world` `bar`"),
            "hello {world} {bar}"
        );
        assert_eq!(
            convert.apply_verilog_regex("`timescale 1ns / 1ps"),
            "`timescale 1ns / 1ps"
        );
    }

    #[test]
    fn test_switch_line_type() {
        let mut line_type = LineType::default();
        let convert = Convert::default();
        convert.switch_line_type(&mut line_type, "assign a = b;");
        assert_eq!(line_type, LineType::Verilog);
        convert.switch_line_type(&mut line_type, "//! num = 2 ** n;");
        assert_eq!(line_type, LineType::PythonInline);
        convert.switch_line_type(&mut line_type, "   //! num = num + 1;");
        assert_eq!(line_type, LineType::PythonInline);
        convert.switch_line_type(&mut line_type, "/*!");
        assert_eq!(line_type, LineType::PythonBlock(false));
        convert.switch_line_type(&mut line_type, "num = 2 ** n;");
        assert_eq!(line_type, LineType::PythonBlock(true));
        convert.switch_line_type(&mut line_type, "*/");
        assert_eq!(line_type, LineType::None);
        convert.switch_line_type(&mut line_type, "// Verilog comment");
        assert_eq!(line_type, LineType::Verilog);
    }
}
