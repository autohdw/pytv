use std::error::Error;
use std::io::Write;
// use std::io::{Write, Result as IoResult};
use super::Convert;

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
        inst_str: &str,
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
                if *within_inst {
                    inst_str.to_string().push_str(&format!("{line}\n"));
                } else {
                    writeln!(stream, "{}", utf8_slice::from(&line, py_indent_space))?;
                    // normal Python line
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
        // TODO: process YML and write to file
        writeln!(stream, "{}", inst_str)?;
        Ok(())
    }
}
