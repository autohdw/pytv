use super::Convert;
use std::error::Error;
use std::io::Write;

enum InstState {
    None,
    Begin,
    End,
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

    fn print_inst<W: Write>(&self, stream: &mut W, inst_str: &str) -> Result<(), Box<dyn Error>> {
        let inst_map: serde_yaml::Value =
            serde_yaml::from_str(&self.apply_protected_verilog_regex(inst_str))?;
        let mut inst_str_parsed = serde_yaml::to_string(&vec![inst_map])?;
        inst_str_parsed = inst_str_parsed.replace("__LEFT_BRACKET__{", "{");
        inst_str_parsed = inst_str_parsed.replace("}__RIGHT_BRACKET__", "}");
        writeln!(stream, "_inst_file.write(f'''{}''')", inst_str_parsed)?;
        Ok(())
    }
}
