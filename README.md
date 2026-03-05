# Recursive Hash

A GUI & CLI application for recursively hashing files.

## Install

Clone the repository to your local machine and run

```bash
RUSTFLAGS="-C target-cpu=native" cargo install --path .
```

to install the non-GUI version. (recursive-hash gui is not available)

To build with GTK4 gui support, run

```bash
RUSTFLAGS="-C target-cpu=native" cargo install --path . --features gui
```

## Usage

The GUI has not been implemented yet. The command line options are as follows:

```
Usage: recursive-hash cli [OPTIONS]

Options:
  -a, --audit <AUDIT>        Check file hashes according to the checklist file.
  -p, --paths <PATHS>        Path to the file or directory to hash [default: .]
  -e, --excludes <EXCLUDES>  Path to the file or directory to exclude from hashing [default: checklist,checklist.md5,checklist.sha1,checklist.sha256,checklist.sha384,checklist.sha512]
  -m, --hasher <HASHER>      Hashing algorithm to use [default: sha256] [possible values: md5, sha1, sha256, sha384, sha512]
  -o, --out <OUT>            Path to the output file [default: checklist.sha256]
  -h, --help                 Print help
```

## Output

The output file format is compatible with the `sha256sum` | `md5sum` command, which is:

```
<hash>  <file_path>
```

Note that `<file_path>` is the relative path to the `<OUT>` file.
