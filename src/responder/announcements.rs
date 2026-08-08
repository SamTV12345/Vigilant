// Vigil
//
// Microservices Status Page
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::RwLock;
use time;
use time::format_description::FormatItem;

#[allow(deprecated)]
pub static DATE_NOW_FORMATTER: LazyLock<Vec<FormatItem<'static>>> = LazyLock::new(|| {
    time::format_description::parse(
        "[day padding:none] [month repr:short] [year], \
        [hour]:[minute]:[second] UTC[offset_hour sign:mandatory]:[offset_minute]",
    )
    .expect("invalid time format")
});

pub static STORE: LazyLock<Arc<RwLock<Store>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(Store {
        announcements: Vec::new(),
    }))
});

pub struct Store {
    pub announcements: Vec<Announcement>,
}

#[derive(Serialize)]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub text: String,
    pub date: Option<String>,
}
