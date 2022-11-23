# Seqtools

this is a simple FASTx command line utility, built in rust.  
This tool is designed to work with the UNIX philosphy and by default reads from stdin and writes to stdout. However there is always an option to specify an input file (which is probably better for bigger files), and commands that can have large outputs typically have an option to specify an output file.

## Usage

```
Seqtools is a simple utility to work with FASTX files from the command line.
It seamlessly handles compressed files (.gz, .xz or bz2 formats)

Usage: seqtools [OPTIONS] [COMMAND]

Commands:
  count    Counts the number of sequences in FASTX data
  length   Get length in nucleotides of sequences
  freqs    Get statistics about frequencies in the file
  random   Generate random sequences with normally distributed lengths
  ids      Extract sequence ids
  convert  Convert file to format
  help     Print this message or the help of the given subcommand(s)

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -h, --help       Print help information
```

### `count`
```
Counts the number of sequences in FASTX data

Usage: seqtools count [OPTIONS]

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -h, --help       Print help information
```

### `length`
```
Get length in nucleotides of sequences

Usage: seqtools length [OPTIONS]

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -s, --summary    Report statistics about lengths instead of individual lengths
  -t, --histogram  Draw a histogram of lengths
  -h, --help       Print help information
```

### `freqs`
```
Get statistics about frequencies in the file

Usage: seqtools freqs [OPTIONS]

Options:
  -i, --in <FILE>     Path to an input FASTX file. Reads from stdin by default
  -s, --per-sequence  Get frequencies per sequence instead of globally
  -h, --help          Print help information
```

### `random`
```
Generate random sequences with normally distributed lengths

Usage: seqtools random [OPTIONS]

Options:
  -i, --in <FILE>
          Path to an input FASTX file. Reads from stdin by default
  -n, --num <NUM>
          number of sequences to generate [default: 10]
  -l, --len <LEN>
          Average length of sequences to generate [default: 100]
  -s, --std <STD>
          Standard deviation of read length [default: 0]
  -t, --sequence-type <SEQUENCE_TYPE>
          Sequence type to generate [default: dna] [possible values: dna, rna, protein]
  -o, --out <FILE>
          Path to output file (default is stdout)
  -f, --format <FORMAT>
          Format of generated sequences [default: fasta] [possible values: fasta, fastq]
  -h, --help
          Print help information
```

### `ids`
```
Extract sequence ids

Usage: seqtools ids [OPTIONS]

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -h, --help       Print help information
```

### `convert`
```
Convert file to format

Usage: seqtools convert [OPTIONS]

Options:
  -i, --in <FILE>   Path to an input FASTX file. Reads from stdin by default
  -t, --to <TO>     Format of output sequences [default: fasta] [possible values: fasta, fastq]
  -o, --out <FILE>  Path to output file (default is stdout)
  -h, --help        Print help information
```