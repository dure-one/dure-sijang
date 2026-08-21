use crate::db::establish_connection;
use crate::models::browser::*;
use crate::schema::*;
use diesel::prelude::*;
use anyhow::Result;
use chrono::Utc;

// Tab operations
pub fn create_tab(title: &str, url: &str, mode: &str) -> Result<Tab> {
    let conn = &mut establish_connection();
    let now = Utc::now().naive_utc();

    let new_tab = NewTab {
        title,
        url,
        mode,
        store_id: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(tabs::table)
        .values(&new_tab)
        .execute(conn)?;

    tabs::table
        .order(tabs::id.desc())
        .first(conn)
        .map_err(Into::into)
}

pub fn update_tab(id: i32, title: &str, url: &str) -> Result<()> {
    let conn = &mut establish_connection();
    let now = Utc::now().naive_utc();

    diesel::update(tabs::table.filter(tabs::id.eq(id)))
        .set((
            tabs::title.eq(title),
            tabs::url.eq(url),
            tabs::updated_at.eq(now),
        ))
        .execute(conn)?;

    Ok(())
}

pub fn delete_tab(id: i32) -> Result<()> {
    let conn = &mut establish_connection();

    diesel::delete(tabs::table.filter(tabs::id.eq(id)))
        .execute(conn)?;

    Ok(())
}

pub fn get_all_tabs() -> Result<Vec<Tab>> {
    let conn = &mut establish_connection();

    tabs::table
        .order(tabs::created_at.asc())
        .load::<Tab>(conn)
        .map_err(Into::into)
}

pub fn get_tab(id: i32) -> Result<Tab> {
    let conn = &mut establish_connection();

    tabs::table
        .filter(tabs::id.eq(id))
        .first(conn)
        .map_err(Into::into)
}

// Browsing history operations
pub fn add_history(tab_id: i32, url: &str, title: Option<&str>) -> Result<()> {
    let conn = &mut establish_connection();
    let now = Utc::now().naive_utc();

    let new_history = NewBrowsingHistory {
        tab_id,
        url,
        title,
        visited_at: now,
    };

    diesel::insert_into(browsing_history::table)
        .values(&new_history)
        .execute(conn)?;

    Ok(())
}

pub fn get_history(tab_id: i32, limit: i64) -> Result<Vec<BrowsingHistory>> {
    let conn = &mut establish_connection();

    browsing_history::table
        .filter(browsing_history::tab_id.eq(tab_id))
        .order(browsing_history::visited_at.desc())
        .limit(limit)
        .load::<BrowsingHistory>(conn)
        .map_err(Into::into)
}

pub fn get_all_history(limit: i64) -> Result<Vec<BrowsingHistory>> {
    let conn = &mut establish_connection();

    browsing_history::table
        .order(browsing_history::visited_at.desc())
        .limit(limit)
        .load::<BrowsingHistory>(conn)
        .map_err(Into::into)
}

pub fn clear_history(tab_id: Option<i32>) -> Result<()> {
    let conn = &mut establish_connection();

    if let Some(id) = tab_id {
        diesel::delete(browsing_history::table.filter(browsing_history::tab_id.eq(id)))
            .execute(conn)?;
    } else {
        diesel::delete(browsing_history::table)
            .execute(conn)?;
    }

    Ok(())
}

// Bookmark operations
pub fn add_bookmark(title: &str, url: &str, store_id: Option<i32>) -> Result<()> {
    let conn = &mut establish_connection();
    let now = Utc::now().naive_utc();

    let new_bookmark = NewBookmark {
        title,
        url,
        store_id,
        created_at: now,
    };

    diesel::insert_into(bookmarks::table)
        .values(&new_bookmark)
        .execute(conn)?;

    Ok(())
}

pub fn get_bookmarks() -> Result<Vec<Bookmark>> {
    let conn = &mut establish_connection();

    bookmarks::table
        .order(bookmarks::created_at.desc())
        .load::<Bookmark>(conn)
        .map_err(Into::into)
}

pub fn delete_bookmark(id: i32) -> Result<()> {
    let conn = &mut establish_connection();

    diesel::delete(bookmarks::table.filter(bookmarks::id.eq(id)))
        .execute(conn)?;

    Ok(())
}

pub fn bookmark_exists(url: &str) -> Result<bool> {
    let conn = &mut establish_connection();

    let count: i64 = bookmarks::table
        .filter(bookmarks::url.eq(url))
        .count()
        .get_result(conn)?;

    Ok(count > 0)
}
