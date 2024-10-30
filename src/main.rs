mod fileworker;
mod ispyb;

use crate::ispyb::{create_conn_pool, parse_ispyb_url, fetch_visit_info};
use crate::fileworker::{EFWorker, ZWorker, WorkerShared};

use diesel::result;
use formulatrix_uploader::{ConfigPaths, Config, Credentials};
use serde_json;
use anyhow::{Context, Result, Error};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use glob::glob;
use log::{error, info};
use serde::de::DeserializeOwned;
use mysql::*;
use mysql::prelude::*;

fn main() -> Result<(),Error> {
    dotenvy::dotenv().ok();
    let config_paths: ConfigPaths = envy::from_env::<ConfigPaths>()
    .context("Failed to load configuration data from .env file")?;

    let database_url: String = parse_ispyb_url(&config_paths.credentials_path).context("Failed to parse ISPyB URL")?;
    let pool: Pool = create_conn_pool(database_url).context("Failed to establish connection pool")?;
    
    let worker_ef: Box<dyn WorkerShared> = setup_worker(&config_paths.config_file_ef).context("Could not setup EF worker")?;
    let worker_z: Box<dyn WorkerShared> = setup_worker(&config_paths.config_file_z).context("Could not setup Z worker")?;

    let res = worker_z.process_job(&pool);
    
    Ok(())
}

fn setup_worker(config_path: &String) -> Result<Box<dyn WorkerShared>, Error> {
    let config: std::result::Result<Config, Error> = load_data_from_json(config_path);
    match glob_files(config) {

        Ok((paths, config)) => {
            info!("Found files: {:?}", paths);
            let path_vector: Vec<PathBuf> = paths.filter_map(|entry: std::result::Result<PathBuf, glob::GlobError>| {
                match entry {
                    Ok(path) => Some(path),
                    Err(_) => None,
                }
            })
            .collect();

            match config.task.as_str() {
                "Z" => {
                    let worker = ZWorker::new(config, path_vector);
                    Ok(Box::new(worker))
                }
                "EF" => {
                    let worker = EFWorker::new(config, path_vector);
                    Ok(Box::new(worker))
                }
                _ => {
                    error!("Unknown task type in config file: {}", config.task);
                    Err(anyhow::Error::msg("Unknown task type"))
                }
            }
        }

        Err(err) => {
            error!("Error processing config: {:?}", err);
            Err(err)
        }
    }
}

fn load_from_json<T: DeserializeOwned>(file_path: &String) -> Result<T> {
    let mut file: File = File::open(file_path)
        .with_context(|| format!("Failed to open config file: {}", file_path))?;

    let mut content: String = String::new();
    file.read_to_string(&mut content)
        .with_context(|| "Failed to read file content")?;

    if content.is_empty() {
        anyhow::bail!("File content is empty");
    }

    let parsed_content: T = serde_json::from_str(&content)
        .with_context(|| "Failed to parse JSON")?;

    Ok(parsed_content)
}

pub fn load_data_from_json(file_path: &String) -> Result<Config> {
    load_from_json(file_path)
}

pub fn load_creds_from_json(file_path: &String) -> Result<Credentials> {
    load_from_json(file_path)
}

fn glob_files(config: Result<Config, Error>) -> Result<(glob::Paths, Config), Error> {
    let config = config?;
    let files: glob::Paths = glob(&format!("{}/{}", &config.holding_dir, "*"))?;
    Ok((files, config))
}