use formulatrix_uploader::{Credentials, VisitInfo};
use anyhow::{Context, Result, Error};
use mysql::*;
use mysql::prelude::*;
use crate::load_creds_from_json;

pub fn create_conn_pool(database_url: String) -> Result<Pool, mysql::Error>{
    let pool = Pool::new(database_url.as_str());
    return pool
}

pub fn fetch_visit_info(barcode: &String, pool: &Pool) -> Result<Option<VisitInfo>, mysql::Error> {
    let mut conn = pool.get_conn()?;
    
    let query = r#"
        SELECT 
            CONCAT(p.proposalCode, p.proposalNumber, "-", bs.visit_number) AS visit,
            DATE_FORMAT(c.blTimeStamp, "%Y") AS year 
        FROM Container c 
        LEFT OUTER JOIN BLSession bs ON bs.sessionId = c.sessionId 
        LEFT OUTER JOIN Proposal p ON p.proposalId = bs.proposalId 
        WHERE c.barcode = ? 
        LIMIT 1;
    "#;

    let result = conn.exec_first(
        query,
        (barcode,)
    )?.map(|(visit, year)| VisitInfo { visit, year });

    Ok(result)
}

pub fn parse_ispyb_url(file_path: &String) -> Result<String,Error>{
    let database_creds:Credentials = load_creds_from_json(&file_path)?;

    let database_url: String = format!("mysql://{}:{}@{}:{}/{}?pool_min=1&pool_max=1", database_creds.username, database_creds.password, database_creds.host, database_creds.port, database_creds.database);

    Ok(database_url)
}