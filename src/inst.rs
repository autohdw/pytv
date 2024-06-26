use super::Convert;
use regex::{self, Regex};
use std::error::Error;
use std::io::Write;

/// Represents the state of module instantiation.
enum InstState {
    None,
    Begin,
    End,
}

fn yaml_value_as_str(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string().clone()),
        serde_yaml::Value::Bool(b) => Some(b.to_string().clone()),
        _ => None,
    }
}

impl Convert {
    pub(crate) fn process_python_line<W: Write>(
        &self,
        line: &str,
        py_indent_prior: usize,
        stream: &mut W,
        within_inst: &mut bool,
        inst_str: &mut String,
        inst_indent_space: &mut usize,
    ) -> Result<(), Box<dyn Error>> {
        match self.inst_state(line) {
            InstState::Begin => {
                // calculate the space before the <INST>
                // and print the Python code before the <INST>
                let all_space = &line.len() - line.trim_start().len();
                if all_space < py_indent_prior {
                    return Err("Indentation error: <INST> is not properly indented.".into());
                }
                *inst_indent_space = all_space - py_indent_prior;
                if *within_inst {
                    return Err("Nested <INST> is not allowed.".into());
                }
                *within_inst = true;
                writeln!(stream, "{}print('// INST')", " ".repeat(*inst_indent_space))?;
            }
            InstState::End => {
                if !*within_inst {
                    return Err("Encountering </INST> with no <INST> to end.".into());
                }
                *within_inst = false;
                self.print_inst(stream, inst_str, *inst_indent_space)?;
                inst_str.clear();
                writeln!(stream, "{}print('// END of INST')", " ".repeat(*inst_indent_space))?;
                *inst_indent_space = 0;
            }
            _ => {
                let useful_str = utf8_slice::from(&line, py_indent_prior);
                if *within_inst {
                    inst_str.push_str(&format!("{useful_str}\n"));
                } else {
                    // normal Python line
                    writeln!(stream, "{useful_str}")?;
                }
            }
        }
        Ok(())
    }

    fn inst_state(&self, line: &str) -> InstState {
        match line.trim() {
            "<INST>" => InstState::Begin,
            "</INST>" => InstState::End,
            _ => InstState::None,
        }
    }

    fn apply_protected_inst_group_regex(inst_str: &str) -> String {
        let re = Regex::new(r"!(\w+):(.*)[\r\n$]").unwrap();
        re.replace_all(inst_str, "__group_$1:$2\n").to_string()
    }

    fn inst_group_print_to_dot_inst(inst_str: &str, inst_indent_space: usize) -> String {
        let re = Regex::new(r"__group_\w+:\s*(.*)[\r\n$]").unwrap();
        re.replace_all(
            inst_str,
            format!(
                "''')\n{}_inst_file.write(f'{{_inst_var_map($1)}}')\n{}_inst_file.write(f'''",
                " ".repeat(inst_indent_space),
                " ".repeat(inst_indent_space)
            ),
        )
        .to_string()
    }

    fn print_inst<W: Write>(
        &self,
        stream: &mut W,
        inst_str: &str,
        inst_indent_space: usize,
    ) -> Result<(), Box<dyn Error>> {
        let inst_map: serde_yaml::Value =
            serde_yaml::from_str(&self.apply_protected_verilog_regex(
                Self::apply_protected_inst_group_regex(inst_str).as_str(),
            ))?;
        let mut inst_str_parsed = serde_yaml::to_string(&vec![&inst_map])?;
        inst_str_parsed = Self::inst_group_print_to_dot_inst(
            self.undo_protected_brackets(inst_str_parsed.as_str())
                .as_str(),
            inst_indent_space,
        );
        // print to .inst
        writeln!(stream, "{}_inst_file.write(f'''{}''')", " ".repeat(inst_indent_space), inst_str_parsed)?;
        // print to .v
        match inst_map["module"].as_str() {
            Some(module) => writeln!(
                stream,
                "{}print(f'{}', end='')",
                " ".repeat(inst_indent_space),
                self.undo_protected_brackets(module)
            )?,
            None => return Err("No module name found in the <INST>.".into()),
        }
        let mut first_vparam = true;
        if let Some(vparams) = inst_map["vparams"].as_mapping() {
            for (key, value) in vparams.iter() {
                if let (Some(key_str), Some(value_str)) = (key.as_str(), yaml_value_as_str(value)) {
                    let value_str = value_str.as_str();
                    if key_str.starts_with("__group_") {
                        writeln!(
                            stream,
                            "{}print(_verilog_vparams_var_map({}, {}), end='')",
                            " ".repeat(inst_indent_space),
                            value_str,
                            if first_vparam {
                                first_vparam = false;
                                "True"
                            } else {
                                "False"
                            },
                        )?;
                    } else {
                        writeln!(
                            stream,
                            "{}print(f'{}\\n  .{}({})', end='')",
                            " ".repeat(inst_indent_space),
                            if first_vparam {
                                first_vparam = false;
                                "#("
                            } else {
                                ","
                            },
                            self.escape_single_quote(
                                self.undo_protected_brackets(key_str).as_str()
                            ),
                            self.escape_single_quote(
                                self.undo_protected_brackets(value_str).as_str()
                            )
                        )?;
                    }
                } else {
                    return Err("Invalid vparams found in the <INST>.".into());
                }
            }
        }
        if !first_vparam {
            writeln!(stream, "{}print(')')", " ".repeat(inst_indent_space))?;
        }
        match inst_map["name"].as_str() {
            Some(name) => writeln!(
                stream,
                "{}print(f' {} (')",
                " ".repeat(inst_indent_space),
                self.undo_protected_brackets(name)
            )?,
            None => return Err("No instantiation name found in the <INST>.".into()),
        }
        let mut first_port = true;
        if let Some(ports) = inst_map["ports"].as_mapping() {
            for (key, value) in ports.iter() {
                if let (Some(key_str), Some(value_str)) = (key.as_str(), yaml_value_as_str(value)) {
                    let value_str = value_str.as_str();
                    if key_str.starts_with("__group_") {
                        writeln!(
                            stream,
                            "{}print(_verilog_ports_var_map({}, {}), end='')",
                            " ".repeat(inst_indent_space),
                            value_str,
                            if first_port {
                                first_port = false;
                                "True"
                            } else {
                                "False"
                            },
                        )?;
                    } else {
                        writeln!(
                            stream,
                            "{}print(f'{}  .{}({})', end='')",
                            " ".repeat(inst_indent_space),
                            if first_port {
                                first_port = false;
                                ""
                            } else {
                                ",\\n"
                            },
                            self.escape_single_quote(
                                self.undo_protected_brackets(key_str).as_str()
                            ),
                            self.escape_single_quote(
                                self.undo_protected_brackets(value_str).as_str()
                            )
                        )?;
                    }
                }
            }
        }
        writeln!(stream, "{}print(f'\\n);')", " ".repeat(inst_indent_space))?;

        Ok(())
    }

    fn undo_protected_brackets(&self, str: &str) -> String {
        str.replace("__LEFT_BRACKET__{", "{")
            .replace("}__RIGHT_BRACKET__", "}")
    }

    fn escape_single_quote(&self, str: &str) -> String {
        str.replace("'", "\\'")
    }
}
