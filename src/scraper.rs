use scraper::Selector;
use std::fs::{self, File};
use std::io::{Write, BufWriter, BufRead, BufReader};


pub struct Scraper {
    base: String,
    url: String
}

impl Scraper {

    const FILENAME:&str = "./last_blog.txt";

    pub fn new(base: &str, url: &str) -> Scraper {
        Scraper { base: base.to_string(), url: url.to_string() }
    }

    pub async fn scrape_latest_url(&self) -> String {
        let html = reqwest::get(&self.url)
        .await
        .unwrap()
        .text().await.unwrap();
        
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("a.p-blog-main__image").unwrap();
        let mut url = String::new();
        for node in doc.select(&sel) {
            let href = node.value().attr("href").unwrap();
            url.push_str(href);
            break;
        }
        
        return url;
    }

    // pub async fn scrape_images(&self, url: String) {

    // }


    pub async fn scrape_text(&self, url: &str) -> String {
        let html = reqwest::get(format!("{}{}",self.base, url))
        .await.unwrap().text().await.unwrap();
        
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("div.c-blog-article__text").unwrap();
        let mut text = String::new();
        for element in doc.select(&sel) {
            text.push_str(&element.text().collect::<String>());
        }

        return text;
    }

    pub fn save_url(&self, url: &str) {
        {
            let fp = File::create(Self::FILENAME).unwrap();
            let mut writer = BufWriter::new(fp);

            writer.write(url.as_bytes()).unwrap();
        }

        let s = fs::read_to_string(Self::FILENAME).unwrap();
        println!("{}",s);
    }

    pub fn load_url(&self) -> String {
        let fp = File::open(Self::FILENAME).unwrap();
        let reader = BufReader::new(fp);
        let mut url = String::new();
        for line in reader.lines(){
            url.push_str(&line.unwrap());
        }
        return url;
    }
}