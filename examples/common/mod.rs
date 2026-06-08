use alfred_workflow_rs::{Icon, Item, ItemText, Result};
use url::Url;

pub(crate) fn query_from_env() -> String {
    query_from_args(std::env::args().skip(1))
}

pub(crate) fn query_from_args<I>(args: I) -> String
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--query=") {
            return normalize_query(value);
        }

        if arg == "-q" || arg == "--query" {
            return match args.next() {
                Some(value) => normalize_query(&value),
                None => String::new(),
            };
        }
    }

    String::new()
}

pub(crate) fn placeholder_item() -> Item {
    Item::new("Search for some particular stuff ...").set_icon(Icon::new("icon.png"))
}

pub(crate) fn google_item(query: &str) -> Result<Item> {
    let mut url = Url::parse("https://www.google.com/search")?;
    url.query_pairs_mut().append_pair("q", query);
    let url = url.to_string();

    Ok(
        Item::with_arg("Sorry I can't help you with that query.", &url)
            .set_subtitle("Shall I try and search Google?")
            .set_text(ItemText::new(&url))
            .set_quick_look_url(&url)
            .set_icon(Icon::new("google.png"))
            .set_valid(true),
    )
}

fn normalize_query(value: &str) -> String {
    let mut normalized = String::new();

    for part in value.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(part);
    }

    normalized
}
