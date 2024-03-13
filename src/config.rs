use clap::Parser;
use regex::Regex;

// class for storing configuration
#[derive(Debug)]
pub struct Config {
    pub magic_comment_str: String,
    pub template_re: Regex,
}

#[derive(Debug)]
pub struct FileOptions {
    pub input: String,
    pub output: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new("!".to_string(), Self::default_template_re())
    }
}

/// Python Templated Verilog
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// input file
    #[arg(index = 1, value_name = "FILE")]
    input: String,
    /// output file
    #[arg(short, long)]
    output: Option<String>,
    /// Magic comment string (after "//")
    #[arg(short, long, default_value = "!")]
    magic: String,
}

impl Config {
    pub fn new(magic_comment_str: String, template_re: Regex) -> Config {
        Config {
            magic_comment_str,
            template_re,
        }
    }

    /// Parse command line arguments and return a tuple of Config and FileOptions
    pub fn from_args() -> (Config, FileOptions) {
        let args = Args::parse();
        (
            Self::new(args.magic, Self::default_template_re()),
            FileOptions {
                input: args.input,
                output: args.output,
            },
        )
    }

    fn default_template_re() -> Regex {
        Regex::new(r"`(\w+)`").unwrap()
    }
}
