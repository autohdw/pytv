use crate::Config;
use crate::FileOptions;
use std::io::{Result, Write};
use std::path;

/// The `Convert` struct represents a converter that converts PYTV script to Python script to generate Verilog.
///
/// It is also possible to run the Python script after conversion and optionally delete it after running it.
/// It contains methods for converting code and managing input/output files.
#[derive(Debug)]
pub struct Convert {
    config: Config,
    file_options: FileOptions,
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
    fn open_input(&self) -> Result<String> {
        std::fs::read_to_string(&self.file_options.input)
    }

    /// Opens the output file and returns a file handle.
    /// Note: This will overwrite the existing file.
    fn open_output(&self) -> Result<std::fs::File> {
        std::fs::File::create(&self.output_file_name())
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
                output.push_str(".v.py");
            } else {
                output = output.replace(ext.to_str().unwrap(), "v.py");
            }
            output
        })
    }

    /// Checks if a line of code is a Python line based on the magic comment string in the configuration.
    fn if_py_line(&self, line: &str) -> bool {
        line.trim_start()
            .starts_with(&format!("//{}", self.config.magic_comment_str))
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

    /// Converts the code and writes the converted code to the given stream.
    pub fn convert<W: Write>(&self, mut stream: W) -> Result<()> {
        let mut first_py_line = false;
        let mut py_indent_space = 0usize;
        let magic_string_len = 2 + self.config.magic_comment_str.len();
        // parse line by line
        for line in self.open_input()?.lines() {
            let line = self.pre_process_line(&line);
            if self.if_py_line(&line) {
                let line = utf8_slice::from(line.trim_start(), magic_string_len);
                if !first_py_line && !line.is_empty() {
                    first_py_line = true;
                    py_indent_space = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
                }
                writeln!(stream, "{}", utf8_slice::from(&line, py_indent_space))?;
            } else {
                let line = self.apply_verilog_regex(self.escape_verilog(&line).as_str());
                writeln!(stream, "print(f'{line}')")?;
            }
        }
        Ok(())
    }

    /// Converts the code and writes the converted code to a file.
    pub fn convert_to_file(&self) -> Result<()> {
        let out_f = self.open_output()?;
        self.convert(out_f)
    }
}
