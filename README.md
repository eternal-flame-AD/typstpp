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

### Backend

- [X] Mix Haskell and R code
- [ ] Inline code
- [ ] Raw output

### R

- [X] Better kable parsing
- [X] Mix of Graphics, Tables, and Text
- [ ] Passing arbitrary arguments to knitr

### Haskell

- [ ] Remove the need for `:{` and `:}`

## License

This project is licensed under the Apache-2.0 license, see [LICENSE](LICENSE) for more information.

### Dependency Licenses

Licenses for dependencies:

[Table of Dependencies w/SPDX identifiers](./LICENSE-dependencies)

A mapping of licenses to their SPDX identifiers can be found [here](https://spdx.org/licenses/).
