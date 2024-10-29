use serde::Deserialize;
use diesel::QueryableByName;

/// The paths to cofiguration files
#[derive(Deserialize, Debug)]
pub struct ConfigPaths {
    /// Path for lists of handled EF files
    pub up_files_out_dir: String,
    /// Path for ISPyB credentials
    pub credentials_path: String,
    /// Path for EF handling configuration
    pub config_file_ef: String,
    /// Path for Z handling configuration
    pub config_file_z: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct PlateLayout {
    pub well_per_row: u8,
    pub drops_per_well: u8,
}

#[derive(Deserialize, Debug, Default)]
pub struct PlateTypes {
    pub CrystalQuickX: PlateLayout,
    pub MitegenInSitu: PlateLayout,
    pub MitegenInSitu_3_Drop: PlateLayout,
    pub FilmBatch: PlateLayout,
    pub ReferencePlate: PlateLayout,
}

#[derive(Deserialize, Debug)]
pub struct LoggingConfig {
    pub filename: String,
    pub max_bytes: u32,
    pub no_files: u32,
    pub format: String,
    pub level: String,
}

#[derive(Deserialize, Debug)]
pub struct Logging {
    rotating_file: LoggingConfig
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub upload_dir: String,
    pub holding_dir: String,
    pub task: String,
    pub max_files: u32,
    #[serde(default)]
    pub max_files_in_batch: u32,
    #[serde(default)]
    pub thumb_width: u32,
    #[serde(default)]
    pub thumb_height: u32,
    #[serde(default)]
    pub types: PlateTypes,
    pub logging: Logging,
}

#[derive(Deserialize, Debug)]
pub struct Credentials {
    pub database: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u32,
}

#[derive(QueryableByName, Debug)]
pub struct ContainerInfo {
    #[sql_type = "diesel::sql_types::Integer"] // Change to the appropriate SQL type
    pub container_id: i32,
    
    #[sql_type = "diesel::sql_types::Integer"] // Change to the appropriate SQL type
    pub session_id: i32,
    
    #[sql_type = "diesel::sql_types::Integer"] // Change to the appropriate SQL type
    pub dewar_id: i32,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub name: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub barcode: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub status: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub container_type: String,
    
    #[sql_type = "diesel::sql_types::Integer"] // Change to the appropriate SQL type
    pub capacity: i32,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub location: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub  beamline: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub comments: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub experiment_type: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub visit: String,
    
    #[sql_type = "diesel::sql_types::Text"] // Change to the appropriate SQL type
    pub year: String,
}

#[derive(Debug)]
pub struct Test {
    pub containerId: i32,
}