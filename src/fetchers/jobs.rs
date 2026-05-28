use crate::cli::DatabricksCli;
use crate::shape::{ListItem, Shape};
use anyhow::Result;

pub async fn fetch(cli: &DatabricksCli) -> Result<Shape> {
    let json = cli.run(&["jobs", "list"]).await?;
    let items = json
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|j| ListItem {
                    name: j["settings"]["name"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string(),
                    status: j["settings"]["format"]
                        .as_str()
                        .unwrap_or("")
                        .parse()
                        .unwrap(),
                    detail: j["job_id"].as_u64().map(|id| id.to_string()),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Shape::List(items))
}
