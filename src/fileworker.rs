use crate::ispyb::{
    populate_test_data, fetch_inspection_info, fetch_visit_info, populate_test_data_for_inspection
};

use std::{collections::HashSet, path::PathBuf};
use formulatrix_uploader::{Config, VisitInfo, XmlDatum};
use std::collections::HashMap;
use glob::glob;
use anyhow::{Context, Error, Ok, Result};
use mysql::*;
use anyhow::anyhow;
use std::path::Path;
use std::result::Result::Ok as OtherOk;
use std::fs;
use std::io::prelude::*;
use std::process::Command;
use image::{open, DynamicImage, imageops};
use rayon::prelude::*;
use elementtree::{self, Element};

pub trait WorkerShared {
    fn process_job(&self, pool: &Pool) -> Result<(),Error>;

    fn get_visit_dir(&self, query_result: VisitInfo, upload_dir: String) -> Result<PathBuf,Error>{    
        let visit = query_result.visit.unwrap();
        let proposal = if let Some(index) = visit.find('-') {
            visit[..index].to_string()
        } else {
            visit.clone()
        };
    
        let new_root: PathBuf = Path::new(&upload_dir)
            .join(Path::new(&proposal))
            .join(Path::new(&visit));
                        
        let old_root: PathBuf = if let Some(year) = query_result.year {
            Path::new(&upload_dir)
            .join(Path::new(&year))
            .join(Path::new(&visit))
        } else{
            PathBuf::new()
        };
        
        match fs::canonicalize(old_root.clone()) {
            OtherOk(path) => Ok(path),
            Err(_) => {
                match fs::canonicalize(new_root.clone()) {
                    OtherOk(path) => Ok(path),
                    Err(_) => Err(anyhow!(format!(
                        "Visit directory path does not exist. Tried old root: {} and new root: {}",
                        old_root.to_string_lossy().into_owned(),
                        new_root.to_string_lossy().into_owned()
                    ))),
                }
            }
        }
    }

    fn make_dirs(&self, path: &Path, web_user: String) -> Result<(), Error>{
        if path.exists() {
            Ok(())
        } else {
            fs::create_dir_all(path).map_err(anyhow::Error::from)?;
        
            match Command::new("/usr/bin/setfacl")
            .args(["-R", "-m", &format!("u:{}:rwx", web_user), path.to_string_lossy().as_ref()])
            .status() {
                OtherOk(setfacl_status) => {
                    if setfacl_status.success() {
                        Ok(())
                    } else {
                        println!("setfacl failed with exit code: {}", setfacl_status.code().unwrap_or(-1));
                        Ok(())
                    }
                }
                Err(e) => {
                    println!("setfacl process failed with: {}", e);
                    Ok(())
                }
            }
        }
    }

    fn move_dir(&self, src: &Path, target: &Path) -> Result<(),Error>{
        let new_filename = target.join(src.file_name().unwrap());

        let has_tiff_extension = matches!(src.extension(), Some(ext) if ext == "tiff");

        if has_tiff_extension {
            let img: DynamicImage = open(src)?;
            let flipped_image = imageops::flip_vertical(&img);
            flipped_image.save(&new_filename)?;
            //fs::remove_file(src).context("Failed to delete file from source")?;
        } else {
            fs::copy(src, &new_filename).context("Failed to copy file")?;
            //fs::remove_file(src).context("Failed to delete file from source")?;
        }

        Ok(())
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

            let barcode_dir: Vec<PathBuf> = barcodes
                .filter_map(Result::ok)
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

    pub fn get_target_and_move(&self, barcode: &String, date_dir: &String, pool: &Pool, holding_dir: String)  -> Result<Vec<Result<PathBuf, Error>>, Error> {
        //for testing//
        populate_test_data(barcode, pool)?;
        //for testing//
        let query_result= fetch_visit_info(barcode, pool).context("Failed to retrieve container info from bracode")?;
        if let None = query_result.clone() {
            return Err(anyhow!(format!("No container info found for barcode {}", barcode)))
        }
        
        if let None = query_result.clone().unwrap().visit {
            return Err(anyhow!(format!("No visit directory found for barcode {}", barcode)))
        }
        
        let visit_dir: PathBuf=  self.get_visit_dir(
            query_result
            .clone()
            .unwrap(), 
            self.config.upload_dir
            .clone())
            .context(format!("Could not obtain visit directory for barcode: {}", barcode))?;
                        
        let target_dir: PathBuf = visit_dir.join("tmp").join(barcode);

        self.make_dirs(&target_dir, self.config.web_user.clone()).context("Failed to create target directory")?;

        let src_dir = Path::new(&holding_dir).join(date_dir).join(barcode);
        
        let files: Vec<PathBuf> = glob(src_dir.join("*").to_string_lossy().as_ref())
        .context(format!("Failed to glob source directory for barcode: {}", barcode))?
        .filter_map(Result::ok)
        .collect();
    
        Ok(files
        .par_iter()
        .map(|file| {
            match self.move_dir(file, &target_dir) {
                OtherOk(_) => Ok(file.clone()),
                Err(err) => {
                    println!("Failed to move file {:?}: {}", file, err);
                    Err(err)
                }
            }
        })
        .collect())
    }
}

impl WorkerShared for ZWorker {
    fn process_job(&self, pool: &Pool) -> Result<(),Error>{
        println!("Processing job for Z task");
        let container_dict: HashMap<String, String> = self.get_container_dict(self.date_dirs.clone())?;

        container_dict.par_iter().for_each(|(barcode, date_dir)| {
            let result = self.get_target_and_move(barcode, date_dir, pool, self.config.holding_dir.clone());
            match result {
                OtherOk(files) => {
                    println!("This barcode has finished processing: {}", barcode);
                    for file in files {
                        if let Err(err) = file {
                            println!("Failed to process file: {}", err);
                        }
                    }
                },
                Err(err) => {
                    println!("Failed to process barcode: {}", barcode);
                    println!("{:?}", err);
                }
            }
        });

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
    
    pub fn handle_ef(&self, xml_datum: &XmlDatum, pool: &Pool) -> Result<(), Error>{
        println!("Handling EF files");
        //for testing//
        populate_test_data_for_inspection(&xml_datum.inspection_id, pool)?;
        //for testing//
        let container: std::result::Result<Option<formulatrix_uploader::InspectionInfo>, mysql::Error> = fetch_inspection_info(&xml_datum.inspection_id, pool);
        println!("{:?}", container);

        Ok(())
    }

    pub fn check_pairs_collect_xml(&self) -> Vec<&PathBuf> {
        let jpg_file_stems: HashSet<String> = self.files.iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("jpg"))
        .filter_map(|path| path.file_stem().and_then(|stem| stem.to_str()).map(|s| s.to_string()))
        .collect();

        let xml_files: Vec<&PathBuf> = self.files.iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("xml"))
        .collect();

        xml_files
        .into_iter()
        .filter(|path| {
            let should_include = path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(|s| jpg_file_stems.contains(s))
                .unwrap_or(false);

            if !should_include {
                println!("XML file has no corresponding JPEG: {:?}", path);
            }
            should_include
        })
        .collect()
    }

    pub fn get_inspection_id(&self, xml_file: &Path) -> Result<(String, String, Element), Error>{
        let path = xml_file;
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let root = Element::from_reader(contents.as_bytes())?;
        let ns = root.tag().ns().context("No namespace found")?;
        let nss = format!("{{{}}}ImagingId", ns);
        let imaging_id = root.find(nss.as_str()).map(|f| f.text()).context("Imaging ID not found in XML")?;
        let inspection_id = imaging_id.split("-").next().map(|sub| sub.to_string()).context("Failed to parse inspection ID from imaging ID")?;
        Ok((inspection_id, nss, root))
    }

}

impl WorkerShared for EFWorker {
    fn process_job(&self, pool: &Pool) -> Result<(),Error> {
        println!("Processing job for Z task");

        let xml_files: Vec<&PathBuf> = self.check_pairs_collect_xml();

        let mut xml_data: Vec<XmlDatum> = xml_files.into_iter()
        .filter_map(|xml_file| {
            match self.get_inspection_id(xml_file) {
                OtherOk((inspection_id, nss, root)) => {
                    let mut xml_datum = XmlDatum{
                        xml: xml_file.to_string_lossy().into_owned(), 
                        inspection_id, 
                        root, 
                        nss, 
                        container: None};
                    Some(xml_datum)
                }
                Err(e) => {
                    println!("{:?}", e);
                    None
                }
            }
        })
        .collect();

        let res = self.handle_ef(&xml_data[1    ], pool);

        Ok(())
    }
}