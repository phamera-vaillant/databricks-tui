use crate::cli::DatabricksCli;
use crate::shape::{ListItem, Shape, Status};
use anyhow::Result;

pub async fn fetch(cli: &DatabricksCli) -> Result<Shape> {
    let json = cli.run(&["pipelines", "list-pipelines"]).await?;
    let items = json
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|p| ListItem {
                    name: p["name"].as_str().unwrap_or("unknown").to_string(),
                    status: Status::from_str(p["state"].as_str().unwrap_or("")),
                    detail: p["pipeline_id"].as_str().map(str::to_string),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Shape::List(items))
}
