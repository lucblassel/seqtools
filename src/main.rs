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
    };
}

pub mod commands {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use histogram::Histogram;
    use needletail::parser::{self, LineEnding};
    use needletail::FastxReader;
    use rand::Rng;
    use rand_distr::{Distribution, Normal};
    use textplots::{Chart, Plot, Shape};

    const CHARSET: &[u8] = b"ACGT";

    use crate::Format;

    fn init_reader(input: Option<PathBuf>) -> Box<dyn FastxReader> {
        match input {
            Some(path) => needletail::parse_fastx_file(path),
            None => needletail::parse_fastx_stdin(),
        }
        .unwrap()
    }

    fn draw_hist(hist: &mut Histogram) {
        let min_x = hist.minimum().unwrap();
        let max_x = hist.maximum().unwrap();

        let points: Vec<(f32, f32)> = hist
            .into_iter()
            .map(|bucket| (bucket.value() as f32, bucket.count() as f32))
            .filter(|(_, c)| *c > 0.)
            .collect();

        let chart = Chart::new(200, 50, min_x as f32 - 1., max_x as f32 + 1.)
            .lineplot(&Shape::Bars(&points))
            .to_string();

        eprintln!("{chart}");
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

    pub fn length(input: Option<PathBuf>, stats: bool, histogram: bool) {
        let mut reader = init_reader(input);

        if stats {
            let mut hist = Histogram::new();

            while let Some(r) = reader.next() {
                let record = r.expect("Invalid record");
                let l = record.seq().len();
                hist.increment(l as u64)
                    .expect("Error incrementing histogram");
            }

            let (min, max) = (hist.minimum().unwrap(), hist.maximum().unwrap());
            let (mean, std) = (hist.mean().unwrap(), hist.stddev().unwrap());
            let (median, q1, q3) = (
                hist.percentile(50.).unwrap(),
                hist.percentile(25.).unwrap(),
                hist.percentile(75.).unwrap(),
            );

            if histogram {
                draw_hist(&mut hist);
                eprintln!("Min: {min}\tMax: {max}\tMean: {mean}\tSdev: {std}\tQ1: {q1}\tMedian: {median}\tQ3: {q3}",);
            } else {
                println!("Min:\t{min}");
                println!("Max:\t{max}");
                println!("Mean:\t{mean}");
                println!("Sdev:\t{std}");
                println!("Q1:\t{q1}");
                println!("Median:\t{median}");
                println!("Q3:\t{q3}");
            }
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

    pub fn generate_random(
        num: i32,
        len: f64,
        std: f64,
        format: super::Format,
        line_ending: LineEnding,
    ) {
        let mut writer = std::io::stdout();

        let mut rng = rand::thread_rng();
        let mut hist = Histogram::new();

        let normal = Normal::new(len, std).unwrap();

        for i in 0..num {
            let id_str = format!("S{i}");
            let id = id_str.as_bytes();

            let x: u64 = normal.sample(&mut rng) as u64;
            hist.increment(x).unwrap();

            let seq: String = (0..x)
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

        if std > 0. {
            let (min, max) = (hist.minimum().unwrap(), hist.maximum().unwrap());
            let (mean, std) = (hist.mean().unwrap(), hist.stddev().unwrap());
            let (median, q1, q3) = (
                hist.percentile(50.).unwrap(),
                hist.percentile(25.).unwrap(),
                hist.percentile(75.).unwrap(),
            );

            draw_hist(&mut hist);
            eprintln!("Min: {min}\tMax: {max}\tMean: {mean}\tSdev: {std}\tQ1: {q1}\tMedian: {median}\tQ3: {q3}",);
        }
    }

    pub fn frequencies(input: Option<PathBuf>, per_sequence: bool) {
        let mut reader = init_reader(input);

        if per_sequence {
            while let Some(r) = reader.next() {
                let mut counter: HashMap<u8, u32> = HashMap::new();
                let record = r.expect("Error parsing record");
                for c in record.seq().iter() {
                    counter
                        .entry(*c)
                        .and_modify(|count| *count += 1)
                        .or_insert(0);
                }
                print!("{}", std::str::from_utf8(record.id()).unwrap());
                let total: u32 = counter.values().sum();
                let mut keys: Vec<&u8> = counter.keys().collect();
                keys.sort();

                for key in keys {
                    let val = counter.get(key).unwrap();
                    let p = (*val as f64 / total as f64) * 100.;
                    print!("\t{}: {} {p:.2}%", *key as char, val);
                }
                println!();
            }
        } else {
            let mut counter: HashMap<u8, u32> = HashMap::new();
            while let Some(r) = reader.next() {
                let record = r.expect("Error parsing record");
                for c in record.seq().iter() {
                    counter
                        .entry(*c)
                        .and_modify(|count| *count += 1)
                        .or_insert(0);
                }
            }
            let total: u32 = counter.values().sum();
            for (key, val) in counter.iter() {
                let p = (*val as f64 / total as f64) * 100.;
                println!("{}\t{}\t{p:.2} %", *key as char, val);
            }
        }
    }
}
