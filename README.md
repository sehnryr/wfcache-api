# wfcache-api

`wfcache-api` is a Rust application that allows you to read and extract data 
from Warframe's cache files programmatically.

## API support

- [x] list files and directories
- [x] move around directories
- [ ] extract a specific file (see [supported formats](#supported-formats) below)
- [ ] extract a directory and all its files and subdirectories recursively

## Shell ergonomics

- [ ] auto-completion

## Environment variables

- `RUST_LOG`: set the log level (e.g. `RUST_LOG=debug`)

## Supported formats

- [ ] `*.png` Images (Exported as DDS)
- [ ] `*.fbx` 3D models
