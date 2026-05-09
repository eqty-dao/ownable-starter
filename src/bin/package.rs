use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use serde::Serialize;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const WASM_FILENAME: &str = "ownable_bg.wasm";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageJson<'a> {
    name: &'a str,
    version: &'a str,
    description: &'a str,
    ownables_abi: &'a str,
    wire_format: &'a str,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let package_name = env!("CARGO_PKG_NAME");
    let crate_name = package_name.replace('-', "_");

    run_command(
        &cargo,
        &["build", "--release", "--target", WASM_TARGET],
        &repo_root,
    )?;
    run_command(&cargo, &["run", "--example", "schema"], &repo_root)?;

    let wasm_source = repo_root
        .join("target")
        .join(WASM_TARGET)
        .join("release")
        .join(format!("{crate_name}.wasm"));
    let pkg_dir = repo_root.join("pkg");
    let schema_dir = repo_root.join("schema");
    let assets_dir = repo_root.join("assets");
    let zip_path = repo_root.join(format!("{package_name}.zip"));
    let wasm_output = pkg_dir.join(WASM_FILENAME);
    let package_json_path = pkg_dir.join("package.json");

    if !wasm_source.exists() {
        return Err(format!("Missing wasm artifact: {}", wasm_source.display()));
    }

    fs::create_dir_all(&pkg_dir).map_err(io_error)?;
    fs::copy(&wasm_source, &wasm_output).map_err(io_error)?;
    fs::write(&package_json_path, package_json_bytes().map_err(io_error)?).map_err(io_error)?;

    create_zip(
        &zip_path,
        collect_asset_files(&assets_dir)?,
        &wasm_output,
        &package_json_path,
        collect_schema_files(&schema_dir)?,
    )
    .map_err(io_error)?;

    println!("Created {}", zip_path.display());
    Ok(())
}

fn run_command(cargo: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let status = Command::new(cargo)
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|err| format!("Failed to execute `{cargo} {}`: {err}", args.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Command `{cargo} {}` failed with status {}",
            args.join(" "),
            format_status(status)
        ))
    }
}

fn collect_asset_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    collect_files(dir).map_err(io_error)
}

fn collect_schema_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = collect_files(dir).map_err(io_error)?;
    files.retain(|path| path.extension().is_some_and(|ext| ext == "json"));
    Ok(files)
}

fn collect_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_files_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_files_recursive(&path, files)?;
        } else if entry.file_type()?.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn package_json_bytes() -> io::Result<Vec<u8>> {
    let package_json = PackageJson {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        description: env!("CARGO_PKG_DESCRIPTION"),
        ownables_abi: "1",
        wire_format: "cbor",
    };

    let mut bytes = serde_json::to_vec_pretty(&package_json)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn create_zip(
    zip_path: &Path,
    asset_files: Vec<PathBuf>,
    wasm_path: &Path,
    package_json_path: &Path,
    schema_files: Vec<PathBuf>,
) -> io::Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut names = BTreeSet::new();

    for path in asset_files {
        add_file_to_zip(&mut zip, &path, &mut names, options)?;
    }
    add_file_to_zip(&mut zip, wasm_path, &mut names, options)?;
    add_file_to_zip(&mut zip, package_json_path, &mut names, options)?;
    for path in schema_files {
        add_file_to_zip(&mut zip, &path, &mut names, options)?;
    }

    zip.finish()?;
    Ok(())
}

fn add_file_to_zip(
    zip: &mut ZipWriter<File>,
    path: &Path,
    names: &mut BTreeSet<String>,
    options: SimpleFileOptions,
) -> io::Result<()> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
        .to_string();

    if !names.insert(name.clone()) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Duplicate archive entry name: {name}"),
        ));
    }

    zip.start_file(name, options)?;
    let bytes = fs::read(path)?;
    zip.write_all(&bytes)?;
    Ok(())
}

fn format_status(status: ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "terminated by signal".to_string())
}

fn io_error(err: io::Error) -> String {
    err.to_string()
}
