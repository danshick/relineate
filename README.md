# Relineate

An SVG renderer for reMarkable .rm v5 files

## Building

```sh
cargo build
```

## Usage

```sh
relineate 0.1.0
Dan Shick <dan.shick@gmail.com>
Render .rm v5 files as SVGs

USAGE:
    relineate [FLAGS] [OPTIONS] --input <INPUT>

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity
    -V, --version    Prints version information

OPTIONS:
    -i, --input <INPUT>      Specifies an .rm v5 input file
    -o, --output <OUTPUT>    Specifies an SVG output file
```

## Todo

- [ ] Render different _______ differently
  - [ ] Brushes/Pens/Tools
  - [ ] Colors
  - [ ] Line widths
  - [ ] Stroke speeds/pressures/angles
- [ ] Modularize and tidy up the code
- [ ] Add template support
- [ ] Create scripts (or TUI app?) for convenient document retrieval
- [ ] Build for ARM for use on-device (with a GUI?)
- [ ] Support v3 .rm documents (maybe???)
