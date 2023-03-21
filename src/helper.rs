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
    const TWITTER_LIMIT: usize = 280;
    count_twitter_chars(text) <= TWITTER_LIMIT
}


#[cfg(test)]
mod tests {
    use crate::helper::is_within_twitter_limit;

    #[test]
    fn test_is_within_twitter_limit(){
        // assert!(is_within_twitter_limit("山口陽世ちゃん、こんにちは！ブログ読んでファンになっちゃったよ☺️かき氷美味しそうだし、メキシコ戦感動的だったねOne choiceのフォーメーション楽しみにしてるよ！カラフルな衣装も可愛いし、まりぃちゃんのメガネ似合ってるねスパイファミリーのミュージカルも素晴らしかったんだね！リハ頑張ってね⚾️またブログ楽しみにしてるね\nhttps://www.hinatazaka46.com/s/official/diary/detail/49582?ima=0000&cd=member"));
        assert!(is_within_twitter_limit("山"));

    }
}