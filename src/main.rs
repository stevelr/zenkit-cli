use clap::Clap;
use std::{fmt, fs, sync::Arc};
use zenkit::{
    self,
    types::{
        ElementCategoryId, FieldVal, NewWebhook, TextFormat, UpdateAction, WebhookTriggerType,
        Workspace, ID,
    },
    ApiConfig,
};

mod backup;
use backup::{backup_list, BackupItem};

const TOKEN_VAR: &str = "ZENKIT_API_TOKEN";
const WORKSPACE_VAR: &str = "ZENKIT_WORKSPACE";

#[derive(Debug)]
enum Error {
    Message(String),
    Zenkit(zenkit::Error),
    Io(String),
}

impl std::error::Error for Error {}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Error {
        Error::Message(e.to_string())
    }
}

impl From<zenkit::Error> for Error {
    fn from(e: zenkit::Error) -> Error {
        Error::Zenkit(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

fn parse_try_textfmt(s: &str) -> Result<TextFormat, &'static str> {
    if let Ok(tf) = s.parse() {
        Ok(tf)
    } else {
        Err("Invalid textformat: should be plain,markdown, or html")
    }
}

#[derive(Clap, PartialEq, Debug)]
struct SetValueOpt {
    /// List name or id
    #[clap(short, long)]
    list: String,

    /// Item id to modify
    #[clap(short, long)]
    item: ID,

    /// Field name or id
    #[clap(short, long)]
    field: String,

    /// Value (alternate to --file).
    /// If value is an item reference, it must be a uuid.
    /// If value is a person, it may be the person's uuid or display name (case-insensitive).
    /// If value is a choice, it may be the id or the display name (case-sensitive).
    #[clap(short, long, group = "file_or_value")]
    value: Option<String>,

    /// Text format (plain,markdown, or html). If unspecified, leave as-is.
    /// Only applicable for Text fields.
    #[clap(short, long, parse(try_from_str=parse_try_textfmt))]
    text: Option<TextFormat>,

    /// Read Value from file (alternate to -v). Only applicable for Text fields.
    #[clap(short = 'F', long, group = "file_or_value")]
    file: Option<String>,
}

#[derive(Clap, PartialEq, Debug)]
struct CommentOpt {
    /// List name or id
    #[clap(short, long)]
    list: String,

    /// Item id or uuid
    #[clap(short, long)]
    item: String,

    // User display-name (must be valid user)
    //#[clap(short, long)]
    //user: String,
    /// Comment
    #[clap(short, long)]
    comment: String,
}

#[derive(Clap, PartialEq, Debug)]
struct CreateOpt {
    /// List name or id
    #[clap(short, long)]
    list: String,

    /// -F field=value -F field=value ... Field names are case-sensitive.
    #[clap(short='F', parse(try_from_str=parse_key_val), number_of_values = 1)]
    fields: Vec<(String, String)>,
}

#[derive(Clap, PartialEq, Debug)]
struct BackupOpt {
    /// Output folder where json files will be created
    #[clap(short, long)]
    output: String,

    /// List - backup single list. If not specified, backs up all lists
    list: Option<String>,
}

#[derive(Clap, PartialEq, Debug)]
enum Sub {
    /// Show all workspaces and lists
    Workspaces,

    /// Show users in workspace
    Users,

    /// Show lists in workspace
    Lists,

    /// Show items in list
    #[clap(alias = "list")]
    Items(ListOpt),

    /// Show fields for a list
    Fields(ListOpt),

    /// Describe field of a list (detail view)
    Field(FieldOpt),

    /// Describe a list item (detail view)
    Item(ItemOpt),

    /// Show choices for a category field
    Choices(FieldOpt),

    /// Set field value
    Set(SetValueOpt),

    /// Create new list item
    Create(CreateOpt),

    /// Add comment to list item
    Comment(CommentOpt),

    /// Add a webhook
    #[clap(alias = "new-webhook")]
    Webhook(WebhookOpt),

    /// List webhooks
    ListWebhooks,

    /// Delete webhook
    DeleteWebhook(DelWebhookOpt),

    /// Backup
    Backup(BackupOpt),
}

#[derive(Clap, PartialEq, Debug)]
struct ListOpt {
    /// List name or id
    #[clap(short, long)]
    list: String,
}

#[derive(Clap, PartialEq, Debug)]
struct ItemOpt {
    /// List name or id
    #[clap(short, long)]
    list: String,

    /// Item id (integer) or uuid
    #[clap(short, long)]
    item: String,
}

#[derive(Clap, PartialEq, Debug)]
struct FieldOpt {
    /// List name or id
    #[clap(short, long)]
    list: String,

    /// Field id or name
    #[clap(short, long)]
    field: String,
}

#[derive(Clap, Debug, PartialEq)]
enum WebhookType {
    Item,
    Activity,
    Notification,
    System,
    Comment,
    Field,
}

#[derive(Clap, PartialEq, Debug)]
struct WebhookOpt {
    /// Webhook trigger type
    #[clap(short, long = "type", arg_enum)]
    trigger_type: WebhookType,

    /// Server url
    #[clap(short, long)]
    url: String,

    /// List id to restrict webhook to this list
    #[clap(short, long)]
    list: Option<String>,

    /// Item id to restrict webhook to this item
    #[clap(short, long)]
    item: Option<String>,

    /// Field id to restrict webhook to this field
    #[clap(short, long)]
    field: Option<String>,

    /// Locale
    #[clap(long, default_value = "en")]
    locale: String,
}

#[derive(Clap, PartialEq, Debug)]
struct DelWebhookOpt {
    /// Webhook id
    #[clap(short = 'W', long)]
    webhook: u64,
}

/// Zenkit command-line tool.
/// Source and docs at https://github.com/stevelr/zenkit-cli
#[derive(Clap, PartialEq, Debug)]
#[clap(name = env!("CARGO_BIN_NAME"), version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    /// URL for API endpoint
    #[clap(long, default_value = "https://zenkit.com/api/v1")]
    endpoint: String,

    /// API token. Defaults to environment var ZENKIT_API_TOKEN
    #[clap(short, long)]
    token: Option<String>,

    /// Workspace name, id, or uuid. Required unless set in environment var ZENKIT_WORKSPACE
    #[clap(short, long)]
    workspace: Option<String>,

    #[clap(subcommand)]
    cmd: Sub,
}

/// Parse "key=value" token
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn std::error::Error>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + 'static,
{
    let pos = s.find('=').ok_or_else(|| {
        Box::new(Error::Message(format!(
            "invalid KEY=value: no `=` found in `{}`",
            s
        )))
    })?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn parse_setval(s: String) -> FieldVal {
    if s.starts_with('[') && s.ends_with(']') {
        let value_list = &s[1..s.len() - 1];
        FieldVal::ArrStr(value_list.split(',').map(|v| v.to_string()).collect())
    } else if let Some(v) = s.strip_prefix("plain::") {
        FieldVal::Formatted(v.to_string(), TextFormat::Plain)
    } else if let Some(v) = s.strip_prefix("html::") {
        FieldVal::Formatted(v.to_string(), TextFormat::HTML)
    } else if let Some(v) = s.strip_prefix("markdown::") {
        FieldVal::Formatted(v.to_string(), TextFormat::Markdown)
    } else {
        FieldVal::Str(s)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let opt = Opt::parse();
    if let Err(e) = run(opt).await {
        eprintln!("Error: {:#?}", e);
        std::process::exit(1);
    }
}

async fn run(opt: Opt) -> Result<(), Error> {
    let token = match opt.token {
        Some(t) => t,
        None => std::env::var(&TOKEN_VAR)
            .map_err(|_| Error::Message(format!("Missing env var {}", TOKEN_VAR)))?,
    };
    let ws_name = match opt.cmd {
        // we only need to get workspace for some commands
        Sub::Workspaces | Sub::ListWebhooks | Sub::DeleteWebhook(_) => String::from(""),
        _ => match opt.workspace {
            Some(t) => t,
            None => std::env::var(&WORKSPACE_VAR).map_err(|_| {
                Error::Message(format!(
                    "Workspace undefined. Must be set with '-w' or in env var {}",
                    WORKSPACE_VAR
                ))
            })?,
        },
    };

    let api = zenkit::init_api(ApiConfig {
        endpoint: opt.endpoint,
        token,
    })?;

    match opt.cmd {
        Sub::Workspaces => {
            // list all workspaces and lists
            let workspaces: Vec<Arc<Workspace>> = api.get_all_workspaces_and_lists().await?;
            for ws in workspaces.iter() {
                println!("\nW\t{}\t{}\t{}", ws.id, ws.uuid, ws.name);
                for list in ws.lists.iter() {
                    let dep_status = match list.deprecated_at {
                        Some(_) => " (Deprecated)",
                        None => "",
                    };
                    println!(
                        "L\t{}\t{}\t{}\t{}",
                        list.id, list.uuid, list.name, dep_status
                    );
                }
            }
        }
        Sub::Lists => {
            let ws = api.get_workspace(&ws_name).await?;
            for list in ws.lists.iter() {
                let dep_status = match list.deprecated_at {
                    Some(_) => " (Deprecated)",
                    None => "",
                };
                println!("{}\t{}\t{}\t{}", list.id, list.uuid, list.name, dep_status);
            }
        }
        Sub::Users => {
            let ws = api.get_workspace(&ws_name).await?;
            for u in api.get_users(ws.get_id()).await?.iter() {
                println!("{}\t{}\t{}", u.id, u.uuid.clone(), u.display_name.clone());
            }
        }
        Sub::Items(list_opt) => {
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &list_opt.list).await?;
            let items = list_info.get_items().await?;
            for item in items.iter() {
                println!(
                    "{}\t{}\t{}",
                    item.get_id(),
                    item.get_uuid(),
                    item.display_string
                );
            }
        }
        Sub::Fields(list_opt) => {
            // show fields for list
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &list_opt.list).await?;
            for field in api.get_list_elements(list_info.get_id()).await?.iter() {
                println!(
                    "{}\t{}\t{}\t{}",
                    field.id, field.uuid, field.name, field.element_category
                )
            }
        }
        Sub::Field(field_opt) => {
            // show field detailed definition
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &field_opt.list).await?;
            match api
                .get_list_elements(list_info.get_id())
                .await?
                .iter()
                .find(|f| {
                    f.name == field_opt.field
                        || f.uuid == field_opt.field
                        || f.id.to_string() == field_opt.field
                }) {
                Some(f) => println!("{:#?}", f),
                None => println!("Field '{}' not found", field_opt.field),
            }
        }
        Sub::Choices(choices_opt) => {
            // show choices for field
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &choices_opt.list).await?;
            match api
                .get_list_elements(list_info.get_id())
                .await?
                .iter()
                .find(|f| {
                    f.name == choices_opt.field
                        || f.uuid == choices_opt.field
                        || f.id.to_string() == choices_opt.field
                }) {
                Some(field) => {
                    if field.element_category == ElementCategoryId::Categories {
                        if let Some(categories) = &field.element_data.predefined_categories {
                            for c in categories {
                                println!("{}\t{}", c.id, c.name);
                            }
                        }
                    } else {
                        println!("Field '{}' is not a choice field", choices_opt.field)
                    }
                }
                None => println!("Field '{}' not found", choices_opt.field),
            }
        }
        Sub::Item(item_opt) => {
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &item_opt.list).await?;
            let item = api.get_entry(list_info.get_id(), &item_opt.item).await?;
            println!("{:#?}", item);
        }
        Sub::Set(set_opt) => {
            // set value
            let sval = if let Some(value) = set_opt.value {
                value
            } else if let Some(fname) = set_opt.file {
                println!("Reading value from file {}", &fname);
                fs::read_to_string(&fname)?
            } else {
                // this case is already handled by clap, but Rust doesn't know that
                return Err(Error::Message(
                    "Either --value or --field must be used for set option".to_string(),
                ));
            };
            let value = match set_opt.text {
                None => FieldVal::Str(sval),
                Some(fmt) => FieldVal::Formatted(sval, fmt),
            };
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &set_opt.list).await?;
            list_info
                .update_item(
                    set_opt.item,
                    vec![(set_opt.field.clone(), value, UpdateAction::Replace)],
                )
                .await?;
        }
        Sub::Create(mut create_opt) => {
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &create_opt.list).await?;
            // create item
            let fields = create_opt
                .fields
                .drain(..)
                .map(|(k, v)| (k, parse_setval(v), UpdateAction::Null))
                .collect();
            let new_item = list_info.create_item(fields).await?;
            println!("{:#?}", new_item);
        }
        Sub::Comment(comment_opt) => {
            let ws = api.get_workspace(&ws_name).await?;
            let list_info = api.get_list_info(ws.get_id(), &comment_opt.list).await?;
            list_info
                .add_item_comment(
                    &comment_opt.item, // entry id or uuid
                    comment_opt.comment,
                )
                .await?;
        }
        Sub::ListWebhooks => {
            let resp = api.get_webhooks().await?;
            println!("{:#?}", resp);
        }
        Sub::DeleteWebhook(del_opt) => {
            let resp = api.delete_webhook(del_opt.webhook).await?;
            println!("{:#?}", resp);
        }
        Sub::Webhook(webhook_opt) => {
            let ws = api.get_workspace(&ws_name).await?;
            let mut item_id: Option<ID> = None;
            let mut list_id: Option<ID> = None;
            let mut field_id: Option<ID> = None; // experimental
            let mut workspace_id: Option<ID> = None;
            let list_info;
            match webhook_opt.list {
                Some(li) => {
                    list_info = api.get_list_info(ws.get_id(), &li).await?;
                    list_id = Some(list_info.get_id());

                    if let Some(it) = webhook_opt.item {
                        let item = list_info.get_item(&it).await?;
                        item_id = Some(item.get_id());
                    }

                    // watching field of list (experimental)
                    if let Some(fi) = webhook_opt.field {
                        let field = list_info.get_field(&fi)?;
                        field_id = Some(field.id);
                    }
                }
                None => {
                    if webhook_opt.item.is_some() {
                        return Err(Error::Message(
                            "If you use item id, you must also specify list id".to_string(),
                        ));
                    } else {
                        // no list, no item; must be looking for workspace
                        workspace_id = Some(ws.get_id());
                    }
                }
            };
            let hook = NewWebhook {
                trigger_type: match webhook_opt.trigger_type {
                    WebhookType::Item => WebhookTriggerType::Entry,
                    WebhookType::Activity => WebhookTriggerType::Activity,
                    WebhookType::Notification => WebhookTriggerType::Notification,
                    WebhookType::System => WebhookTriggerType::SystemMessage,
                    WebhookType::Comment => WebhookTriggerType::Comment,
                    WebhookType::Field => WebhookTriggerType::Element, // experimental
                },
                url: webhook_opt.url,
                list_id,
                list_entry_id: item_id,
                workspace_id,
                element_id: field_id,
                locale: webhook_opt.locale,
            };
            let response = api.create_webhook(&hook).await?;
            println!("{:#?}", response);
        }
        Sub::Backup(backup_opt) => {
            use std::time::SystemTime;
            let ws = api.get_workspace(&ws_name).await?;
            let mut lists: Vec<BackupItem> = Vec::new();
            if let Some(lname) = backup_opt.list {
                lists.push(backup_list(ws.get_id(), &lname, &backup_opt.output).await?);
            } else {
                // backup all lists
                for list in ws.lists.iter() {
                    lists.push(backup_list(ws.get_id(), &list.uuid, &backup_opt.output).await?);
                }
            }
            // create summary_tstamp.json
            let tstamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_millis() as u64,
                Err(_) => 0,
            };
            let summary = BackupSummary {
                workspace: ws.name.clone(),
                uuid: ws.uuid.clone(),
                tstamp,
                lists,
            };
            let summary_fname = format!("{}/summary_{}.json", &backup_opt.output, tstamp);
            let summary_data = serde_json::to_string(&summary).map_err(|e| {
                Error::Message(format!("Error generating summary: {}", e.to_string()))
            })?;
            fs::write(summary_fname, &summary_data)?;
        }
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct BackupSummary {
    workspace: String,
    uuid: String,
    tstamp: u64,
    lists: Vec<BackupItem>,
}
