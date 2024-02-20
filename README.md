# typstpp

The Typst preprocessor. (Or Typst++)...

Executes your Haskell or R code in your Typst source file. Wrapping around the `compile` and `watch` commands of the Typst CLI.

## Installation

```bash
cargo install --git https://github.com/eternal-flame-AD/typstpp.git \
    --features "r hs" \
    --locked
```

## Usage

```bash
typstpp --help
```

```
The Typst preprocessor

Usage: typstpp <COMMAND>

Commands:
  info        Print typstpp info
  preprocess  Preprocess a typst file
  compile     Preprocess and compile a typst file
  watch       Watch a typst file and preprocess then recompile on changes
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Example

See [example.typ](example.typ). For an example input.

See [example.out.typ](example.out.typ). For the preprocess output.

See [example.out.pdf](example.out.pdf). For the final PDF output.

## TODO

- [X] Better kable parsing
- [ ] Inline code