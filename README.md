# ctmp_numis
ctmpnumis.fr updater

## Description
`ctmp_numis` is a small utility program living in a menu bar which polls [ctmpnumis.fr](https://www.ctmpnumis.fr/en/) every few seconds and displays a system notification if a new listing is added onto the website.

This project will only work on MacOS due to OS-specific libraries.

## Running the project
Assuming you have Rust installed, just type:

```bash
cargo run --release
```

## Additional info
The project was tested using Rust 1.50.0 (Stable) on macOS 11.1 Big Sur.
