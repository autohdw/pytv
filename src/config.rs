use clap::Parser;
use regex::Regex;

// class for storing configuration
#[derive(Debug)]
pub struct Config {
    pub magic_comment_str: String,
    pub template_re: Regex,
    pub run_python: bool,
    pub delete_python: bool,
    pub tab_size: u32,
}

#[derive(Debug)]
pub struct FileOptions {
    pub input: String,
    pub output: Option<String>,
}

impl Default for Config {
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
}

impl Config {
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

    /// Parse command line arguments and return a tuple of Config and FileOptions
    pub fn from_args() -> (Config, FileOptions) {
        let args = Args::parse();
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
        )
    }

    fn default_template_re() -> Regex {
        Regex::new(r"`([^`]+)`").unwrap()
    }
}
