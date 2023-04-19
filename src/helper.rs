use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;



pub fn count_twitter_chars(text: &str) -> usize {
            let url_regex = Regex::new(r"https?://\S+").unwrap();
            let mut count = 0;

            for token in url_regex.split(text) {
                let graphemes = UnicodeSegmentation::graphemes(token, true);
                for grapheme in graphemes {
                    count += if grapheme.width() > 1 { 2 } else { 1 };
                }
            }

            let url_count = url_regex.find_iter(text).count() * 23;
            println!("{}",count);
            count + url_count
        }

pub fn is_within_twitter_limit(text: &str) -> bool {
    const TWITTER_LIMIT: usize = 260;
    count_twitter_chars(text) <= TWITTER_LIMIT
}


#[cfg(test)]
mod tests {
    use crate::helper::is_within_twitter_limit;

    #[test]
    fn test_is_within_twitter_limit(){
        assert!(is_within_twitter_limit("東村芽依ちゃんのブログ読んだよ！サインとメッセージ書かせてもらったんだね！｢One choice｣発売日おめでとう特に｢シーラカンス｣好きなんだって！4期生の透き通った歌声最高だね！世界卓球2023の応援サポーターに就任したって！小さい頃卓球好きだった
からすごく嬉しいね\nhttps://www.hinatazaka46.com/s/official/diary/detail/50010?ima=0000&cd=member"));
        assert!(is_within_twitter_limit("山"));

    }
}