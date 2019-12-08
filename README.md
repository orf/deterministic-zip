# Deterministic zip 🗄
[![Crates.io](https://img.shields.io/crates/v/deterministic-zip.svg)](https://crates.io/crates/deterministic-zip)
[![Actions Status](https://github.com/orf/deterministic-zip/workflows/CI/badge.svg)](https://github.com/orf/deterministic-zip/actions)

Regular zip utilities do not create deterministic zipfiles when given identical content. Things like the file 
modification times, directory iteration ordering and file permissions can change the hash of the zip file.

This is particularly annoying when using terraform with AWS Lambda. Without this tool every rebuild of the lambda 
source would result in a zip file with a different hash which would result in the lambda being re-deployed even if 
the source had not changed.

# Install 💿

## Homebrew (MacOS, Linux)

`brew tap orf/brew`, then `brew install git-workspace`

## Cargo

`cargo install deterministic-zip`

## Github releases

Download a prebuilt binary from [the releases page](https://github.com/orf/deterministic-zip/releases)

## Usage 📀

The command is run like so: `deterministic-zip [output-file] [files...]`

For example: `deterministic-zip output.zip my-source-code/`

You can customize the compression with `--compression`. 

## Alternatives 🌎

https://github.com/bboe/deterministic_zip and https://www.npmjs.com/package/deterministic-zip

