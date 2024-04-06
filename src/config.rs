use clap::Parser;
use regex::Regex;
use std::error::Error;

/// Represents the configuration options for PyTV.
#[derive(Debug)]
pub struct Config {
    /// The magic comment string used to identify template sections in the input file.
    pub magic_comment_str: String,
    /// The regular expression used to match template sections in the input file.
    pub template_re: Regex,
    /// Whether to run the Python script or not.
    pub run_python: bool,
    /// Whether to delete the Python script after running or not.
    pub delete_python: bool,
    /// The tab size used for parsing in the input file.
    pub tab_size: u32,
}

/// Represents the options for input and output file for PyTV.
#[derive(Debug, Default)]
pub struct FileOptions {
    /// The input file path.
    pub input: String,
    /// The output file path (optional).
    pub output: Option<String>,
}

impl Default for Config {
    /// Creates a new `Config` instance with default values.
    fn default() -> Self {
        Self::new(
            "!".to_string(),
            Self::default_template_re(),
            false,
            false,
            4,
        )
    }
}

/// Parse a single key-value pair
fn parse_key_val(s: &str) -> Result<(String, String), Box<dyn Error>>
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// Python Templated Verilog
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(index = 1, value_name = "FILE")]
    input: String,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
    /// Run python (keep Python script)
    #[arg(
        short = 'r',
        long = "run-py",
        conflicts_with = "run_python_del",
        default_value = "false"
    )]
    run_python: bool,
    /// Run python (delete Python script)
    #[arg(
        short = 'R',
        long = "run-py-del",
        conflicts_with = "run_python",
        default_value = "false"
    )]
    run_python_del: bool,
    /// Tab size
    #[arg(short, long, default_value = "4", value_name = "INT")]
    tab_size: u32,
    /// Magic comment string (after "//")
    #[arg(short, long, default_value = "!", value_name = "STRING")]
    magic: String,
    /// Variables (multiple occurrences allowed)
    #[arg(short, long = "var", value_name = "KEY=VAL")]
    vars: Vec<String>,
    /// Preamble Python file
    #[arg(short, long = "preamble", value_name = "FILE")]
    preamble_py: Option<String>
}

impl Config {
    /// Creates a new `Config` instance with the specified values.
    ///
    /// # Example
    /// ```
    /// use pytv::Config;
    /// use regex::Regex;
    /// let config = Config::new("!".to_string(), Regex::new(r"`([^`]+)`").unwrap(), false, false, 4);
    /// let default_config = Config::default();
    /// assert_eq!(config.magic_comment_str, default_config.magic_comment_str);
    /// ```
    pub fn new(
        magic_comment_str: String,
        template_re: Regex,
        run_python: bool,
        delete_python: bool,
        tab_size: u32,
    ) -> Config {
        Config {
            magic_comment_str,
            template_re,
            run_python,
            delete_python,
            tab_size,
        }
    }

    /// Parses the command line arguments and returns a tuple of `Config` and `FileOptions`.
    pub fn from_args() -> (Config, FileOptions, Option<Vec<(String, String)>>, Option<String>) {
        let args = Args::parse();
        let vars = args
            .vars
            .iter()
            .map(|s| parse_key_val(s))
            .collect::<Result<Vec<(String, String)>, Box<dyn Error>>>();
        if vars.is_err() {
            eprintln!("Error: {}", vars.err().unwrap());
            std::process::exit(1);
        }
        (
            Self::new(
                args.magic,
                Self::default_template_re(),
                args.run_python || args.run_python_del,
                args.run_python_del && !args.run_python,
                args.tab_size,
            ),
            FileOptions {
                input: args.input,
                output: args.output,
            },
            vars.ok(),
            args.preamble_py,
        )
    }

    /// Returns the default regular expression used to match template sections in the input file.
    fn default_template_re() -> Regex {
        Regex::new(r"`([^`]+)`").unwrap()
    }
}
