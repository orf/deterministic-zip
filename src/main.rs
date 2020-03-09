use std::fs::File;
use std::io::{Read, Seek, Write};
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
    }
}

impl Into<CompressionMethod> for Compression {
    fn into(self) -> CompressionMethod {
        match self {
            Compression::None => CompressionMethod::Stored,
            Compression::Deflate => CompressionMethod::Deflated,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
    #[structopt(parse(from_os_str))]
    output: PathBuf,

    #[structopt(short, long,
    possible_values = & Compression::variants(), case_insensitive = true,
    default_value = "Deflate")]
    compression: Compression,

    #[structopt(short, long)]
    quiet: bool,

    #[structopt(parse(from_os_str), required(true))]
    paths: Vec<PathBuf>,
}

#[paw::main]
fn main(args: Opt) -> Result<(), std::io::Error> {
    let paths: Vec<(PathBuf, PathBuf)> = args
        .paths
        .into_iter()
        .flat_map(handle_path)
        .map(|p| (p.clone(), p))
        .collect();
    let output_file = File::create(args.output)?;
    create_zip_file(output_file, paths, args.compression.into(), args.quiet)?;
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

fn create_zip_file<W>(
    output_file: W,
    mut paths: Vec<(PathBuf, PathBuf)>,
    compression: CompressionMethod,
    quiet: bool,
) -> Result<(), std::io::Error>
where
    W: Write + Seek,
{
    paths.sort();
    let options = FileOptions::default()
        .last_modified_time(DateTime::default())
        .compression_method(compression);
    let mut zip_writer = ZipWriter::new(output_file);

    let mut buffer = Vec::new();

    for (name, path) in paths {
        if !quiet {
            println!("{}", path.display());
        }
        if path.is_dir() {
            if path.as_os_str().is_empty() {
                continue;
            }
            zip_writer.add_directory_from_path(name.as_path(), options)?;
        } else {
            zip_writer.start_file_from_path(name.as_path(), options)?;
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
    use super::*;
    use sha2::Digest;
    use std::fs;
    use std::io::Cursor;
    use tempfile;

    #[test]
    fn test_consistency() {
        let temp_dir = tempfile::tempdir().expect("Failed to make tempdir");
        let files = vec!["test1", "test2", "test3"];
        for f in &files {
            fs::write(temp_dir.path().join(f), format!("file {}", f))
                .expect("Could not write test file");
        }
        let mut buffer = Vec::new();
        let zip_file = Cursor::new(&mut buffer);
        let files_with_paths = files
            .iter()
            .map(|f| (PathBuf::from(f), temp_dir.path().join(f)))
            .collect();
        create_zip_file(
            zip_file,
            files_with_paths,
            CompressionMethod::Deflated,
            true,
        )
        .expect("Error running create_zip_file");

        let result = sha2::Sha256::digest(&buffer);
        assert_eq!(
            format!("{:x}", result),
            "810a8f84ba4ab8300152e6c07f250c717fc646ba0f391ee8c148747f212f99b9"
        );
    }

    #[test]
    fn test_handle_path_with_dir() {
        let temp_dir = tempfile::tempdir().expect("Failed to make tempdir");
        let root_file = temp_dir.path().join("test.txt");
        let nested_directory = temp_dir.path().join("inner");
        let nested_file = nested_directory.join("inner.txt");

        File::create(&root_file).expect("Failed to create root_file");
        std::fs::create_dir(&nested_directory).expect("Failed to create nested directory");
        File::create(&nested_file).expect("Failed to create nested_file");

        let root_path = temp_dir.into_path();

        let mut results = handle_path(root_path.clone());
        results.sort();

        assert_eq!(
            results,
            vec![root_path, nested_directory, nested_file, root_file]
        );
    }

    #[test]
    fn test_handle_path_with_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to make tempdir");
        let temp_file_path = temp_dir.path().join("test.txt");
        File::create(&temp_file_path).expect("Failed to create temporary file");
        assert_eq!(handle_path(temp_file_path.clone()), vec![temp_file_path]);
    }
}
