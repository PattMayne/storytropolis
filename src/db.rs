// I'm actually using MariaDB which is supposedly a drop-in replacement for MySQL

use jsonwebtoken::signature::digest::typenum::uint;
use sqlx::{ MySqlPool };
use time::{ OffsetDateTime, Duration, macros::format_description };
use anyhow::{ Result, anyhow };
use serde;
use comrak::{markdown_to_html, Options};


use crate::{
    utils,
    auth,
};

/* 
 * 
 * 
 * 
 * 
 * =============================
 * =============================
 * =====                   =====
 * =====  STRUCTS & ENUMS  =====
 * =====                   =====
 * =============================
 * =============================
 * 
 * 
 * 
 * 
 */


// use pattern matching in an impl function to get a String to store to DB
pub enum UserRole {
    Admin,
    Reader,
}

#[derive(Debug)]
struct Count {
    count: i64,
}



#[derive(serde::Serialize)]
pub struct User {
    id: i32,
    username: String,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    role: String,
    password_hash: String,
    created_timestamp: OffsetDateTime,
    email_verified: i8 // actually a bool but mysql doesn't do "real" bools
}

#[derive(Clone)]
#[derive(serde::Serialize)]
pub struct BlogPost {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub author_name: String,
    pub pinned: i8,
    pub pinned_to_blog: i8,
    pub created_timestamp: OffsetDateTime,
    pub updated_timestamp: OffsetDateTime,
}

#[derive(serde::Serialize)]
pub struct UnifiedPost {
    pub post: BlogPost,
    pub categories: Vec<String>,
}

impl UnifiedPost {
    pub fn cats_string(&self) -> String {
        self.categories.join(", ")
    }
}


#[derive(serde::Serialize)]
pub struct Username {
    pub username: String,
}


#[derive(serde::Serialize)]
pub struct UsernameAndRole {
    pub username: String,
    pub role: String,
    pub email_verified: i8,
}

impl UsernameAndRole {
    pub fn is_verified(&self) -> bool {
        self.email_verified != 0
    }
}


#[derive(serde::Serialize)]
pub struct RefreshToken {
    id: i32,
    user_id: i32,
    token: String,
    created_timestamp: OffsetDateTime,
    expires_timestamp: OffsetDateTime
}


#[derive(serde::Serialize)]
pub struct RedirectUri {
    pub redirect_uri: String,
}


impl BlogPost {
    pub fn get_formatted_updated_timestamp(&self) -> String {
        let format: &[time::format_description::BorrowedFormatItem<'_>] =
            format_description!("[day] [month repr:short], [year]");
        self.updated_timestamp.format(&format).unwrap()
    }

    pub fn is_pinned_to_blog(&self) -> bool {
        self.pinned_to_blog == 1
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned == 1
    }

    pub fn get_body_as_html(&self) -> String {
        let mut options: Options<'_> = Options::default();
        options.render.r#unsafe = true;
        markdown_to_html(&self.body, &options)
    }
}


impl RefreshToken {
    pub fn get_token(&self) -> &String { &self.token }
    pub fn get_user_id(&self) -> i32 { self.user_id }
    pub fn get_created_timestamp(&self) -> &OffsetDateTime { &self.created_timestamp }
    pub fn get_expires_timestamp(&self) -> &OffsetDateTime { &self.expires_timestamp }

    pub fn is_expired(&self) -> bool {
        self.expires_timestamp < OffsetDateTime::now_utc()
    }
}


impl User {

    pub fn get_email_verified(&self) -> bool {
        self.email_verified != 0
    }

    pub fn new(
        id: i32,
        username: String,
        email: String,
        first_name: Option<String>,
        last_name: Option<String>,
        role: String,
        password_hash: String,
        created_timestamp: OffsetDateTime,
        email_verified: bool
    ) -> Self {
        User {
            id, username, email, first_name, last_name, role,
            password_hash, created_timestamp,
            email_verified: if email_verified { 1 } else { 0 }
        }
    }

    pub fn get_password_hash(&self) -> &String {
        &self.password_hash
    }

    pub fn get_email(&self) -> &String {
        &self.email
    }

    pub fn get_id(&self) -> i32 { self.id }
    pub fn get_role(&self) -> &String { &self.role }
    pub fn get_username(&self) -> &String { &self.username }

    pub fn get_first_name(&self) -> String {
        match self.first_name.clone() {
            Some(first_name) => first_name.to_owned(),
            None => String::new()
        }
    }

    pub fn get_last_name(&self) -> String {
        match self.last_name.clone() {
            Some(last_name) => last_name.to_owned(),
            None => String::new()
        }
    }

}



/* 
 * 
 * 
 * 
 * 
 * ==============================
 * ==============================
 * =====                    =====
 * =====  SELECT FUNCTIONS  =====
 * =====                    =====
 * ==============================
 * ==============================
 * 
 * retrieving data from the DB
 * 
 * 
 * 
 * 
 * 
*/


// You got the post. Now package it with its categories.
pub async fn get_unified_post(
    pool: &MySqlPool,
    post: BlogPost
) -> Result<UnifiedPost, sqlx::Error> {
    let categories: Vec<String> =
        get_categories_by_post_id(post.id as i64, pool)
        .await?;
    Ok(UnifiedPost{ post, categories })
}


// You got a collection of posts. Now package them with their categories.
pub async fn get_unified_posts_from_posts(
    pool: &MySqlPool,
    posts: Vec<BlogPost>
) -> Result<Vec<UnifiedPost>> {
    let mut uposts: Vec<UnifiedPost> = Vec::new();
    let mut tx: sqlx::Transaction<'_, sqlx::MySql> = pool.begin().await?;

    for post in posts {
        let categories: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT c.name
            FROM categories c
            INNER JOIN post_categories pc
                ON pc.category_id = c.id
            WHERE pc.post_id = ?
            ORDER BY c.name
            "#
        )
        .bind(post.id)
        .fetch_all(&mut *tx)
        .await?;
        
        uposts.push(UnifiedPost { post, categories });
    }

    tx.commit().await?;
    Ok(uposts)
}



pub async fn get_post_by_id(
    pool: &MySqlPool,
    post_id: i64
) -> Result<Option<BlogPost>> {
    Ok(sqlx::query_as!(
            BlogPost,
            "SELECT id, author_name, title, body,
            created_timestamp, updated_timestamp, pinned, 
            pinned_to_blog FROM blog_post WHERE id = ?",
            post_id
        ).fetch_optional(pool).await?)
}


pub async fn get_latest_pinned_post(
    pool: &MySqlPool
) -> Result<Option<BlogPost>> {
    Ok(sqlx::query_as!(
            BlogPost,
            "SELECT id, author_name, title, body,
            created_timestamp, updated_timestamp, pinned, pinned_to_blog
            FROM blog_post WHERE pinned = ? ORDER BY created_timestamp DESC LIMIT 1",
            true
        ).fetch_optional(pool).await?)
}


pub async fn get_posts(
    pool: &MySqlPool
) -> Result<Vec<BlogPost>> {
    let blog_posts: Vec<BlogPost> = sqlx::query_as!(
        BlogPost,
        "SELECT id, author_name, title, body, created_timestamp, 
        updated_timestamp, pinned, pinned_to_blog
        FROM blog_post ORDER BY created_timestamp DESC"
    ).fetch_all(pool).await?;

    Ok(blog_posts)
}

pub async fn get_categories_by_post_id(
    post_id: i64,
    pool: &MySqlPool,
) -> Result<Vec<String>, sqlx::Error> {

    let category_names: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT c.name
        FROM categories c
        INNER JOIN post_categories pc
            ON pc.category_id = c.id
        WHERE pc.post_id = ?
        ORDER BY c.name
        "#
    )
    .bind(post_id)
    .fetch_all(pool)
    .await?;

    Ok(category_names)
}


pub async fn get_active_categories(
    pool: &MySqlPool
) -> Result<Vec<String>, sqlx::Error> {
    let category_names: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT c.name
        FROM categories c
        INNER JOIN post_categories pc
            ON pc.category_id = c.id
        GROUP BY c.id, c.name
        ORDER BY COUNT(*) DESC, c.name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(category_names)
}


pub async fn get_non_pinned_posts(
    pool: &MySqlPool
) -> Result<Vec<BlogPost>> {
    let blog_posts: Vec<BlogPost> = sqlx::query_as!(
        BlogPost,
        "SELECT id, author_name, title, body, created_timestamp, 
        updated_timestamp, pinned, pinned_to_blog
        FROM blog_post WHERE pinned = ? 
        ORDER BY created_timestamp DESC",
        false
    ).fetch_all(pool).await?;

    Ok(blog_posts)
}


pub async fn get_user_by_username(
    pool: &MySqlPool,
    username: &String
) -> Result<Option<User>> {
    Ok(sqlx::query_as!(
            User,
            "SELECT id, username, email, first_name,
                last_name, role, password_hash, created_timestamp,
                email_verified FROM users WHERE username = ?",
            username
        ).fetch_optional(pool).await?)
}


pub async fn get_verification_code(
    pool: &MySqlPool,
    user_id: i32
) -> Result<Option<auth::HashedVerificationCode>> {
    Ok(sqlx::query_as!(
        auth::HashedVerificationCode,
        "SELECT user_id, code_hash, attempts,
            created_timestamp, expires_timestamp
            FROM verification_codes WHERE user_id = ?",
        user_id
    ).fetch_optional(pool).await?)
}


pub async fn get_user_by_email(
    pool: &MySqlPool,
    email: &String
) -> Result<Option<User>> {
    Ok(sqlx::query_as!(
        User,
        "SELECT id, username, email,
            first_name, last_name, role,
            password_hash, created_timestamp,
            email_verified FROM users WHERE email = ?",
        email
    ).fetch_optional(pool).await?)
}


pub async fn get_username_by_id(
    pool: &MySqlPool,
    id: i32
) -> Result<Option<Username>> {
    Ok(sqlx::query_as!(
        Username,
        "SELECT username FROM users WHERE id = ?",
        id
    ).fetch_optional(pool).await?)
}


pub async fn get_username_and_role_by_id(
    pool: &MySqlPool,
    id: i32
) -> Result<Option<UsernameAndRole>> {
    Ok(sqlx::query_as!(
        UsernameAndRole,
        "SELECT username, role, email_verified FROM users WHERE id = ?",
        id
    ).fetch_optional(pool).await?)
}


pub async fn get_user_by_id(
    pool: &MySqlPool,
    id: i32
) -> Result<Option<User>> {
    Ok(sqlx::query_as!(
        User,
        "SELECT id, username, email,
            first_name, last_name, role,
            password_hash, created_timestamp,
            email_verified FROM users WHERE id = ?",
        id
    ).fetch_optional(pool).await?)
}


/**
 * Get a refresh token for specified user
 */
pub async fn get_refresh_token(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Option<RefreshToken>> {
    Ok(sqlx::query_as!(
        RefreshToken,
        "SELECT id, user_id, token, created_timestamp, expires_timestamp
            FROM refresh_tokens WHERE user_id = ?",
        user_id
    ).fetch_optional(pool).await?)
}


/* 
 * 
 * 
 * 
 * 
 * 
 * 
 * ==============================
 * ==============================
 * =====                    =====
 * =====  INSERT FUNCTIONS  =====
 * =====                    =====
 * ==============================
 * ==============================
 * 
 * Adding new entries to the database
 * 
 * 
 * 
 * 
 */

 /**
  * Add a refresh token to the database for a particular user.
  * Take ownership of token, because it should ONLY be given back
  * if it's saved successfully to the DB.
  */
 pub async fn add_refresh_token(
    pool: &MySqlPool,
    user_id: i32,
    refresh_token: String
) -> Result<String, anyhow::Error> {
    let expires_timestamp: OffsetDateTime =
        OffsetDateTime::now_utc() + Duration::days(14); // TODO: put this in resources?
    let created_timestamp: OffsetDateTime = OffsetDateTime::now_utc();

    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "INSERT INTO refresh_tokens (
            user_id,
            token,
            created_timestamp,
            expires_timestamp)
        VALUES (?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            token = VALUES(token),
            created_timestamp = VALUES(created_timestamp),
            expires_timestamp = VALUES(expires_timestamp);
            ")
    .bind(user_id)
    .bind(&refresh_token)
    .bind(created_timestamp)
    .bind(expires_timestamp)
    .execute(pool).await.map_err(|e| {
        eprintln!("Failed to save refresh_token to database: {:?}", e);
        anyhow!("Could not save refresh_token to database: {e}")
    })?;

    // Return the refresh_token, because now it's safe to use (saved to DB)
    println!("Added refresh token id: {}", result.last_insert_id());
    println!{"{}", refresh_token};
    Ok(refresh_token)
 }


 
 /**
  * Add a auth token to the database for a particular user.
  * Take ownership of token, because it should ONLY be given back
  * if it's saved successfully to the DB.
  */
 pub async fn add_auth_code(
    pool: &MySqlPool,
    user_id: i32,
    auth_code: String
) -> Result<String, anyhow::Error> {
    let expires_timestamp: OffsetDateTime =
        OffsetDateTime::now_utc() + Duration::minutes(1);
    let created_timestamp: OffsetDateTime = OffsetDateTime::now_utc();

    let _result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "INSERT INTO auth_codes (
            user_id,
            code,
            created_timestamp,
            expires_timestamp)
        VALUES (?, ?, ?, ?)")
    .bind(user_id)
    .bind(&auth_code)
    .bind(created_timestamp)
    .bind(expires_timestamp)
    .execute(pool).await.map_err(|e| {
        eprintln!("Failed to save auth_code to database: {:?}", e);
        anyhow!("Could not save auth_code to database: {e}")
    })?;

    // Return the auth_token, because now it's safe to use (saved to DB)
    Ok(auth_code)
 }


// Add new user to database
pub async fn add_user(
    pool: &MySqlPool,
    username: &String,
    email: &String,
    password: String,
    has_agreed_terms: bool
) -> Result<i32, anyhow::Error> {
    let password_hash: String = auth::hash_password(password);
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "INSERT INTO users (
            username,
            email,
            password_hash,
            has_agreed_terms)
        VALUES (?, ?, ?, ?)")
    .bind(username)
    .bind(email)
    .bind(&password_hash)
    .bind(has_agreed_terms)
    .execute(pool).await.map_err(|e| {
        eprintln!("Failed to save user to database: {:?}", e);
        anyhow!("Could not save user to database: {e}")
    })?;

    Ok(result.last_insert_id() as i32)
}


/**
 * When the server starts up we make sure there is an admin.
 * Their default pre-hashed password is saved in an env variable.
 */
pub async fn create_primary_admin(pool: &MySqlPool) -> Result<bool, anyhow::Error> {
    // If admin already exists, print their name and return false.

    let count_option: Option<Count> = match sqlx::query_as!(
        Count,
        "SELECT COUNT(*) as count FROM users WHERE role = ?",
        "admin"
    ).fetch_optional(pool).await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to fetch admin user count from DB: {:?}", e);
            return Err(anyhow!("Could not fetch admin count: {e}"));
        }
    };

    let count: i64 = count_option.unwrap_or(Count{count: 0}).count;
    if count > 0 {
        println!("Admin already exists.");
        return Ok(false);
    }

    // Admin does NOT exist (if we reached this point in the function)
    // Time to create the admin
    let default_pw: String = std::env::var("ADMIN_PW")?;

    let username: &str = "pattmayne";
    let email: &str = "pattmayne@protonmail.com";
    let role: &str = "admin";
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
            "INSERT INTO users (
                username,
                email,
                role,
                password_hash)
            VALUES (?, ?, ?, ?)")
        .bind(username)
        .bind(email)
        .bind(role)
        .bind(&default_pw)
        .execute(pool).await.map_err(|e| {
            eprintln!("Failed to save FIRST ADMIN user to database: {:?}", e);
            anyhow!("Could not save FIRST ADMIN user to database: {e}")
        })?;

    let new_rows_count: u64 = result.rows_affected();
    Ok(new_rows_count > 0)
}



/**
 * Add a new post to the blog_post table in the database.
 */
pub async fn add_post(
    pool: &MySqlPool,
    post_title: &String,
    post_body: &String,
    cats_string: &String,
    author_name: String,
    pinned: bool,
    pinned_to_blog: bool
) -> Result<u64, anyhow::Error> {
    // We trust that the data has already been checked.
    // We simply enter it like obedient robots now.
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "INSERT INTO blog_post (
            title, body, author_name, pinned, pinned_to_blog
        ) VALUES (?, ?, ?, ?, ?)")
        .bind(post_title)
        .bind(post_body)
        .bind(author_name)
        .bind(pinned)
        .bind(pinned_to_blog)
        .execute(pool).await.map_err(|e| {
            eprintln!("Failed to save NEW POST to database: {:?}", e);
            anyhow!("Could not save NEW POST to database: {e}")
        })?;
    
    let post_id: u64 = result.last_insert_id();
    attach_categories_to_post(pool, post_id as i64, cats_string).await?;
    Ok(post_id)
}


/**
 * Add a new book to the book table in the database.
 */
pub async fn add_book(
    pool: &MySqlPool,
    title: &String,
    author: &String,
    publisher: &String,
    release_year: u16,
    price: f32,
    book_type: &String,
    description: &String,
    slug: &String
) -> Result<u64, anyhow::Error> {

    // We trust that the data has already been checked. We simply enter it like obedient robots now.
    // genres gets added with a different call, after creating this book entry,
    // because of the many-to-many relationship between books and genres.
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "INSERT INTO books (
            title, author, publisher, release_year, price, book_type, description, slug
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(title)
        .bind(author)
        .bind(publisher)
        .bind(release_year)
        .bind(price)
        .bind(book_type)
        .bind(description)
        .bind(slug)
        .execute(pool).await.map_err(|e| {
            eprintln!("Failed to save NEW BOOK to database: {:?}", e);
            anyhow!("Could not save NEW BOOK to database: {e}")
        })?;
    
    Ok(result.last_insert_id())
}


/**
 * Add a new verification code to the DB.
 */
pub async fn create_verification_code(
    pool: &MySqlPool,
    new_code: &auth::NewVerificationCode
) -> Result<u64, anyhow::Error> {

    // We trust that the data has already been checked. We simply enter it like obedient robots now.
    // Except that we will turn the bool into an int.
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "INSERT INTO verification_codes (
            user_id, code_hash, attempts, created_timestamp, expires_timestamp
        ) VALUES (?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            code_hash = VALUES(code_hash),
            created_timestamp = VALUES(created_timestamp),
            expires_timestamp = VALUES(expires_timestamp),
            attempts = VALUES(attempts);")
        .bind(new_code.user_id)
        .bind(new_code.code_hash.to_owned())
        .bind(0)
        .bind(new_code.created_timestamp)
        .bind(new_code.expires_timestamp)
        .execute(pool).await.map_err(|e| {
            eprintln!("Failed to save NEW VERIFICATION CODE to database: {:?}", e);
            anyhow!("Could not save NEW VERIFICATION CODE to database: {e}")
        })?;
    
    Ok(result.rows_affected())
}


/**
 * When a post is created or edited we need to add the categories
 * in one big transaction.
 */
pub async fn attach_categories_to_post(
    pool: &MySqlPool,
    post_id: i64,
    cats_string: &str,
) -> Result<(), sqlx::Error> {

    let mut tx: sqlx::Transaction<'_, sqlx::MySql> = pool.begin().await?;
    let category_names: Vec<String> = cats_string
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    // Remove existing category relationships first.
    // User (me) might have deleted some from the text field.
    sqlx::query(
        r#"
        DELETE FROM post_categories
        WHERE post_id = ?
        "#
    )
    .bind(post_id)
    .execute(&mut *tx)
    .await?;

    for category_name in category_names {
        // Insert category if it doesn't exist.
        // If it DOES exist, retrieve existing id.
        let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
            r#"
            INSERT INTO categories (name)
            VALUES (?)
            ON DUPLICATE KEY UPDATE
                id = LAST_INSERT_ID(id)
            "#
        )
        .bind(&category_name)
        .execute(&mut *tx)
        .await?;

        // Will get the new category id or existing id
        let category_id: u64 = result.last_insert_id();

        // Create relationship row
        // INSERT IGNORE prevents duplicate relationships
        sqlx::query(
            r#"
            INSERT IGNORE INTO post_categories
                (post_id, category_id)
            VALUES (?, ?)
            "#
        )
        .bind(post_id)
        .bind(category_id as i64)
        .execute(&mut *tx)
        .await?;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(())
}


/* 
 * 
 * 
 * 
 * 
 * 
 * 
 * ==============================
 * ==============================
 * =====                    =====
 * =====  UPDATE FUNCTIONS  =====
 * =====                    =====
 * ==============================
 * ==============================
 * 
 * 
 * 
 * update existing entries in the DB
 * 
 * 
 * 
 * 
*/

pub async fn increment_verification_attempt(
    pool: &MySqlPool,
    user_id: i32,
    code_obj_option: Option<auth::HashedVerificationCode>
) -> Result<auth::HashedVerificationCode, anyhow::Error> {

    // first get the current attempts
    let code_obj: auth::HashedVerificationCode =
        match code_obj_option {
            Some(code_obj) => code_obj,
            None => match get_verification_code(pool, user_id).await? {
                Some(code_obj) => code_obj,
                None => return Err(anyhow!("No code found in DB".to_string()))
            }
        };        

    let attempts: i32 = code_obj.attempts;
    let incr_attemtps: i32 = attempts + 1;

    let _result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "UPDATE verification_codes SET attempts = ? WHERE user_id = ?")
        .bind(incr_attemtps)
        .bind(user_id)
        .execute(pool)
        .await?;
    
    let updated: auth::HashedVerificationCode = match get_verification_code(pool, user_id).await? {
        Some(code_obj) => code_obj,
        None => return Err(anyhow!("No code found in DB".to_string()))
    };

    Ok(updated)    
}




pub async fn update_real_names(
    pool: &MySqlPool,
    first_name: &String,
    last_name: &String,
    id: i32
)-> Result<i32, anyhow::Error> {
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "UPDATE users SET first_name = ?, last_name = ? WHERE id = ?")
        .bind(first_name)
        .bind(last_name)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as i32)
}


pub async fn verify_user(pool: &MySqlPool, id: i32)-> Result<i32, anyhow::Error> {
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "UPDATE users SET email_verified = ? WHERE id = ?")
        .bind(1)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as i32)
}


/**
 * Update blog post
 */
pub async fn update_post(
    pool: &MySqlPool,
    post_id: i64,
    post_title: &String,
    post_body: &String,
    cats_string: &String,
    pinned: bool,
    pinned_to_blog: bool
) -> Result<i32, anyhow::Error> {
    let update_time: OffsetDateTime = OffsetDateTime::now_utc();
    let pinned_i8: i8 = if pinned { 1 } else { 0 };
    let pinned_to_blog_i8: i8 = if pinned_to_blog { 1 } else { 0 };

    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "UPDATE blog_post 
            SET title = ?, body = ?, updated_timestamp = ? ,
            pinned = ?, pinned_to_blog = ?
            WHERE id = ?")
        .bind(post_title)
        .bind(post_body)
        .bind(update_time)
        .bind(pinned_i8)
        .bind(pinned_to_blog_i8)
        .bind(post_id)
        .execute(pool)
        .await?;

    attach_categories_to_post(pool, post_id, cats_string).await?;
    Ok(result.rows_affected() as i32)
}


/**
 * User is updating password.
 * Route has already confirmed that it's an acceptable password.
 * Hash it and save it to the database.
 */
pub async fn update_password(
    pool: &MySqlPool,
    password: &String,
    id: i32
)-> Result<i32, anyhow::Error> {

    // save password to DB and return positive result
    let hashed_password: String = auth::hash_password(password.to_owned());
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(hashed_password)
            .bind(id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() as i32)
}


/* 
 * 
 * 
 * 
 * 
 * 
 * 
 * ==============================
 * ==============================
 * =====                    =====
 * =====  DELETE FUNCTIONS  =====
 * =====                    =====
 * ==============================
 * ==============================
 * 
 * 
 * 
 * 
 * 
 * 
 */


/**
 * When a user logs out of a site, delete all their refresh tokens
 */
pub async fn delete_refresh_token(
    pool: &MySqlPool,
    user_id: i32
) -> Result<i32, anyhow::Error> {
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "DELETE FROM refresh_tokens WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() as i32)
}


pub async fn delete_post(
    pool: &MySqlPool,
    post_id: i32
) -> Result<bool> {
    // Delete related post_categories first
    sqlx::query(
        "DELETE FROM post_categories WHERE post_id = ?"
    )
    .bind(post_id)
    .execute(pool)
    .await?;

    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "DELETE FROM blog_post WHERE id = ?")
        .bind(post_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
}


/* 
 * 
 * 
 * 
 * 
 * 
 * ==============================
 * ==============================
 * =====                    =====
 * =====  HELPER FUNCTIONS  =====
 * =====                    =====
 * ==============================
 * ==============================
 * 
 * 
 * 
 * Functions which facilitate the processing of the above DB functions
 * 
 * 
 * 
 * 
 */



// Pre-check for duplicates
// TO DO: errors cannot return false. We haven't confirmed the values are unique.
// ACTUALLY don't bother. Instead, deal with broken pools when doing the actual insert.

// check if username already exists in DB
pub async fn username_taken(pool: &MySqlPool, username: &String) -> bool {
    let count_option: Option<Count> = match sqlx::query_as!(
        Count,
        "SELECT COUNT(*) as count FROM users WHERE username = ?",
        username
    ).fetch_optional(pool).await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to fetch count from DB: {:?}", e);
            return false;
        }
    };

    let count: i64 = count_option.unwrap_or(Count{count: 0}).count;
    count > 0
}

// Check if email address already exists in DB
pub async fn email_taken(pool: &MySqlPool, email: &String) -> bool {
    let count_option: Option<Count> = match sqlx::query_as!(
        Count,
        "SELECT COUNT(*) as count FROM users WHERE email = ?",
        email
    ).fetch_optional(pool).await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to fetch count from DB: {:?}", e);
            return false;
        }
    };

    let count: i64 = count_option.unwrap_or(Count{count: 0}).count;
    count > 0
}