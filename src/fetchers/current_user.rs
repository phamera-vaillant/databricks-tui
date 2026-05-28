use crate::cli::DatabricksCli;
use crate::shape::{BadgeData, Shape};
use anyhow::Result;

pub async fn fetch(cli: &DatabricksCli) -> Result<Shape> {
    let json = cli.run(&["current-user", "me"]).await?;
    let value = json["userName"]
        .as_str()
        .or_else(|| json["displayName"].as_str())
        .unwrap_or("unknown")
        .to_string();
    Ok(Shape::Badge(BadgeData {
        label: "Logged in as".to_string(),
        value,
    }))
}
