# wfcache-api

`wfcache-api` is a Rust application that allows you to read and extract data 
from Warframe's cache files programmatically.

![Demo](docs/demo.gif)

## Dependencies

This binary depends on [Oodle](http://www.radgametools.com/oodle.htm) for
decompression. You will need to install the shared library, to do so follow the
instructions here: https://github.com/sehnryr/get-oodle-lib

## API support

- [x] list files and directories
- [x] move around directories
- [x] extract a specific file (see [supported formats](#supported-formats) below)
- [x] extract a directory and all its files and subdirectories recursively
- [x] find files or directories by name (case-sensitive)

## Shell ergonomics

- [x] path completion
- [x] run commands from outside the shell (e.g. `wfcache-api -c "ls /Lotus" [arg ...]`)

## Environment variables

- `RUST_LOG`: set the log level (e.g. `RUST_LOG=wfcache_api=debug`)

## Supported formats

- [x] `*.png` Images (Exported as DDS)
- [x] `*.wav` Audio (Exported as either WAV or Opus)
- [ ] `*.fbx` 3D models
