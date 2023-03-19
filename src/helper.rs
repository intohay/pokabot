use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;




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

