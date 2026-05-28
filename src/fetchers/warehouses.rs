use crate::cli::DatabricksCli;
use crate::shape::{Shape, TableData};
use anyhow::Result;

pub async fn fetch(cli: &DatabricksCli) -> Result<Shape> {
    let json = cli.run(&["warehouses", "list"]).await?;
    let headers = vec![
        "Name".to_string(),
        "State".to_string(),
        "Size".to_string(),
        "ID".to_string(),
    ];
    let rows = json["warehouses"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|w| {
                    vec![
                        w["name"].as_str().unwrap_or("").to_string(),
                        w["state"].as_str().unwrap_or("").to_string(),
                        w["cluster_size"].as_str().unwrap_or("").to_string(),
                        w["id"].as_str().unwrap_or("").to_string(),
                    ]
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Shape::Table(TableData { headers, rows }))
}
