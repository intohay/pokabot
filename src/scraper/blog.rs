// pokabot/scraper/blog.rs
use chrono::NaiveDateTime;
use bytes::Bytes;

pub struct Blog {
    pub url: String,
    pub name: String,
    pub title: String,
    pub body: String,
    pub posted_at: NaiveDateTime,
    pub images: Vec<Bytes>,
}

impl Blog {
    // `Blog`の新しいインスタンスを作成するためのコンストラクタ
    pub fn new(url: String, name: String, title: String, body: String, posted_at: NaiveDateTime, images: Vec<Bytes>) -> Self {
        Blog {
            url,
            name,
            title,
            body,
            posted_at,
            images,
        }
    }

    // 各フィールドのゲッターメソッドを追加
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn posted_at(&self) -> &NaiveDateTime {
        &self.posted_at
    }

    pub fn images(&self) -> &Vec<Bytes> {
        &self.images
    }
}
