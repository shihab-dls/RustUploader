use std::path::PathBuf;
use formulatrix_uploader::Config;

pub trait WorkerShared {
    fn print_paths(&self, paths: &Vec<PathBuf>) {
        println!("Processing Paths: {:?}", paths);
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

    pub fn z_specific_function(&self) {
        self.print_paths(&self.date_dirs);
    }
}

impl WorkerShared for ZWorker {}

pub struct EFWorker {
    pub config: Config,
    pub files: Vec<PathBuf>,
}

impl EFWorker {
    pub fn new(config: Config, files: Vec<PathBuf>) -> Self {
        Self { config, files }
    }

    pub fn ef_specific_function(&self) {
        self.print_paths(&self.files);
    }
}

impl WorkerShared for EFWorker {}