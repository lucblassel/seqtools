use clap::{Parser, Subcommand, ValueEnum};
use std::error::Error;
use std::path::PathBuf;

mod commands;
mod errors;

#[derive(Parser, Debug)]
/// Seqtools is a simple utility to work with FASTX files from the command line.
/// Seamlessly handles compressed files (.gz, .xz or bz2 formats).
struct Cli {
    /// Path to an input FASTX file. Reads from stdin by default
    #[arg(short, long = "in", value_name = "FILE", global = true)]
    input: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Counts the number of sequences in FASTX data
    Count,
    /// Get length in nucleotides of sequences
    Length {
        /// Report statistics about lengths instead of individual lengths
        #[arg(short, long)]
        summary: bool,
        /// Draw a histogram of lengths
        #[arg(short = 't', long)]
        histogram: bool,
    },
    /// Get statistics about frequencies in the file
    Freqs {
        /// Get frequencies per sequence instead of globally
        #[arg(short = 's', long = "per-sequence")]
        per_sequence: bool,
    },
    /// Generate random sequences with normally distributed lengths
    Random {
        /// number of sequences to generate
        #[arg(short, long, default_value_t = 10)]
        num: i32,
        /// Average length of sequences to generate
        #[arg(short, long, default_value_t = 100.)]
        len: f64,
        /// Standard deviation of read length
        #[arg(short, long, default_value_t = 0.)]
        std: f64,
        /// Format of generated sequences (FAST(a) or FAST(q))
        #[arg(short, long, value_enum, default_value_t=Format::A)]
        format: Format,
    },
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum Format {
    /// Fasta
    A,
    /// Fastq
    Q,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if cli.command.is_none() {
        return Err(errors::MainError::new("You must specify a command.").into());
    }

    let line_ending = match std::env::consts::OS {
        "linux" | "macos" | "freebsd" | "netbsd" | "openbsd" => {
            needletail::parser::LineEnding::Unix
        }
        "windows" => needletail::parser::LineEnding::Windows,
        _ => return Err(errors::MainError::new("Windows is not supported..").into()),
    };

    match cli.command {
        Some(Commands::Count) => commands::count(cli.input),
        Some(Commands::Length { summary, histogram }) => {
            commands::length(cli.input, summary, histogram)
        }
        Some(Commands::Freqs { per_sequence }) => commands::frequencies(cli.input, per_sequence),
        Some(Commands::Random {
            num,
            len,
            std,
            format,
        }) => commands::generate_random(num, len, std, format, line_ending),
        None => unreachable!(),
    }?;

    Ok(())
}
