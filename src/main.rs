use clap::{Parser, Subcommand, ValueEnum};
use std::error::Error;
use std::path::PathBuf;

mod commands;
mod errors;

#[derive(Parser, Debug)]
#[clap(author, version, verbatim_doc_comment)]
/// Seqtools is a simple utility to work with FASTX files from the command line.
/// It seamlessly handles compressed files (.gz, .xz or bz2 formats).
pub struct Cli {
    /// Path to an input FASTX file. [default: stdin]
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
        /// Sequence type to generate
        #[arg(short='t', long, value_enum, default_value_t=Molecule::DNA)]
        sequence_type: Molecule,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
        /// Format of generated sequences
        #[arg(short, long, value_enum, default_value_t=Format::Fasta)]
        format: Format,
    },
    /// Extract sequence ids
    Ids,
    /// Convert file to format
    Convert {
        /// Format of output sequences
        #[arg(short, long, value_enum, default_value_t=Format::Fasta)]
        to: Format,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Select sequences from file by identifier or index
    Select {
        /// List of sequence identifiers
        ids: Option<Vec<String>>,
        /// Specify indices instead of identifiers
        #[arg(short, long)]
        use_indices: bool,
        /// Path to a file containing sequence identifiers
        #[arg(short = 'f', long, value_name = "FILE")]
        ids_file: Option<PathBuf>,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum Format {
    Fasta,
    Fastq,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum Molecule {
    DNA,
    RNA,
    Protein,
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
            sequence_type,
            out,
            format,
        }) => commands::generate_random(num, len, std, sequence_type, out, format, line_ending),
        Some(Commands::Ids) => commands::ids(cli.input),
        Some(Commands::Convert { to, out }) => commands::convert(cli.input, to, out, line_ending),
        Some(Commands::Select {
            ids,
            use_indices,
            ids_file,
            out,
        }) => {
            if use_indices {
                commands::select_by_index(cli.input, ids, ids_file, out, line_ending)
            } else {
                commands::select_by_ids(cli.input, ids, ids_file, out, line_ending)
            }
        }
        None => unreachable!(),
    }?;

    Ok(())
}
