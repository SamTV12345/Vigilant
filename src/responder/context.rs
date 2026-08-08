// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::LazyLock;

use time;

use url_serde::SerdeUrl;

use super::announcements::Announcement;
use crate::APP_CONF;
use crate::prober::states::ServiceStates;

const LOGO_EXTENSION_SPLIT_SPAN: usize = 4;

pub static INDEX_CONFIG: LazyLock<IndexContextConfig> = LazyLock::new(|| IndexContextConfig {
    runtime_version: env!("CARGO_PKG_VERSION").to_string(),
    page_title: APP_CONF.branding.page_title.to_owned(),
    company_name: APP_CONF.branding.company_name.to_owned(),
    icon_color: APP_CONF.branding.icon_color.to_owned(),
    icon_url: APP_CONF.branding.icon_url.to_owned(),
    icon_mime: ImageMime::guess_from(APP_CONF.branding.icon_url.as_str()),
    logo_color: APP_CONF.branding.logo_color.to_owned(),
    logo_url: APP_CONF.branding.logo_url.to_owned(),
    website_url: APP_CONF.branding.website_url.to_owned(),
    support_url: APP_CONF.branding.support_url.to_owned(),
    custom_html: APP_CONF.branding.custom_html.to_owned(),
});
pub static INDEX_ENVIRONMENT: LazyLock<IndexContextEnvironment> =
    LazyLock::new(|| IndexContextEnvironment::default());

#[derive(Serialize)]
pub enum ImageMime {
    #[serde(rename = "image/png")]
    ImagePNG,

    #[serde(rename = "image/jpeg")]
    ImageJPEG,

    #[serde(rename = "image/gif")]
    ImageGIF,

    #[serde(rename = "image/svg")]
    ImageSVG,
}

impl ImageMime {
    pub fn guess_from(logo_url: &str) -> ImageMime {
        if logo_url.len() > LOGO_EXTENSION_SPLIT_SPAN {
            let (_, logo_url_extension) =
                logo_url.split_at(logo_url.len() - LOGO_EXTENSION_SPLIT_SPAN);

            match logo_url_extension {
                ".svg" => ImageMime::ImageSVG,
                ".jpg" => ImageMime::ImageJPEG,
                ".gif" => ImageMime::ImageGIF,
                _ => ImageMime::ImagePNG,
            }
        } else {
            ImageMime::ImagePNG
        }
    }
}

impl Default for IndexContextEnvironment {
    fn default() -> Self {
        IndexContextEnvironment {
            year: time::OffsetDateTime::now_utc().year() as u16,
        }
    }
}

#[derive(Serialize)]
pub struct IndexContext<'a, 'b> {
    pub states: &'a ServiceStates,
    pub announcements: &'a Vec<Announcement>,
    pub environment: &'a IndexContextEnvironment,
    pub config: &'b IndexContextConfig,
}

#[derive(Serialize)]
pub struct IndexContextConfig {
    pub runtime_version: String,
    pub page_title: String,
    pub company_name: String,
    pub icon_color: String,
    pub icon_url: SerdeUrl,
    pub icon_mime: ImageMime,
    pub logo_color: String,
    pub logo_url: SerdeUrl,
    pub website_url: SerdeUrl,
    pub support_url: SerdeUrl,
    pub custom_html: Option<String>,
}

#[derive(Serialize)]
pub struct IndexContextEnvironment {
    pub year: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_from_svg() {
        let mime = ImageMime::guess_from("https://example.com/logo.svg");
        assert!(matches!(mime, ImageMime::ImageSVG));
    }

    #[test]
    fn test_guess_from_jpg() {
        let mime = ImageMime::guess_from("https://example.com/logo.jpg");
        assert!(matches!(mime, ImageMime::ImageJPEG));
    }

    #[test]
    fn test_guess_from_gif() {
        let mime = ImageMime::guess_from("https://example.com/logo.gif");
        assert!(matches!(mime, ImageMime::ImageGIF));
    }

    #[test]
    fn test_guess_from_png() {
        let mime = ImageMime::guess_from("https://example.com/logo.png");
        assert!(matches!(mime, ImageMime::ImagePNG));
    }

    #[test]
    fn test_guess_from_unknown_extension_defaults_to_png() {
        let mime = ImageMime::guess_from("https://example.com/logo.webp");
        assert!(matches!(mime, ImageMime::ImagePNG));
    }

    #[test]
    fn test_guess_from_no_extension_defaults_to_png() {
        let mime = ImageMime::guess_from("https://example.com/logo");
        assert!(matches!(mime, ImageMime::ImagePNG));
    }

    #[test]
    fn test_guess_from_short_url_defaults_to_png() {
        let mime = ImageMime::guess_from("a.b");
        assert!(matches!(mime, ImageMime::ImagePNG));
    }

    #[test]
    fn test_guess_from_empty_url() {
        let mime = ImageMime::guess_from("");
        assert!(matches!(mime, ImageMime::ImagePNG));
    }
}
