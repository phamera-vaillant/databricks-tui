use crate::cli::DatabricksCli;
use crate::shape::{ListItem, Shape};
use anyhow::Result;

pub async fn fetch(cli: &DatabricksCli) -> Result<Shape> {
    let json = cli.run(&["clusters", "list"]).await?;
    let items = json
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|c| ListItem {
                    name: c["cluster_name"].as_str().unwrap_or("unknown").to_string(),
                    status: c["state"].as_str().unwrap_or("").parse().unwrap(),
                    detail: c["cluster_id"].as_str().map(str::to_string),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Shape::List(items))
}
