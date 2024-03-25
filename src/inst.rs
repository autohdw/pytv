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
        py_indent_space: usize,
        stream: &mut W,
        within_inst: &mut bool,
        inst_str: &mut String,
    ) -> Result<(), Box<dyn Error>> {
        match self.inst_state(line) {
            InstState::Begin => {
                if *within_inst {
                    return Err("Nested <INST> is not allowed.".into());
                }
                *within_inst = true;
                writeln!(stream, "print('// INST')")?;
            }
            InstState::End => {
                if !*within_inst {
                    return Err("Encountering </INST> with no <INST> to end.".into());
                }
                *within_inst = false;
                self.print_inst(stream, inst_str)?;
                inst_str.clear();
                writeln!(stream, "print('// END of INST')")?;
            }
            _ => {
                let useful_str = utf8_slice::from(&line, py_indent_space);
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

    fn inst_group_print_to_dot_inst(inst_str: &str) -> String {
        let re = Regex::new(r"__group_\w+:\s*(.*)[\r\n$]").unwrap();
        re.replace_all(
            inst_str,
            "''')\n_inst_file.write(f'{_inst_var_map($1)}')\n_inst_file.write(f'''",
        )
        .to_string()
    }

    fn print_inst<W: Write>(&self, stream: &mut W, inst_str: &str) -> Result<(), Box<dyn Error>> {
        let inst_map: serde_yaml::Value =
            serde_yaml::from_str(&self.apply_protected_verilog_regex(
                Self::apply_protected_inst_group_regex(inst_str).as_str(),
            ))?;
        let mut inst_str_parsed = serde_yaml::to_string(&vec![&inst_map])?;
        inst_str_parsed = Self::inst_group_print_to_dot_inst(
            self.undo_protected_brackets(inst_str_parsed.as_str())
                .as_str(),
        );
        // print to .inst
        writeln!(stream, "_inst_file.write(f'''{}''')", inst_str_parsed)?;
        // print to .v
        match inst_map["module"].as_str() {
            Some(module) => writeln!(
                stream,
                "print(f'{}', end='')",
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
                            "print(_verilog_vparams_var_map({}, {}), end='')",
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
                            "print(f'{}\\n  parameter {} = {}', end='')",
                            if first_vparam {
                                first_vparam = false;
                                "#("
                            } else {
                                ","
                            },
                            self.escape_single_quote(self.undo_protected_brackets(key_str).as_str()),
                            self.escape_single_quote(self.undo_protected_brackets(value_str).as_str())
                        )?;
                    }
                } else {
                    return Err("Invalid vparams found in the <INST>.".into());
                }
            }
        }
        if !first_vparam {
            writeln!(stream, "print(f')')")?;
        }
        match inst_map["name"].as_str() {
            Some(name) => writeln!(
                stream,
                "print(f' {} (')",
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
                            "print(_verilog_ports_var_map({}, {}), end='')",
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
                            "print(f'{}  .{}({})', end='')",
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
        writeln!(stream, "print(f'\\n);')")?;

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
