use crate::cli::DatabricksCli;
use crate::shape::{ListItem, Shape, Status};
use anyhow::Result;
use serde_json::Value;

fn items_of(json: &Value) -> &[Value] {
    json.as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn entry(v: &Value, kind: &str) -> ListItem {
    ListItem {
        name: v["name"].as_str().unwrap_or("unknown").to_string(),
        status: Status::Unknown(kind.to_string()),
        detail: v["comment"].as_str().map(str::to_string),
        id: v["full_name"]
            .as_str()
            .or_else(|| v["name"].as_str())
            .map(str::to_string),
        history: Vec::new(),
    }
}

/// Lists one level of the Unity Catalog tree:
/// no path → catalogs, [catalog] → schemas, [catalog, schema] → tables,
/// views and volumes.
pub async fn fetch(cli: &DatabricksCli, path: &[String]) -> Result<Shape> {
    let items = match path {
        [] => items_of(&cli.run(&["catalogs", "list"]).await?)
            .iter()
            .map(|c| entry(c, "CATALOG"))
            .collect(),
        [catalog] => items_of(&cli.run(&["schemas", "list", catalog]).await?)
            .iter()
            .map(|s| entry(s, "SCHEMA"))
            .collect(),
        [catalog, schema, ..] => {
            let table_args = ["tables", "list", catalog, schema];
            let volume_args = ["volumes", "list", catalog, schema];
            let (tables, volumes) = tokio::join!(cli.run(&table_args), cli.run(&volume_args));
            let mut items: Vec<ListItem> = items_of(tables.as_ref().unwrap_or(&Value::Null))
                .iter()
                .map(|t| {
                    let kind = match t["table_type"].as_str() {
                        Some("VIEW") | Some("MATERIALIZED_VIEW") => "VIEW",
                        _ => "TABLE",
                    };
                    entry(t, kind)
                })
                .collect();
            items.extend(
                items_of(volumes.as_ref().unwrap_or(&Value::Null))
                    .iter()
                    .map(|v| entry(v, "VOLUME")),
            );
            items
        }
    };
    Ok(Shape::List(items))
}
