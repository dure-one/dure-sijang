use super::super::schema::{tabs, browsing_history, bookmarks};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = tabs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Tab {
    pub id: Option<i32>,
    pub title: String,
    pub url: String,
    pub mode: String,
    pub store_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = tabs)]
pub struct NewTab<'a> {
    pub title: &'a str,
    pub url: &'a str,
    pub mode: &'a str,
    pub store_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = browsing_history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BrowsingHistory {
    pub id: Option<i32>,
    pub tab_id: i32,
    pub url: String,
    pub title: Option<String>,
    pub visited_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = browsing_history)]
pub struct NewBrowsingHistory<'a> {
    pub tab_id: i32,
    pub url: &'a str,
    pub title: Option<&'a str>,
    pub visited_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = bookmarks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Bookmark {
    pub id: Option<i32>,
    pub title: String,
    pub url: String,
    pub store_id: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = bookmarks)]
pub struct NewBookmark<'a> {
    pub title: &'a str,
    pub url: &'a str,
    pub store_id: Option<i32>,
    pub created_at: NaiveDateTime,
}
