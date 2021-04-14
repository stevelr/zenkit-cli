use crate::{BackupOpt, Error};
use std::result::Result;
use tokio::fs;
use zenkit::types::{Entry, GetEntriesRequest, ID};

/// Backup a list in json to three files in the output directory, named
///     <uuid>_list.json, <uuid>_fields.json, and <uuid>_items.json
// The data written is not exactly what was received from the server:
//   It's been unserialized and then re-serialized. If there are
//   missing fields in the (Element or List) struct definitions,
//   they may be omitted from the json. (Entry has a catch-all 'fields' field,
//   so it's not likely to be suceptible to this risk). Also, from manual review,
//   it appears that all business (user-defined) fields are included in the definitions.
pub(crate) async fn backup_list<'ws>(
    ws_id: ID,
    list_id: &str,
    opt: &BackupOpt,
    //output_dir: &str,
) -> Result<BackupItem, Error> {
    let api = zenkit::get_api()?;
    let list_info = api.get_list_info(ws_id, list_id).await.map_err(|e| {
        crate::Error::Message(format!("Error loading list {}: {}", list_id, e.to_string()))
    })?;

    let list_fname = format!("{}/{}_list.json", &opt.output, list_info.list().uuid);
    let list_data = serde_json::to_string(list_info.list())?;
    fs::write(list_fname, &list_data).await?;

    let fields_fname = format!("{}/{}_fields.json", &opt.output, &list_info.list().uuid);
    let fields_data = serde_json::to_string(list_info.fields())?;
    fs::write(fields_fname, &fields_data).await?;

    let items_fname = format!("{}/{}_items.json", &opt.output, &list_info.list().uuid);
    let mut all_items: Vec<Entry> = Vec::new();
    let max_items = 500usize; // items per iteeration
    let mut start_index = 0usize;
    loop {
        // get the items and build the index
        let mut batch_items: Vec<Entry> = api
            .get_list_entries(
                list_id,
                &GetEntriesRequest {
                    limit: max_items,
                    skip: start_index,
                    allow_deprecated: opt.include_archived,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| {
                eprintln!(
                    "Error getting items from list {} (start={})",
                    list_id, start_index
                );
                e
            })?;
        if batch_items.is_empty() {
            break;
        }
        start_index += batch_items.len();
        all_items.append(&mut batch_items);
    }
    let items_data = serde_json::to_string(&all_items)?;
    fs::write(items_fname, &items_data).await?;
    Ok(BackupItem {
        name: list_info.list().name.clone(),
        uuid: list_info.list().uuid.clone(),
    })
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct BackupItem {
    name: String,
    uuid: String,
}
