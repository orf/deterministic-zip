use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};
use zip::{CompressionMethod, DateTime};

arg_enum! {
    #[derive(Debug)]
    enum Compression {
        None,
        Deflate,
        Bzip2,
    }
}

impl Into<CompressionMethod> for Compression {
    fn into(self) -> CompressionMethod {
        match self {
            Compression::None => CompressionMethod::Stored,
            Compression::Deflate => CompressionMethod::Deflated,
            Compression::Bzip2 => CompressionMethod::Bzip2,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    output: PathBuf,

    #[structopt(short, long,
    possible_values = & Compression::variants(), case_insensitive = true,
    default_value = "Bzip2")]
    compression: Compression,

    #[structopt(parse(from_os_str), required(true))]
    paths: Vec<PathBuf>,
}

#[paw::main]
fn main(args: Opt) -> Result<(), std::io::Error> {
    let paths: Vec<PathBuf> = args.paths.into_iter().flat_map(handle_path).collect();
    let output_file = File::create(args.output)?;
    create_zip_file(output_file, paths, args.compression.into())?;
    Ok(())
}

fn handle_path(path: PathBuf) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path]
    } else {
        WalkDir::new(&path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.into_path())
            .collect()
    }
}

fn create_zip_file(
    output_file: File,
    mut paths: Vec<PathBuf>,
    compression: CompressionMethod,
) -> Result<(), std::io::Error>
{
    paths.sort();
    let options = FileOptions::default()
        .last_modified_time(DateTime::default())
        .compression_method(compression);
    let mut zip_writer = ZipWriter::new(output_file);

    let mut buffer = Vec::new();

    for path in paths {
        println!("{}", path.display());
        if path.is_dir() {
            zip_writer.add_directory_from_path(path.as_path(), options)?;
        } else {
            zip_writer.start_file_from_path(path.as_path(), options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip_writer.write_all(&*buffer)?;
            buffer.clear();
        }
    }

    zip_writer.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

//    #[test]
//    fn test_add() {
//        assert_eq!(add(1, 2), 3);
//    }
//
//    #[test]
//    fn test_bad_add() {
//        // This assert would fire and test will fail.
//        // Please note, that private functions can be tested too!
//        assert_eq!(bad_add(1, 2), 3);
//    }
}
