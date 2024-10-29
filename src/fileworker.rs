use std::path::PathBuf;
use formulatrix_uploader::Config;
use std::collections::HashMap;
use glob::glob;
use anyhow::{Result, Error};
use std::ffi::OsStr;

pub trait WorkerShared {
    fn process_job(&self) -> Result<&str,Error>;

    fn retrieve_container_for_barcode(&self, barcode: String){
        println!("Accessing shared trait")
    }
}

pub struct ZWorker {
    pub config: Config,
    pub date_dirs: Vec<PathBuf>,
}

impl ZWorker {
    pub fn new(config: Config, date_dirs:Vec<PathBuf>) -> Self {
        Self { config, date_dirs }
    }

    pub fn get_container_dict(&self, date_dirs:Vec<PathBuf>) -> Result<HashMap<String, String>, Error>{
        let mut containers: HashMap<String, String> = HashMap::new();
        for entry in date_dirs{
            let barcodes: glob::Paths = glob(&format!("{}/{}", &entry.into_os_string().into_string().expect("Cannot convert PathBuf to String"), "*/"))?;

            let barcode_dir: Vec<PathBuf> = barcodes.filter_map(|entry: std::result::Result<PathBuf, glob::GlobError>| {
                match entry {
                    Ok(path) => Some(path),
                    Err(_) => None,
                }
            })
            .collect();

            for entry in barcode_dir{
                containers.insert(entry.file_name().expect("Could not parse filename from barcode path").to_string_lossy().into_owned(), entry.parent().expect("Could not parse parent from barcode path").file_name().expect("Could not parse filename from barcode parent path").to_string_lossy().into_owned());
            }
        }
        Ok(containers)
    }

    pub fn get_target_and_move(&self, barcode: String, date: String){
        self.retrieve_container_for_barcode(barcode);
    }
}

impl WorkerShared for ZWorker {
    fn process_job(&self) -> Result<&str,Error>{
        println!("Processing job for Z task");
        let container_dict: HashMap<String, String> = self.get_container_dict(self.date_dirs.clone())?;

        for (k, v)in container_dict {
            self.get_target_and_move(k,v);
        }




        let res = "test";
        Ok(&res)
    }
}

pub struct EFWorker {
    pub config: Config,
    pub files: Vec<PathBuf>,
}

impl EFWorker {
    pub fn new(config: Config, files: Vec<PathBuf>) -> Self {
        Self { config, files }
    }

    pub fn handle_ef(&self){
        println!("Handling EF files");
    }
}

impl WorkerShared for EFWorker {
    fn process_job(&self) -> Result<&str,Error> {
        println!("Processing job for EF task");
        let res = "test";
        Ok(&res)
    }
}