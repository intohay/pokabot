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
    blog_url: String,
    news_url: String
}

pub struct News {
    url: String,
    body: String,
    images: Vec<Bytes>
}

impl News {
    pub fn get_body(&self) -> &String {
        &self.body
    }
    pub fn get_url(&self) -> &String {
        &self.url
    }
    pub fn get_images(&self) -> &Vec<Bytes> {
        &self.images
    }
}

impl Scraper {

    const FILENAME:&str = "./last_blog.txt";

    pub fn new(base: &str, blog_url: &str, news_url: &str) -> Scraper {
        Scraper { base: base.to_string(), blog_url: blog_url.to_string(), news_url: news_url.to_string()}
    }

    pub fn get_base(&self) -> &String {
        &self.base
    }

    pub fn get_blog_url(&self) -> &String {
        &self.blog_url
    }

    pub async fn scrape_latest_url(&self) -> String {
        let html = reqwest::get(&self.blog_url)
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

    pub async fn scrape_blog_images(&self, url: &str) -> Vec<Bytes> {
        return self.scrape_images(url,  "div.c-blog-article__text img").await;
    } 

    pub async fn scrape_news_images(&self, url: &str) -> Vec<Bytes> {
        return self.scrape_images(url,  "div.p-article__text img").await;
    }

    async fn scrape_images(&self, url: &str, selector: &str) -> Vec<Bytes> {


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
        let sel = Selector::parse(selector).unwrap();
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

    async fn scrape_news(&self, url: &str) -> News {

        let url = if url.contains("https") {
            String::from(url)
        } else {
            format!("{}{}",self.get_base(), url)
        };

        let html = reqwest::get(&url)
        .await.unwrap().text().await.unwrap();
        let doc = scraper::Html::parse_document(&html);


        let title_sel = Selector::parse("div.c-article__title").unwrap();
        let body_sel = Selector::parse("div.p-article__text").unwrap();
        
        let title = doc.select(&title_sel).next().unwrap().text().collect::<String>();
        let body = doc.select(&body_sel).next().unwrap().text().collect::<String>();

        let images = self.scrape_news_images(&url).await;

        News {
            url: url, 
            body: format!("{}\n {}", title, body),
            images: images
        }
    }

    pub async fn scrape_until_latest_news(&self) -> Vec<News> {

        let previous_url = self.load_url("last_news.txt");
        

        let html = reqwest::get(&self.news_url)
        .await.unwrap().text().await.unwrap();
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("li.p-news__item > a").unwrap();


        let mut news_list : Vec<News> = vec![];
        
        for element in doc.select(&sel) {
            let href = element.value().attr("href").unwrap();
            if previous_url == href {
                break;
            }

            news_list.push(self.scrape_news(href).await);
            self.save_url(href, "last_news.txt");
            
            // last_news.txtが空だった場合、一個目でやめる
            if previous_url == "" {
                break;
            }
        } 

        return news_list;

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

    pub fn save_url(&self, url: &str, path: &str) {
        {
            let fp = File::create(path).unwrap();
            let mut writer = BufWriter::new(fp);

            writer.write(url.as_bytes()).unwrap();
        }

        let s = fs::read_to_string(path).unwrap();
        println!("{}",s);
    }

    pub fn load_url(&self, path: &str) -> String {
        let fp = File::open(path).unwrap();
        let reader = BufReader::new(fp);
        let mut url = String::new();
        for line in reader.lines(){
            url.push_str(&line.unwrap());
        }
        return url;
    }
}