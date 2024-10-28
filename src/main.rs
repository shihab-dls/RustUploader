use formulatrix_uploader::{ConfigPaths, Config};
use serde_json;
use anyhow::{Context, Result, Error};
use std::fs::File;
use std::io::Read;
use glob::glob;
use log::{error, info};

fn main() {
    dotenvy::dotenv().ok();
    let config_paths: ConfigPaths = envy::from_env::<ConfigPaths>()
        .expect("Failed to load configuration data from .env file");

    process_config(&config_paths.config_file_ef);
    process_config(&config_paths.config_file_z);
}

fn load_data_from_json(file_path: &String)-> Result<Config, Error>{
    let mut file: File = File::open(file_path)
        .with_context(|| format!("Failed to open config file: {}", file_path))?;

    let mut content: String = String::new();
    file.read_to_string(&mut content)   
        .with_context(|| "Failed to read file content")?;

    if content.is_empty() {
        anyhow::bail!("File content is empty");
    }

    let parsed_content: Config = serde_json::from_str(&content)
        .with_context(|| "Failed to parse JSON")?;

    Ok(parsed_content)
}

fn process_config(config_path: &String) {
    let config = load_data_from_json(config_path);
    match glob_files(config) {
        Ok(paths) => {
            info!("Found files: {:?}", paths);
        }
        Err(err) => {
            error!("Error processing config: {:?}", err);
        }
    }
}

fn glob_files(config: Result<Config, Error>) -> Result<glob::Paths, Error> {
    let config = config?;
    let files: glob::Paths = glob(&format!("{}/{}", &config.holding_dir, "*"))?;
    Ok(files)
}