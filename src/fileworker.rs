use crate::fetch_visit_info;

use std::path::PathBuf;
use formulatrix_uploader::{Config, VisitInfo};
use std::collections::HashMap;
use glob::glob;
use anyhow::{Context, Error, Ok, Result};
use std::ffi::OsStr;
use mysql::*;
use mysql::prelude::*;
use anyhow::anyhow;
use std::path::Path;
use std::result::Result::Ok as OtherOk;
use std::fs;
use std::process::Command;
use image::imageops;


pub trait WorkerShared {
    fn process_job(&self, pool: &Pool) -> Result<(),Error>;

    fn get_visit_dir(&self, query_result: VisitInfo, upload_dir: String) -> Result<String,Error>{
        let visit = query_result.visit.unwrap();

        let proposal = if let Some(index) = visit.find('-') {
            visit[..index].to_string()
        } else {
            visit.clone()
        };

        
        let new_root = format!("{}/{}/{}", upload_dir, proposal, visit);
        
        let old_root = if let Some(year) = query_result.year {
            format!("{}/{}/{}", upload_dir, year, visit)
        } else{
            String::new()
        };
        
        if Path::new(&old_root).exists(){
            return Ok(old_root)
        } else {
            if Path::new(&new_root).exists(){
                return Ok(new_root)
            } else {
                return Err(anyhow!("Visit directory path does not exist"))
            }
        }
    }

    fn make_dirs(&self, path: &Path, web_user: String) -> Result<(), Error>{
        if path.exists() {
            Ok(())
        } else {
            if let Err(e) = fs::create_dir_all(path) {
                Err(anyhow!(e))
            } else{
                let setfacl_status = Command::new("/usr/bin/setfacl")
                    .args(&["-R", "-m", &format!("u:{}:rwx", web_user), &path.to_string_lossy()])
                    .status();
    
                match setfacl_status {
                    OtherOk(status) if status.success() => {
                        Ok(())
                    }
                    OtherOk(status) => {
                        println!("setfacl failed with exit code: {}", status);
                        Ok(())
                    }
                    Err(e) => {
                        println!("Failed to execute setfacl: {}", e);
                        Ok(())
                    }
                }
            }        
        }
    }

    fn move_dir(&self, src: &PathBuf, target: &Path){
        let new_filename = target.join(src.file_name().unwrap());
        let ext = src.extension();
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
            let barcodes: glob::Paths = glob(
                &format!("{}/{}", &entry
                .into_os_string()
                .into_string()
                .expect("Cannot convert PathBuf to String"), "*/")
            )?;

            let barcode_dir: Vec<PathBuf> = barcodes.filter_map(|entry: std::result::Result<PathBuf, glob::GlobError>| {
                match entry {
                    OtherOk(path) => Some(path),
                    Err(_) => None,
                }
            })
            .collect();

            for entry in barcode_dir{
                containers.insert(
                    entry
                    .file_name()
                    .expect("Could not parse filename from barcode path")
                    .to_string_lossy()
                    .into_owned(), 
                    entry
                    .parent().
                    expect("Could not parse parent from barcode path")
                    .file_name()
                    .expect("Could not parse filename from barcode parent path")
                    .to_string_lossy()
                    .into_owned());
            }
        }
        Ok(containers)
    }

    pub fn get_target_and_move(&self, barcode: String, date_dir: String, pool: &Pool, holding_dir: String) -> Result<(),Error>{
        let query_result= fetch_visit_info(&barcode, pool).context("Failed to retrieve container info from bracode")?;

        if let None = query_result.clone() {
            return Err(anyhow!(format!("No container info found for barcode {}", &barcode)))
        }

        if let None = query_result.clone().unwrap().visit {
            return Err(anyhow!(format!("No visit directory found for barcode {}", &barcode)))
        }

        let visit_dir: String=  self.get_visit_dir(query_result.clone().unwrap(), self.config.upload_dir.clone()).context(format!("Could not obtain visit directory for barcode: {}", &barcode))?;

        let target_dir = format!("{}/{}/{}", &visit_dir, "tmp", &barcode);

        self.make_dirs(Path::new(&target_dir), self.config.web_user.clone()).context("Failed to create target directory")?;

        let src_dir = format!("{}/{}/{}", holding_dir, date_dir, barcode);
        
        let files: Vec<PathBuf> = glob(&format!("{}/*", &src_dir))
        .context(format!("Failed to glob source directory for: {}", src_dir))?
        .filter_map(|entry: std::result::Result<PathBuf, glob::GlobError>| {
            match entry {
                OtherOk(path) => Some(path),
                Err(_) => None,
            }
        })
        .collect();

        for file in files {
            self.move_dir(&file, Path::new(&target_dir));
        }
    
        Ok(())

    }
}

impl WorkerShared for ZWorker {
    fn process_job(&self, pool: &Pool) -> Result<(),Error>{
        println!("Processing job for Z task");
        let container_dict: HashMap<String, String> = self.get_container_dict(self.date_dirs.clone())?;

        for (barcode, date_dir)in container_dict {
            let res = self.get_target_and_move(barcode,date_dir, pool, self.config.holding_dir.clone());
        }

        Ok(())
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
    fn process_job(&self, pool: &Pool) -> Result<(),Error> {
        println!("Processing job for EF task");
        Ok(())
    }
}