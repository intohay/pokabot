use diesel::{Queryable, Insertable};
use chrono::prelude::*;
use crate::schema::blogs;
use crate::schema::news;



#[derive(Queryable)]
pub struct Blog {
    pub id: i32,
    pub name: String,
    pub posted_at: NaiveDateTime,
    pub jp_tweeted: bool,
    pub eng_tweeted: bool
}

#[derive(Insertable)]
#[diesel(table_name = blogs)]
pub struct NewBlog<'a> {
    pub id: i32,
    pub name: &'a str,
    pub posted_at: NaiveDateTime,
    pub jp_tweeted: bool,
    pub eng_tweeted: bool
}


#[derive(Queryable)]
pub struct News {
    pub id: i32,
    pub news_id: String,
    pub posted_at: NaiveDateTime,
    pub jp_tweeted: bool,
    pub eng_tweeted: bool
}

#[derive(Insertable)]
#[diesel(table_name = news)]
pub struct NewNews<'a> {
    pub news_id: &'a str,
    pub posted_at: NaiveDateTime,
    pub jp_tweeted: bool,
    pub eng_tweeted: bool
}


