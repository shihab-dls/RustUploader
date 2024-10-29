mod fileworker;

use crate::fileworker::{EFWorker, ZWorker, WorkerShared};
use formulatrix_uploader::{ConfigPaths, Config, Credentials, ContainerInfo, Test};
use prelude::Queryable;
use serde_json;
use anyhow::{Context, Result, Error};
use std::{fmt::format, fs::File};
use std::io::Read;
use std::path::PathBuf;
use glob::glob;
use log::{error, info};
use serde::de::DeserializeOwned;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::sql_query;
use mysql::*;
use mysql::prelude::*;

fn main() {
    dotenvy::dotenv().ok();
    let config_paths: ConfigPaths = envy::from_env::<ConfigPaths>()
    .expect("Failed to load configuration data from .env file");

    let worker_ef: Box<dyn WorkerShared> = setup_worker(&config_paths.config_file_ef).expect("Could not setup EF worker");
    let worker_z: Box<dyn WorkerShared> = setup_worker(&config_paths.config_file_z).expect("Could not setup Z worker");

    let database_url: String = parse_ispyb_url(&config_paths.credentials_path).expect("Failed to parse ISPyB URL");

    let pool = Pool::new(database_url.as_str()).expect("Failed to establish connection pool");

    let mut conn = pool.get_conn().expect("Failed to establish connection to database");

    let selected = conn.query_map("SELECT containerId from Container",|(containerId)| Test {containerId}).expect("Query failed.");

    let selected = conn.query_map("CALL retrieve_container_for_barcode('VMXiSim-001');",|(container_id, session_id, dewar_id, name, barcode, status, container_type, capacity, location, beamline, comments, experiment_type, visit, year)| ContainerInfo {container_id, session_id, dewar_id, name, barcode, status, container_type, capacity, location, beamline, comments, experiment_type, visit, year},).expect("Query failed.");

    println!("{:?}", selected);

    worker_z.process_job();

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

fn load_data_from_json(file_path: &String) -> Result<Config> {
    load_from_json(file_path)
}

fn load_creds_from_json(file_path: &String) -> Result<Credentials> {
    load_from_json(file_path)
}

fn parse_ispyb_url(file_path: &String) -> Result<String,Error>{
    let database_creds:Credentials = load_creds_from_json(&file_path)?;

    let database_url: String = format!("mysql://{}:{}@{}:{}/{}?pool_min=1&pool_max=1", database_creds.username, database_creds.password, database_creds.host, database_creds.port, database_creds.database);

    Ok(database_url)
}

fn glob_files(config: Result<Config, Error>) -> Result<(glob::Paths, Config), Error> {
    let config = config?;
    let files: glob::Paths = glob(&format!("{}/{}", &config.holding_dir, "*"))?;
    Ok((files, config))
}

pub fn establish_connection(database_url: &String) -> MysqlConnection {
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}