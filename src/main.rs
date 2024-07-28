use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    process::exit,
};

use clap::Parser;

/// A C preproc. written in Rust
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input C file
    #[arg(short, long)]
    input: String,
    /// Output C file
    #[arg(short, long)]
    output: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut preproc = Preprocessor::new(args)?;

    preproc.run()?;
    eprintln!("{:?}", preproc);

    Ok(())
}

#[derive(Debug)]
struct Preprocessor {
    input: String,
    definitions: HashSet<String>,
    substitutions: HashMap<String, String>,
    includes: HashSet<String>,
}

#[derive(Debug)]
enum PreprocessorError {
    EmptyDirective,
    InvalidInclude,
}

impl Error for PreprocessorError {}

impl fmt::Display for PreprocessorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PreprocessorError::EmptyDirective => write!(f, "Empty directive encountered!"),
            PreprocessorError::InvalidInclude => write!(f, "Tried to include non-existant file!"),
        }
    }
}

impl Preprocessor {
    fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            input: std::fs::read_to_string(args.input)?,
            definitions: HashSet::new(),
            substitutions: HashMap::new(),
            includes: HashSet::new(),
        })
    }

    fn process_define(&mut self, line: Vec<&str>) -> Result<(), PreprocessorError> {
        if line.is_empty() {
            return Err(PreprocessorError::EmptyDirective);
        }
        match line.len() {
            2 => {
                self.substitutions.remove(line[1]);
                self.definitions.insert(line[1].to_string());
            }
            _ => {
                self.definitions.remove(line[1]);
                self.substitutions
                    .insert(line[1].to_string(), line[2..].join(" ").to_string());
            }
        };
        Ok(())
    }

    fn process_include(&mut self, line: Vec<&str>) -> Result<(), PreprocessorError> {
        if line.is_empty() {
            return Err(PreprocessorError::EmptyDirective);
        }
        let name = &line[1][1..line.len() - 1];
        if std::fs::File::open(name).is_err() {
            return Err(PreprocessorError::InvalidInclude);
        }
        self.includes.insert(name.to_owned());
        Ok(())
    }

    fn process_undef(&mut self, line: Vec<&str>) -> Result<(), PreprocessorError> {
        if line.is_empty() {
            return Err(PreprocessorError::EmptyDirective);
        }
        self.definitions.remove(line[1]);
        Ok(())
    }

    fn run(&mut self) -> Result<String, PreprocessorError> {
        let mut result = String::new();

        let input = self.input.clone();
        for line in input.lines() {
            if line.starts_with('#') {
                let line = line.split_whitespace().collect::<Vec<_>>();
                match line[0] {
                    "#define" => self.process_define(line)?,
                    "#include" => self.process_include(line)?,
                    "#undef" => self.process_undef(line)?,
                    "#region" | "#endregion" => {}
                    _ => {
                        eprintln!("Not supported directive found! {}", line[0]);
                        exit(1);
                    }
                };
            }
            result.push_str(line);
        }

        Ok(result)
    }
}
