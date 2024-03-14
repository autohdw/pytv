use crate::Config;
use crate::FileOptions;
use std::error::Error;
use std::io::Write;
use std::path;

#[derive(Debug)]
pub struct Convert {
    config: Config,
    file_options: FileOptions,
}

impl Convert {
    pub fn new(config: Config, file_options: FileOptions) -> Convert {
        Convert {
            config,
            file_options,
        }
    }

    pub fn from_args() -> Convert {
        let (config, file_options) = Config::from_args();
        Convert::new(config, file_options)
    }

    fn open_input(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.file_options.input)
    }

    fn open_output(&self) -> Result<std::fs::File, std::io::Error> {
        std::fs::File::create(&self.output_file_name()) // note this will overwrite
    }

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

    fn if_py_line(&self, line: &str) -> bool {
        line.trim_start()
            .starts_with(&format!("//{}", self.config.magic_comment_str))
    }

    fn pre_process_line(&self, line: &str) -> String {
        line.trim_end().replace(
            "\t",
            str::repeat(" ", self.config.tab_size as usize).as_str(),
        )
    }

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

    fn apply_verilog_regex(&self, line: &str) -> String {
        // self.config.template_re
        // regex replace
        self.config
            .template_re
            .replace_all(line, format!("{{$1}}").as_str())
            .to_string()
    }

    pub fn convert(&self) -> Result<(), Box<dyn Error>> {
        let mut out_f = self.open_output()?;
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
                writeln!(out_f, "{}", utf8_slice::from(&line, py_indent_space))?;
            } else {
                let line = self.apply_verilog_regex(self.escape_verilog(&line).as_str());
                writeln!(out_f, "print(f'{line}')")?;
            }
        }
        Ok(())
    }
}
