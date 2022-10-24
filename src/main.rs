use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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
    },
    /// Generate random sequences
    Generate {
        /// number of sequences to generate
        #[arg(short, long, default_value_t = 10)]
        num: i32,
        /// Length of sequences to generate
        #[arg(short, long, default_value_t = 100)]
        len: u32,
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

fn main() {
    let cli = Cli::parse();

    if cli.command.is_none() {
        panic!("You must specify a command.");
    }

    let line_ending = match std::env::consts::OS {
        "linux" | "macos" | "freebsd" | "netbsd" | "openbsd" => {
            needletail::parser::LineEnding::Unix
        }
        "windows" => needletail::parser::LineEnding::Windows,
        _ => panic!("This platform is not supported."),
    };

    match cli.command {
        Some(Commands::Count) => commands::count(cli.input),
        Some(Commands::Length { summary }) => commands::length(cli.input, summary),
        Some(Commands::Generate { num, len, format }) => {
            commands::generate(num, len, format, line_ending)
        }
        None => unreachable!(),
    };
}

pub mod commands {
    use needletail::parser::{self, LineEnding};
    use needletail::FastxReader;
    use rand::Rng;
    use std::path::PathBuf;

    const CHARSET: &[u8] = b"ACGT";

    use crate::Format;

    fn init_reader(input: Option<PathBuf>) -> Box<dyn FastxReader> {
        match input {
            Some(path) => needletail::parse_fastx_file(path),
            None => needletail::parse_fastx_stdin(),
        }
        .unwrap()
    }

    pub fn count(input: Option<PathBuf>) {
        let mut reader = init_reader(input);

        let mut count = 0;
        while let Some(r) = reader.next() {
            let _ = r.expect("Invalid record");
            count += 1;
        }

        println!("{count} sequences");
    }

    pub fn length(input: Option<PathBuf>, stats: bool) {
        let mut reader = init_reader(input);

        if stats {
            let l: usize = match reader.next().expect("Invalid record") {
                Ok(record) => record.seq().len(),
                Err(e) => panic!("Error reading input: {e}"),
            };

            let mut max: usize = l;
            let mut min: usize = l;
            let mut total: usize = 0;
            let mut count: usize = 0;

            while let Some(r) = reader.next() {
                let record = r.expect("Invalid record");
                let l = record.seq().len();
                total += l;
                count += 1;

                if l > max {
                    max = l;
                } else if l < min {
                    min = l;
                }
            }
            println!("Min:\t{min}");
            println!("Max:\t{max}");
            println!("Mean:\t{}", total / count);
        } else {
            while let Some(r) = reader.next() {
                let record = r.expect("Invalid record");
                println!(
                    "{}\t{}",
                    std::str::from_utf8(record.id()).unwrap(),
                    record.seq().len()
                );
            }
        }
    }

    pub fn generate(num: i32, len: u32, format: super::Format, line_ending: LineEnding) {
        let mut writer = std::io::stdout();

        let mut rng = rand::thread_rng();

        for i in 0..num {
            let id_str = format!("S{i}");
            let id = id_str.as_bytes();

            let seq: String = (0..len)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();

            match format {
                Format::A => parser::write_fasta(id, seq.as_bytes(), &mut writer, line_ending),
                Format::Q => {
                    parser::write_fastq(id, seq.as_bytes(), None, &mut writer, line_ending)
                }
            }
            .unwrap();
        }
    }
}
