use formulatrix_uploader::{Credentials, VisitInfo, InspectionInfo};
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

pub fn fetch_inspection_info(inspection_id: &String, pool: &Pool) -> Result<Option<InspectionInfo>, mysql::Error> {
    let mut conn = pool.get_conn()?;
    let query = r#"
        SELECT 
            c.containerType, 
            c.containerId, 
            c.sessionId, 
            CONCAT(p.proposalCode, p.proposalNumber, "-", bs.visit_number) AS visit,
            DATE_FORMAT(c.blTimeStamp, "%Y") AS year 
        FROM Container c
        INNER JOIN ContainerInspection ci ON ci.containerId = c.containerId
        INNER JOIN Dewar d ON d.dewarId = c.dewarId
        INNER JOIN Shipping s ON s.shippingId = d.shippingId
        INNER JOIN Proposal p ON p.proposalId = s.proposalId
        LEFT OUTER JOIN BLSession bs ON bs.sessionId = c.sessionId
        WHERE ci.containerinspectionId = ?
        LIMIT 1;
    "#;

    let result = conn.exec_first(
        query,
        (inspection_id,)
    )?.map(|(container_type, container_id, session_id, visit, year)| 
    InspectionInfo { 
        container_type, 
        container_id, 
        session_id, 
        visit, 
        year });

    Ok(result)
}

pub fn parse_ispyb_url(file_path: &String) -> Result<String,Error>{
    let database_creds:Credentials = load_creds_from_json(&file_path)?;

    let database_url: String = format!("mysql://{}:{}@{}:{}/{}?pool_min=1&pool_max=1", database_creds.username, database_creds.password, database_creds.host, database_creds.port, database_creds.database);

    Ok(database_url)
}

//for testing//
pub fn populate_test_data(barcode: &String, pool: &Pool) -> Result<(), mysql::Error> {
    let mut conn = pool.get_conn()?;
    
    let proposal_id = 1;
    let person_id = 1;
    let session_id = 2;
    let visit_number = "2";
    let proposal_code = "ABC";
    let proposal_number = "123";
    let bl_timestamp = "2023-06-20 15:45:00";

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO Proposal (proposalId, proposalCode, proposalNumber, personId)
        VALUES (?, ?, ?, ?)
        "#,
        (proposal_id, proposal_code, proposal_number, person_id),
    )?;

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO BLSession (sessionId, proposalId, visit_number)
        VALUES (?, ?, ?)
        "#,
        (session_id, proposal_id, visit_number),
    )?;

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO Container (barcode, sessionId, blTimeStamp)
        VALUES (?, ?, ?)
        "#,
        (barcode, session_id, bl_timestamp),
    )?;

    Ok(())
}
//for testing//
pub fn populate_test_data_for_inspection(inspection_id: &String, pool: &Pool) -> Result<(), mysql::Error> {
    let mut conn = pool.get_conn()?;

    let barcode = "ABC";
    let inspection_type_id = 1;
    let proposal_id = 1;
    let person_id = 1;
    let session_id = 2;
    let visit_number = "2";
    let proposal_code = "ABC";
    let proposal_number = "123";
    let shipping_id = 1;
    let dewar_id = 1;
    let container_id = 1;
    let container_type = "TypeA";
    let bl_timestamp = "2023-06-20 15:45:00";

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO Proposal (proposalId, proposalCode, proposalNumber, personId)
        VALUES (?, ?, ?, ?)
        "#,
        (proposal_id, proposal_code, proposal_number, person_id),
    )?;

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO Shipping (shippingId, proposalId)
        VALUES (?, ?)
        "#,
        (shipping_id, proposal_id),
    )?;

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO Dewar (dewarId, shippingId)
        VALUES (?, ?)
        "#,
        (dewar_id, shipping_id),
    )?;

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO BLSession (sessionId, proposalId, visit_number)
        VALUES (?, ?, ?)
        "#,
        (session_id, proposal_id, visit_number),
    )?;

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO Container (containerId, dewarId, containerType, sessionId, blTimeStamp, barcode)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        (container_id, dewar_id, container_type, session_id, bl_timestamp, barcode),
    )?;

    conn.exec_drop(
        r#"
        INSERT IGNORE INTO ContainerInspection (containerinspectionId, containerId, inspectionTypeId)
        VALUES (?, ?, ?)
        "#,
        (inspection_id, container_id, inspection_type_id),
    )?;

    Ok(())
}
//for testing//