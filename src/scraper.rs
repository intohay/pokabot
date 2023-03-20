use scraper::Selector;
use std::fs::{self, File};
use std::io::{Write, BufWriter, BufRead, BufReader};
use regex::Regex; 
use base64::encode;
use tokio::time;
use bytes::Bytes;
use url::Url;

pub struct Scraper {
    base: String,
    url: String
}

impl Scraper {

    const FILENAME:&str = "./last_blog.txt";

    pub fn new(base: &str, url: &str) -> Scraper {
        Scraper { base: base.to_string(), url: url.to_string() }
    }

    pub fn get_base(&self) -> &String {
        &self.base
    }

    pub fn get_url(&self) -> &String {
        &self.url
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

    fn extract_id(&self, url: &str) -> Option<String> {
        let re = Regex::new(r"/(\d+)\?").unwrap();
        re.captures(url).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_owned())
    }

    pub async fn scrape_images(&self, url: &str) -> Vec<Bytes> {


        // let post_id = self.extract_id(url).unwrap();
        let post_url: String;
        if url.contains("https") {
            post_url = String::from(url);
        } else {
            post_url = format!("{}{}",self.base, url);
        }

        let html = reqwest::get(post_url)
        .await.unwrap().text().await.unwrap();
        
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("div.c-blog-article__text img").unwrap();
        let mut images: Vec<Bytes> = vec![];

        for (i, element) in doc.select(&sel).enumerate() {
            let src = element.value().attr("src").unwrap();
            
            if !src.contains("https") {
                continue;
            }
            
            println!("{}", src);
           
            let bytes = reqwest::get(src).await.unwrap()
                                    .bytes().await.unwrap();
           
            images.push(bytes);

            time::sleep(time::Duration::from_millis(1000)).await;
        }

        return images;


    }

    

    pub async fn scrape_name(&self, url: &str) -> String {
        let post_url = if url.contains("https") {
            String::from(url)
        } else {
            format!("{}{}",self.base, url)
        };


        let html = reqwest::get(post_url)
        .await.unwrap().text().await.unwrap();
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("div.c-blog-article__name > a").unwrap();
        
        let a_tag = doc.select(&sel).next().unwrap();
        let name = a_tag.text().collect::<String>();

        return name;

    }

    pub async fn scrape_text(&self, url: &str) -> String {

        let post_url = 
        if url.contains("https") {
            String::from(url)
        } else {
            format!("{}{}",self.base, url)
        };

        let html = reqwest::get(post_url)
        .await.unwrap().text().await.unwrap();
        
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("div.c-blog-article__text").unwrap();
        let mut text = String::new();
        for element in doc.select(&sel) {
            text.push_str(&element.text().collect::<String>());
        }

        return text;
    }

    pub async fn page_exists(&self, url: &str) -> bool {
        let post_url = 
        if url.contains("https") {
            String::from(url)
        } else {
            format!("{}{}",self.base, url)
        };

        let html = reqwest::get(post_url)
        .await.unwrap().text().await.unwrap();
        
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("div.c-blog-article__text").unwrap();

        return doc.select(&sel).next().is_some();
    }

    pub fn extract_post_id(&self, url: &str) -> Option<usize> {
        let post_url = 
        if url.contains("https") {
            String::from(url)
        } else {
            format!("{}{}",self.base, url)
        };

        let parsed_url = Url::parse(&post_url).ok()?;
        let path_segments: Vec<_> = parsed_url.path_segments()?.collect();

        if let Some(detail_index) = path_segments.iter().position(|&s| s == "detail") {
            if let Ok(id) = path_segments[detail_index + 1].parse() {
                return Some(id);
            }
        }

        None
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