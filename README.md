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
  select   Select sequences from file by identifier or index
  help     Print this message or the help of the given subcommand(s)

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -h, --help       Print help information
```

Jump to command:
 - [count](#count)
 - [length](#length)
 - [freqs](#freqs)
 - [random](#random)
 - [ids](#ids)
 - [convert](#convert)
 - [select](#select)

### count
```
Counts the number of sequences in FASTX data

Usage: seqtools count [OPTIONS]

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -h, --help       Print help information
```

### length
```
Get length in nucleotides of sequences
Usage: seqtools length [OPTIONS]

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -s, --summary    Report statistics about lengths instead of individual lengths
  -t, --histogram  Draw a histogram of lengths
  -h, --help       Print help information
```

### freqs
```
Get statistics about frequencies in the file

Usage: seqtools freqs [OPTIONS]

Options:
  -i, --in <FILE>     Path to an input FASTX file. Reads from stdin by default
  -s, --per-sequence  Get frequencies per sequence instead of globally
  -h, --help          Print help information
```

### random
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

### ids
```
Extract sequence ids

Usage: seqtools ids [OPTIONS]

Options:
  -i, --in <FILE>  Path to an input FASTX file. Reads from stdin by default
  -h, --help       Print help information
```

### convert
```
Convert file to format

Usage: seqtools convert [OPTIONS]

Options:
  -i, --in <FILE>   Path to an input FASTX file. Reads from stdin by default
  -t, --to <TO>     Format of output sequences [default: fasta] [possible values: fasta, fastq]
  -o, --out <FILE>  Path to output file (default is stdout)
  -h, --help        Print help information
```

### select
Select sequences from file by identifier or index

#### General usage
```
Select sequences from file by identifier or index

Usage: seqtools select [OPTIONS] [IDS]...

Arguments:
  [IDS]...  List of sequence identifiers

Options:
  -i, --in <FILE>        Path to an input FASTX file. [default: stdin]
  -u, --use-indices      Specify indices instead of identifiers (0-start index)
  -f, --ids-file <FILE>  Path to a file containing sequence identifiers (1 per line)
  -o, --out <FILE>       Path to output file [default: stdout]
  -h, --help             Print help information (use `--help` for more detail)
```

#### Examples
We have the following fasta file:
```
>Seq1
AAAAAAAAA
>Seq2
CCCCCCCCC
>Seq3
GGGGGGGGG
>Seq4
TTTTTTTTT
>Seq5
ATATATATA
```
 
`$ cat <fasta> | seqtools select Seq1 Seq5`
```
>Seq1
AAAAAAAAA
>Seq5
ATATATATA
```
`$ cat <fasta> | seqtools select --use-indices 1 2`
```
>Seq2
CCCCCCCCC
>Seq3
GGGGGGGGG
```

If you write ids (or indices) in a file, one per line as follows:  
```
Seq1
Seq5
```

Then you can select from that file  
`$ cat <fasta> | seqtools select -f <ids.txt>`
```
>Seq1
AAAAAAAAA
>Seq5
ATATATATA
```
You can also specify additional ids as positional arguments  
`$ cat <fasta> | seqtools select -f <ids.txt> Seq2`
```
>Seq1
AAAAAAAAA
>Seq2
CCCCCCCCC
>Seq5
ATATATATA
```