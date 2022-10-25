use crate::errors;

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use histogram::Histogram;
use needletail::parser::{self, LineEnding};
use needletail::FastxReader;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use textplots::{Chart, Plot, Shape};

const CHARSET: &[u8] = b"ACGT";

use crate::Format;

struct SumStats {
    min: u64,
    max: u64,
    mean: u64,
    std: u64,
    median: u64,
    q1: u64,
    q3: u64,
}

impl SumStats {
    fn from_hist(hist: &Histogram) -> Result<Self, Box<dyn Error>> {
        let (min, max) = (hist.minimum()?, hist.maximum()?);
        let mean = hist.mean()?;
        let std = match hist.stddev() {
            Some(s) => s,
            None => 0,
        };
        let (median, q1, q3) = (
            hist.percentile(50.)?,
            hist.percentile(25.)?,
            hist.percentile(75.)?,
        );

        Ok(SumStats {
            min,
            max,
            mean,
            std,
            median,
            q1,
            q3,
        })
    }

    fn print_row(&self) {
        eprintln!(
            "Min: {}\tMax: {}\tMean: {}\tSdev: {}\tQ1: {}\tMedian: {}\tQ3: {}",
            self.min, self.max, self.mean, self.std, self.q1, self.median, self.q3
        );
    }

    fn print_col(&self) {
        println!("Min:\t{}", self.min);
        println!("Max:\t{}", self.max);
        println!("Mean:\t{}", self.mean);
        println!("Sdev:\t{}", self.std);
        println!("Q1:\t{}", self.q1);
        println!("Median:\t{}", self.median);
        println!("Q3:\t{}", self.q3);
    }
}

fn init_reader(
    input: Option<PathBuf>,
) -> Result<Box<dyn FastxReader>, needletail::errors::ParseError> {
    match input {
        Some(path) => needletail::parse_fastx_file(path),
        None => needletail::parse_fastx_stdin(),
    }
}

fn draw_hist(hist: &mut Histogram) -> Result<(), Box<dyn Error>> {
    let min_x = hist.minimum()?;
    let max_x = hist.maximum()?;

    let points: Vec<(f32, f32)> = hist
        .into_iter()
        .map(|bucket| (bucket.value() as f32, bucket.count() as f32))
        .filter(|(_, c)| *c > 0.)
        .collect();

    let chart = Chart::new(200, 50, min_x as f32 - 1., max_x as f32 + 1.)
        .lineplot(&Shape::Bars(&points))
        .to_string();

    eprintln!("{chart}");

    Ok(())
}

pub fn count(input: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut reader = init_reader(input)?;

    let mut count = 0;
    while let Some(r) = reader.next() {
        match r {
            Ok(_) => count += 1,
            Err(e) => return Err(e.into()),
        }
        let _ = r.expect("Invalid record");
        count += 1;
    }

    println!("{count} sequences");

    Ok(())
}

pub fn length(input: Option<PathBuf>, stats: bool, histogram: bool) -> Result<(), Box<dyn Error>> {
    let mut reader = init_reader(input)?;

    if stats {
        let mut hist = Histogram::new();

        while let Some(r) = reader.next() {
            let record = r?;
            let l = record.seq().len();
            hist.increment(l as u64)
                .expect("Error incrementing histogram");
        }

        let stats = SumStats::from_hist(&hist)?;

        if histogram {
            draw_hist(&mut hist)?;
            stats.print_row();
        } else {
            stats.print_col();
        }
    } else {
        while let Some(r) = reader.next() {
            let record = r?;
            println!(
                "{}\t{}",
                std::str::from_utf8(record.id())?,
                record.seq().len()
            );
        }
    }

    Ok(())
}

pub fn generate_random(
    num: i32,
    len: f64,
    std: f64,
    format: super::Format,
    line_ending: LineEnding,
) -> Result<(), Box<dyn Error>> {
    let mut writer = std::io::stdout();

    let mut rng = rand::thread_rng();
    let mut hist = Histogram::new();

    let normal = Normal::new(len, std)?;

    for i in 0..num {
        let id_str = format!("S{i}");
        let id = id_str.as_bytes();

        let x: u64 = normal.sample(&mut rng) as u64;
        hist.increment(x)?;

        let seq: String = (0..x)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        match format {
            Format::A => parser::write_fasta(id, seq.as_bytes(), &mut writer, line_ending),
            Format::Q => parser::write_fastq(id, seq.as_bytes(), None, &mut writer, line_ending),
        }?;
    }

    if std > 0. {
        let stats = SumStats::from_hist(&hist)?;
        draw_hist(&mut hist)?;
        stats.print_row();
    }

    Ok(())
}

pub fn frequencies(input: Option<PathBuf>, per_sequence: bool) -> Result<(), Box<dyn Error>> {
    let mut reader = init_reader(input)?;

    if per_sequence {
        while let Some(r) = reader.next() {
            let mut counter: HashMap<u8, u32> = HashMap::new();
            let record = r?;
            for c in record.seq().iter() {
                counter
                    .entry(*c)
                    .and_modify(|count| *count += 1)
                    .or_insert(0);
            }
            print!("{}", std::str::from_utf8(record.id())?);
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
            let record = r?;
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

    Ok(())
}
