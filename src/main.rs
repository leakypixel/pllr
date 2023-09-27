use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path};
use std::process::{Command, exit};

#[derive(serde::Deserialize, Debug)]
struct Item {
    get: String,
    build: Option<String>,
    assets: Vec<String>,
    overwrite: Option<bool>,
    source: Option<String>,
    dest: Option<String>,
    children: Option<Vec<Item>>,
}

#[derive(Debug, serde::Deserialize)]
struct BuildConfig {
    items: Vec<Item>,
}

fn main() {
    // Get the command-line argument (directory path)
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <directory_path>", args[0]);
        exit(1);
    }
    let dir_path = &args[1];

    // Check the specified directory
    let dir = Path::new(dir_path);
    if !dir.is_dir() {
        eprintln!("Error: {} is not a directory.", dir.display());
        exit(1);
    }

    // Look for "pllr.json" in the working directory
    let pllr_json_path = dir.join("pllr.json");
    if !pllr_json_path.exists() {
        eprintln!("Error: pllr.json not found in the working directory.");
        exit(1);
    }

    // Parse the pllr.json file
    let mut pllr_json_content = String::new();
    if let Err(err) = File::open(&pllr_json_path).and_then(|mut file| file.read_to_string(&mut pllr_json_content)) {
        eprintln!("Error: Failed to read pllr.json: {}", err);
        exit(1);
    }

    let build_config: BuildConfig = match serde_json::from_str(&pllr_json_content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: Failed to parse pllr.json: {}", err);
            exit(1);
        }
    };


    // Recursively process the pllr.json configuration
    process_item(&build_config.items, &dir);
}

fn process_item(items: &[Item], base_dir: &Path) {
    for item in items {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let temp_dir_path = temp_dir.path();

        // Execute the "get" command in the temp directory
        let get_command = Command::new("sh")
            .arg("-c")
            .arg(&item.get)
            .current_dir(temp_dir_path)
            .output()
            .expect("Failed to execute 'get' command");

        if !get_command.status.success() {
            eprintln!("Error: 'get' command failed:\n{}", String::from_utf8_lossy(&get_command.stderr));
            exit(1);
        }

        // Set the specified source directory (if provided)
        let source_dir = match &item.source {
            Some(source) => temp_dir_path.join(source),
            None => temp_dir_path.to_owned(), // Use the base directory if no dest is specified
        };

        if let Some(build_command) = &item.build {
            // Execute the "build" command in the current directory
            let build_command_result = Command::new("sh")
                .arg("-c")
                .arg(build_command)
                .current_dir(source_dir.clone())
                .output();

            match build_command_result {
                Ok(build_command_output) => {
                    if !build_command_output.status.success() {
                        eprintln!("Error: 'build' command failed:\n{}", String::from_utf8_lossy(&build_command_output.stderr));
                        exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("Error: Failed to execute 'build' command: {}", err);
                    exit(1);
                }
            }
        }


        // Move assets to the specified dest directory
        let dest_dir = match &item.dest {
            Some(dest) => base_dir.join(dest),
            None => base_dir.to_owned(), // Use the base directory if no dest is specified
        };

        // Ensure the destination directory exists
        if !dest_dir.exists() {
            if let Err(err) = fs_extra::dir::create(&dest_dir, true) {
                eprintln!("Error: Failed to create directory {}: {}", dest_dir.display(), err);
                exit(1);
            }
        }

        for asset in &item.assets {
            let asset_path = source_dir.join(asset);
            let dest_path = dest_dir.join(Path::new(asset).file_name().unwrap());

            // Check if the destination file already exists
            let overwrite = item.overwrite.is_some();
            let file_exists = dest_path.exists();
            let options = &fs_extra::dir::CopyOptions {
                overwrite: overwrite,
                ..Default::default()
            };
            if !file_exists || overwrite {
                // Only copy if the file doesn't exist or "clean" is true
                if asset_path.is_dir() {
                    // If it's a directory, recursively copy its contents
                    if let Err(err) = fs_extra::dir::copy(&asset_path, &dest_dir, options) {
                        eprintln!("Error: Failed to copy directory {} to {}: {}", asset_path.display(), dest_dir.display(), err);
                        exit(1);
                    }
                } else if asset_path.is_file() {
                    // If it's a file, just copy it
                    if let Err(err) = fs::copy(&asset_path, &dest_path) {
                        eprintln!("Error: Failed to copy file {} to {}: {}", asset_path.display(), dest_path.display(), err);
                        exit(1);
                    }
                }
            } else {
                println!("Skipping copy of {} because the file already exists and overwrite is set to false.", asset);
            }
        }

        // Process children recursively
        if let Some(children) = &item.children {
            let child_base_dir = match &item.dest {
                Some(dest) => base_dir.join(dest),
                None => base_dir.to_owned(), // Use the base directory if no dest is specified
            };
            process_item(children, &child_base_dir);
        }
    }
}

