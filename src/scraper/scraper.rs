use chrono::NaiveDateTime;
use scraper::Selector;
use regex::Regex; 
use tokio::time;
use bytes::Bytes;
use url::Url;
use crate::scraper::blog::Blog;
use crate::scraper::news::News;

pub struct Scraper {
    base: String,
    blog_url: String,
    news_url: String
}







impl Scraper {

    pub fn new(base: &str, blog_url: &str, news_url: &str) -> Scraper {
        Scraper { base: base.to_string(), blog_url: blog_url.to_string(), news_url: news_url.to_string()}
    }

    pub fn get_base(&self) -> &String {
        &self.base
    }

    pub fn get_blog_url(&self) -> &String {
        &self.blog_url
    }

    pub fn get_news_url(&self) -> &String {
        &self.news_url
    }


    
    pub async fn scrape_post_ids(&self) -> anyhow::Result<Vec<i32>> {
        let html = reqwest::get(self.get_blog_url()).await?.text().await?;
        let doc = scraper::Html::parse_document(&html);

        let sel = Selector::parse("ul.p-blog-top__list > li > a").unwrap();

        let mut post_ids: Vec<i32> = vec![];
        for element in doc.select(&sel) {
            let href = element.value().attr("href").unwrap();
            if let Some(post_id) = self.extract_post_id(href) {
                post_ids.push(post_id);
            }
            
        }

        Ok(post_ids)

    }

    pub async fn scrape_news_ids(&self) -> anyhow::Result<Vec<String>> {
        let html = reqwest::get(self.get_news_url()).await?.text().await?;
        let doc = scraper::Html::parse_document(&html);
        let sel = Selector::parse("div.p-news__list-group.js-news-tab-list:not(#js-news-tab-list--fc) > ul > li > a").unwrap();

        let mut news_ids: Vec<String> = vec![];
        for element in doc.select(&sel) {
            let href = element.value().attr("href").unwrap();
            if let Some(news_id) = self.extract_news_id(href) {
                news_ids.push(news_id);
            }
        }

        Ok(news_ids)
    }

    pub async fn scrape_news(&self, news_id: &str) -> anyhow::Result<News> {

        let url = format!("https://www.hinatazaka46.com/s/official/news/detail/{}?ima=0000", news_id);
        
        println!("{}",url);
        let html = reqwest::get(&url)
        .await?.text().await?;

        let doc = scraper::Html::parse_document(&html);


        let title_sel = Selector::parse("div.c-article__title").unwrap();
        let date_sel = Selector::parse("time.c-news__date").unwrap();
        let body_sel = Selector::parse("div.p-article__text").unwrap();
        let image_sel = Selector::parse("div.p-article__text img").unwrap();
        


        let title = doc.select(&title_sel).next().unwrap().text().collect::<String>();

        let date_str = doc.select(&date_sel).next().unwrap().text().collect::<String>();
        let posted_at = NaiveDateTime::parse_from_str(&format!("{} 00:00",date_str), "%Y.%-m.%-d %H:%M").unwrap();

        let body = doc.select(&body_sel).next().unwrap().text().collect::<String>();

        let mut images: Vec<Bytes> = vec![];
        for element in doc.select(&image_sel) {
            let src = element.value().attr("src").unwrap();
            
            if !src.contains("https") {
                continue;
            }
            
            println!("{}", src);
           
            let bytes = reqwest::get(src).await?
                                    .bytes().await?;
           
            images.push(bytes);

            time::sleep(time::Duration::from_secs(1)).await;
        }
        Ok(News {
            url: url, 
            body: format!("{}\n {}", title, body),
            posted_at: posted_at,
            images: images
        })
    }

    pub async fn scrape_blog(&self, post_id: i32) -> anyhow::Result<Blog> {
       let url = format!("https://www.hinatazaka46.com/s/official/diary/detail/{}?ima=0000&cd=member", post_id);

       println!("url: {}",url);

        let html = reqwest::get(&url)
        .await?.text().await?;

        let doc = scraper::Html::parse_document(&html);

        let title_sel = Selector::parse("div.c-blog-article__title").unwrap();
        let name_sel = Selector::parse("div.c-blog-article__name > a").unwrap();
        let date_sel = Selector::parse("div.c-blog-article__date > time").unwrap();
        let body_sel = Selector::parse("div.c-blog-article__text").unwrap();
        let image_sel = Selector::parse("div.c-blog-article__text img").unwrap();
        

        let a_tag = doc.select(&name_sel).next().unwrap();
        let name = a_tag.text().collect::<String>();

        let title = doc.select(&title_sel).next().unwrap().text().collect::<String>();

        let date_str = doc.select(&date_sel).next().unwrap().text().collect::<String>();
        let posted_at = NaiveDateTime::parse_from_str(&date_str, "%Y.%-m.%-d %H:%M").unwrap();


        let mut body = String::new();
        for element in doc.select(&body_sel) {
            body.push_str(&element.text().collect::<String>());
        }

        let mut images: Vec<Bytes> = vec![];
        for element in doc.select(&image_sel) {
            if let Some(src) = element.value().attr("src") {
                if !src.contains("https") {
                    continue;
                }
                println!("{}", src);
            
                let bytes = reqwest::get(src).await?
                                        .bytes().await?;
            
                images.push(bytes);

                time::sleep(time::Duration::from_secs(1)).await;
            }

            
        }

        Ok(Blog{
            url: url,
            name: name,
            title: title, 
            body: body, 
            posted_at: posted_at, 
            images: images
        })




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

    fn extract_post_id(&self, url: &str) -> Option<i32> {
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

    fn extract_news_id(&self, url: &str) -> Option<String> {
        let re = Regex::new(r"[A-Z]\d+").unwrap();
        re.find(url).map(|m| m.as_str().to_string())
    }

   

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_news_id() {
        let scraper = Scraper::new("", "", "");
        let result1 = scraper.extract_news_id("/s/official/news/detail/M00058");
        let result2 = scraper.extract_news_id("/s/official/news/detail/O12345");

        println!("{:?}", result1);
        println!("{:?}", result2);
        assert_eq!(result1, Some("M00058".to_string()));
        assert_eq!(result2, Some("O12345".to_string()));
    }
}