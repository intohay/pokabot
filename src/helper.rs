use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

use url::Url;


pub fn count_twitter_chars(text: &str) -> usize {
            let url_regex = Regex::new(r"https?://\S+").unwrap();
            let mut count = 0;

            for token in url_regex.split(text) {
                count += UnicodeSegmentation::graphemes(token, true).count();
            }

            let url_count = url_regex.find_iter(text).count() * 23;
            count + url_count
        }

pub fn is_within_twitter_limit(text: &str) -> bool {
    const TWITTER_LIMIT: usize = 280;
    count_twitter_chars(text) <= TWITTER_LIMIT
}

pub fn extract_post_id(url: &str) -> Option<usize> {
    let parsed_url = Url::parse(url).ok()?;
    let path_segments: Vec<_> = parsed_url.path_segments()?.collect();

    if let Some(detail_index) = path_segments.iter().position(|&s| s == "detail") {
        if let Ok(id) = path_segments[detail_index + 1].parse() {
            return Some(id);
        }
    }

    None
}