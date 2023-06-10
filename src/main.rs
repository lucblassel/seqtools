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
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
        /// Output in tabular format
        #[arg(short = 'b', long)]
        tabular: bool,
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
    #[clap(verbatim_doc_comment)]
    /// Select sequences from file by identifier or index
    ///
    /// ## Examples
    /// We have the following fasta file:
    /// ```
    /// >Seq1
    /// AAAAAAAAA
    /// >Seq2
    /// CCCCCCCCC
    /// >Seq3
    /// GGGGGGGGG
    /// >Seq4
    /// TTTTTTTTT
    /// >Seq5
    /// ATATATATA
    /// ```
    ///  
    /// `$ cat <fasta> | seqtools select Seq1 Seq5`
    /// ```
    /// >Seq1
    /// AAAAAAAAA
    /// >Seq5
    /// ATATATATA
    /// ```
    /// `$ cat <fasta> | seqtools select --use-indices 1 2`
    /// ```
    /// >Seq2
    /// CCCCCCCCC
    /// >Seq3
    /// GGGGGGGGG
    /// ```
    ///
    /// If you write ids (or indices) in a file, one per line as follows:  
    /// ```
    /// Seq1
    /// Seq5
    /// ```
    ///
    /// Then you can select from that file  
    /// `$ cat <fasta> | seqtools select -f <ids.txt>`
    /// ```
    /// >Seq1
    /// AAAAAAAAA
    /// >Seq5
    /// ATATATATA
    /// ```
    /// You can also specify additional ids as positional arguments  
    /// `$ cat <fasta> | seqtools select -f <ids.txt> Seq2`
    /// ```
    /// >Seq1
    /// AAAAAAAAA
    /// >Seq2
    /// CCCCCCCCC
    /// >Seq5
    /// ATATATATA
    /// ```
    Select {
        /// List of sequence identifiers
        ids: Option<Vec<String>>,
        /// Specify indices instead of identifiers (0-start index)
        #[arg(short, long)]
        use_indices: bool,
        /// Path to a file containing sequence identifiers (1 per line)
        #[arg(short = 'f', long, value_name = "FILE")]
        ids_file: Option<PathBuf>,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    #[clap(verbatim_doc_comment)]
    /// Rename sequences in a fasta file
    ///
    /// You can rename in several mutually exclusive ways:  
    ///
    ///    - Numbers: replace sequence header with its index
    ///
    ///    - File: You can define new names by writing them in a tab-separated
    ///            file with the following format on each line:
    ///            <old_name>\t<new_name>
    ///            Sequences whose name isn't specified in this file will not
    ///            be renamed.
    Rename {
        /// Rename the sequences with their index
        #[arg(short, long, group = "method")]
        number: bool,
        /// Tab delimited file for renaming sequences ('<original_id>\t<new_id>')
        #[arg(short = 'f', long, value_name = "FILE", group = "method")]
        map_file: Option<PathBuf>,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    #[clap(verbatim_doc_comment)]
    /// Add a common string to as a prefix or suffix to each sequence header
    ///
    /// A common use case would be to add a label to each sequence of different
    /// fasta files, with potentially duplicated sequence identifiers, in order
    /// to merge them and get unique sequence identifiers.
    AddId {
        /// identifier to add to each sequence header
        to_add: String,
        /// Adds the identifier as a prefix instead of suffix
        #[arg(short = 'p', long)]
        as_prefix: bool,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Remove a certain number of characters from the beginning or end of each sequence
    Trim {
        /// number of characters to trim from the sequence
        n_char: usize,
        ///Remove from the beginning of the sequence instead of the end
        #[arg(short = 's', long)]
        from_start: bool,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Clip all sequences in the alignment to a maximum length
    Clip {
        /// number of characters to trim from the sequence
        max_len: usize,
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Check if alignment has duplicate sequences
    Duplicates {
        /// Also show the identifiers of duplicated sequences instead of only the count
        /// as a JSON object.
        #[arg(short, long)]
        show_names: bool,
    },
    /// Remove duplicated sequences from the alignment
    DeDuplicate {
        /// Path to output file [default: stdout]
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
        /// Show the number and ids of removed sequences
        #[arg(short, long, action=clap::ArgAction::Count)]
        verbose: u8,
    },
    #[clap(verbatim_doc_comment)]
    /// Get an interactive view of the alignment (like Seaview)
    ///
    /// This command does not support reading an alignment from standard input,
    /// you must specify an alignment file using the --in flag.
    View,
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

pub mod viewer;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let line_ending = match std::env::consts::OS {
        "linux" | "macos" | "freebsd" | "netbsd" | "openbsd" => {
            needletail::parser::LineEnding::Unix
        }
        "windows" => needletail::parser::LineEnding::Windows,
        _ => return Err(errors::MainError::new("Windows is not supported..").into()),
    };

    match cli.command {
        Commands::Count => commands::count(cli.input),
        Commands::Length {
            summary,
            histogram,
            tabular,
        } => commands::length(cli.input, summary, histogram, tabular),
        Commands::Freqs { per_sequence } => commands::frequencies(cli.input, per_sequence),
        Commands::Random {
            num,
            len,
            std,
            sequence_type,
            out,
            format,
        } => commands::generate_random(num, len, std, sequence_type, out, format, line_ending),
        Commands::Ids => commands::ids(cli.input),
        Commands::Convert { to, out } => commands::convert(cli.input, to, out, line_ending),
        Commands::Select {
            ids,
            use_indices,
            ids_file,
            out,
        } => {
            if use_indices {
                commands::select_by_index(cli.input, ids, ids_file, out, line_ending)
            } else {
                commands::select_by_ids(cli.input, ids, ids_file, out, line_ending)
            }
        }
        Commands::Rename {
            number,
            map_file,
            out,
        } => {
            if number {
                commands::index_rename_sequences(cli.input, out, line_ending)
            } else {
                commands::map_rename_sequences(cli.input, map_file, out, line_ending)
            }
        }
        Commands::AddId {
            to_add,
            as_prefix,
            out,
        } => commands::add_id(cli.input, to_add, as_prefix, out, line_ending),
        Commands::Trim {
            n_char,
            from_start,
            out,
        } => commands::trim(cli.input, n_char, from_start, out, line_ending),
        Commands::Clip { max_len, out } => commands::clip(cli.input, max_len, out, line_ending),
        Commands::Duplicates { show_names } => commands::check_duplicates(cli.input, show_names),
        Commands::DeDuplicate { out, verbose } => {
            commands::remove_duplicates(cli.input, out, verbose, line_ending)
        }
        Commands::View => commands::view_alignment(cli.input),
    }?;

    Ok(())
}
