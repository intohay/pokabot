use chrono::NaiveDateTime;
use bytes::Bytes;
pub struct News {
    pub url: String,
    pub body: String,
    pub posted_at: NaiveDateTime,
    pub images: Vec<Bytes>
}

impl News {
    pub fn new(url: String, body: String, posted_at: NaiveDateTime, images: Vec<Bytes>) -> Self {
        News {
            url,
            body,
            posted_at,
            images,
        }
    }

    pub fn url(&self) -> &String{
        &self.url
    }

    pub fn body(&self) -> &String {
        &self.body
    }

    pub fn posted_at(&self) -> &NaiveDateTime {
        &self.posted_at
    }

    pub fn images(&self) -> &Vec<Bytes> {
        &self.images
    }

}