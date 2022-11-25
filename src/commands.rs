use crate::{errors, Format, Molecule};

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use histogram::Histogram;
use needletail::parser::{self, LineEnding};
use needletail::FastxReader;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use textplots::{Chart, Plot, Shape};

const DNA: &[u8] = b"ACGT";
const RNA: &[u8] = b"ACGU";
const PROTEIN: &[u8] = b"ACDEFGHIKLMNPQRSTVWY";

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
        let std = hist.stddev().unwrap_or(0);
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
    }

    println!("{count}");

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
    sequence_type: Molecule,
    out: Option<PathBuf>,
    format: Format,
    line_ending: LineEnding,
) -> Result<(), Box<dyn Error>> {
    let mut writer = match out {
        Some(ref path) => Box::new(std::fs::File::create(Path::new(path))?) as Box<dyn Write>,
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };

    let charset = match sequence_type {
        Molecule::DNA => DNA,
        Molecule::RNA => RNA,
        Molecule::Protein => PROTEIN,
    };

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
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();

        match format {
            Format::Fasta => parser::write_fasta(id, seq.as_bytes(), &mut writer, line_ending),
            Format::Fastq => {
                parser::write_fastq(id, seq.as_bytes(), None, &mut writer, line_ending)
            }
        }?;
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

pub fn ids(input: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut reader = init_reader(input)?;

    while let Some(r) = reader.next() {
        let record = r?;
        match std::str::from_utf8(record.id()) {
            Ok(id) => println!("{id}"),
            Err(e) => {
                let msg = format!("Error reading id: {e}");
                return Err(errors::SeqError::new(&msg, record.id()).into());
            }
        };
    }

    Ok(())
}

pub fn convert(
    input: Option<PathBuf>,
    to: Format,
    out: Option<PathBuf>,
    line_ending: LineEnding,
) -> Result<(), Box<dyn Error>> {
    let mut reader = init_reader(input)?;
    let mut writer = match out {
        Some(ref path) => Box::new(std::fs::File::create(Path::new(path))?) as Box<dyn Write>,
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };

    match to {
        Format::Fasta => {
            while let Some(r) = reader.next() {
                let record = r?;
                let (id, seq): (&[u8], &[u8]) = (record.id(), &record.seq());
                parser::write_fasta(id, seq, &mut writer, line_ending)?;
            }
        }
        Format::Fastq => {
            while let Some(r) = reader.next() {
                let record = r?;
                let (id, seq): (&[u8], &[u8]) = (record.id(), &record.seq());
                parser::write_fastq(id, seq, None, &mut writer, line_ending)?;
            }
        }
    };

    Ok(())
}

pub fn select_by_ids(
    input: Option<PathBuf>,
    ids: Option<Vec<String>>,
    ids_file: Option<PathBuf>,
    out: Option<PathBuf>,
    line_ending: LineEnding,
) -> Result<(), Box<dyn Error>> {
    let mut to_select: HashSet<String> = HashSet::new();
    match (ids, ids_file) {
        (None, None) => return Err(errors::MainError::new("").into()),
        (Some(ids), None) => {
            for id in ids {
                to_select.insert(id);
            }
        }
        (None, Some(file)) => {
            let file = File::open(file)?;
            let buf_reader = BufReader::new(file);
            for id in buf_reader.lines() {
                let id = id?;
                to_select.insert(id);
            }
        }
        (Some(ids), Some(file)) => {
            for id in ids {
                to_select.insert(id);
            }
            let file = File::open(file)?;
            let buf_reader = BufReader::new(file);
            for id in buf_reader.lines() {
                let id = id?;
                to_select.insert(id);
            }
        }
    };

    let mut reader = init_reader(input)?;
    let mut writer = match out {
        Some(ref path) => Box::new(std::fs::File::create(Path::new(path))?) as Box<dyn Write>,
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };

    while let Some(r) = reader.next() {
        let record = r?;
        let (id, seq): (&[u8], &[u8]) = (record.id(), &record.seq());
        let id_s = String::from(std::str::from_utf8(id)?);
        if to_select.contains(&id_s) {
            parser::write_fasta(id, seq, &mut writer, line_ending)?;
        }
    }

    Ok(())
}

pub fn select_by_index(
    input: Option<PathBuf>,
    indices: Option<Vec<String>>,
    indices_file: Option<PathBuf>,
    out: Option<PathBuf>,
    line_ending: LineEnding,
) -> Result<(), Box<dyn Error>> {
    let mut to_select: HashSet<usize> = HashSet::new();
    match (indices, indices_file) {
        (None, None) => return Err(errors::MainError::new("").into()),
        (Some(indices), None) => {
            for index in indices {
                to_select.insert(index.parse::<usize>()?);
            }
        }
        (None, Some(file)) => {
            let file = File::open(file)?;
            let buf_reader = BufReader::new(file);
            for index in buf_reader.lines() {
                let index = index?;
                to_select.insert(index.parse::<usize>()?);
            }
        }
        (Some(indices), Some(file)) => {
            for index in indices {
                to_select.insert(index.parse::<usize>()?);
            }
            let file = File::open(file)?;
            let buf_reader = BufReader::new(file);
            for index in buf_reader.lines() {
                let index = index?;
                to_select.insert(index.parse::<usize>()?);
            }
        }
    };

    let mut reader = init_reader(input)?;
    let mut writer = match out {
        Some(ref path) => Box::new(std::fs::File::create(Path::new(path))?) as Box<dyn Write>,
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };

    let mut cursor: usize = 0;
    while let Some(r) = reader.next() {
        let record = r?;
        let (id, seq): (&[u8], &[u8]) = (record.id(), &record.seq());
        if to_select.contains(&cursor) {
            parser::write_fasta(id, seq, &mut writer, line_ending)?;
        }
        cursor += 1;
    }

    Ok(())
}

pub fn map_rename_sequences(
    input: Option<PathBuf>,
    map_file: Option<PathBuf>,
    out: Option<PathBuf>,
    line_ending: LineEnding,
) -> Result<(), Box<dyn Error>> {
    let mut new_names: HashMap<String, String> = HashMap::new();
    let file = match map_file {
        Some(path) => File::open(path)?,
        None => {
            return Err(
                errors::MainError::new("You must specify a name-mapping file or --number").into(),
            )
        }
    };
    let buf_reader = BufReader::new(file);
    for index in buf_reader.lines() {
        if let Ok(index) = index {
            let split: Vec<String> = index.split('\t').map(|s| s.to_owned()).collect();
            if split.len() != 2 {
                return Err(errors::MainError::new(
                    "You must specify '<old_name>\\t<new_name} in your map rename file",
                )
                .into());
            }
            new_names.insert(split[0].clone(), split[1].clone());
        } else {
            return Err(errors::MainError::new("Error parsing map file.").into());
        }
    }

    let mut reader = init_reader(input)?;
    let mut writer = match out {
        Some(ref path) => Box::new(std::fs::File::create(Path::new(path))?) as Box<dyn Write>,
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };

    while let Some(r) = reader.next() {
        let record = r?;
        let (id, seq): (&[u8], &[u8]) = (record.id(), &record.seq());
        let id_s = String::from(std::str::from_utf8(id)?);

        match new_names.get(&id_s) {
            Some(new) => parser::write_fasta(new.as_bytes(), seq, &mut writer, line_ending)?,
            None => parser::write_fasta(id, seq, &mut writer, line_ending)?,
        };
    }

    Ok(())
}

pub fn index_rename_sequences(
    input: Option<PathBuf>,
    out: Option<PathBuf>,
    line_ending: LineEnding,
) -> Result<(), Box<dyn Error>> {
    let mut reader = init_reader(input)?;
    let mut writer = match out {
        Some(ref path) => Box::new(std::fs::File::create(Path::new(path))?) as Box<dyn Write>,
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };

    let mut cursor: usize = 0;
    while let Some(r) = reader.next() {
        let record = r?;
        let new_id = format!("{cursor}");
        let seq: &[u8] = &record.seq();
        parser::write_fasta(new_id.as_bytes(), seq, &mut writer, line_ending)?;
        cursor += 1;
    }

    Ok(())
}
