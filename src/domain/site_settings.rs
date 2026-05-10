use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SiteSettings {
    pub site_name: String,
    pub site_subtitle: String,
    pub site_description: String,
    pub footer_copyright: String,
    pub about_content: String,
    pub posts_per_page: u32,
    pub seo_title_template: String,
}

impl SiteSettings {
    pub fn from_map(map: &HashMap<String, String>) -> Self {
        Self {
            site_name: map.get("site_name").cloned().unwrap_or_default(),
            site_subtitle: map.get("site_subtitle").cloned().unwrap_or_default(),
            site_description: map.get("site_description").cloned().unwrap_or_default(),
            footer_copyright: map.get("footer_copyright").cloned().unwrap_or_default(),
            about_content: map.get("about_content").cloned().unwrap_or_default(),
            posts_per_page: map
                .get("posts_per_page")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            seo_title_template: map
                .get("seo_title_template")
                .cloned()
                .unwrap_or_else(|| "{title} | {site_name}".to_string()),
        }
    }
}
