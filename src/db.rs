// I'm actually using MariaDB which is supposedly a drop-in replacement for MySQL

use sqlx::{ MySqlPool };
use time::{ OffsetDateTime, Duration, macros::format_description };
use anyhow::{ Result, anyhow };
use serde;


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
    Player,
}

#[derive(Debug)]
struct Count {
    count: i64,
}



#[derive(serde::Serialize)]
pub struct AuthCodeData {
    pub id: i32,
    pub user_id: i32,
    pub client_id: String,
    pub code: String,
    pub expires_timestamp: OffsetDateTime,
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


#[derive(serde::Serialize)]
pub struct BlogPost {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub author_name: String,
    pub pinned: i8,
    pub created_timestamp: OffsetDateTime,
    pub updated_timestamp: OffsetDateTime,
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
    client_id: String,
    token: String,
    created_timestamp: OffsetDateTime,
    expires_timestamp: OffsetDateTime
}


pub struct ClientLinkData {
    pub domain: String,
    pub redirect_uri: String,
    pub logo_url: String,
    pub name: String,
    pub description: String,
    pub client_id: String,
}


/**
 * When you UPDATE existing client site data
 */
pub struct UpdateClientData {
    pub site_domain: String,
    pub site_name: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub logo_url: String,
    pub description: String,
    pub category: String,
    pub client_type: String,
    pub is_active: bool,
}

pub struct UpdateClientSecret {
    pub hashed_client_secret: String,
}


pub struct ClientSecret {
    pub hashed_client_secret: String,
}

/**
 * When you ENTER client site data for the first time
 */
pub struct NewClientData {
    pub site_domain: String,
    pub site_name: String,
    pub hashed_client_secret: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub logo_url: String,
    pub description: String,
    pub category: String,
    pub client_type: String,
    pub is_active: bool,
}

/**
 * When you GET the client site data to use
 */
pub struct ClientData {
    pub id: i32,
    pub domain: String,
    pub name: String,
    pub hashed_client_secret: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub logo_url: String,
    pub description: String,
    pub category: String,
    pub client_type: String,
    is_active: i8,
    is_internal: i8,
    pub created_timestamp: OffsetDateTime,
}


/* A container to satisfy sqlx's insatiable lust for structs.
 * For when we need to get a list of all the client_ids from
 * the client_sites table.
*/
pub struct ClientRef {
    pub client_id: String,
    pub name: String,
    pub logo_url: String,
    is_active: i8,
    is_internal: i8,
}


#[derive(serde::Serialize)]
pub struct RedirectUri {
    pub redirect_uri: String,
}


impl ClientData {
    pub fn get_is_active(&self) -> bool { self.is_active == 1 }
    pub fn get_is_internal(&self) -> bool { self.is_internal == 1 }
}


impl ClientRef {
    pub fn get_is_active(&self) -> bool { self.is_active == 1 }
    pub fn get_is_internal(&self) -> bool { self.is_internal == 1 }
}



impl BlogPost {
    pub fn get_formatted_updated_timestamp(&self) -> String {
        let format: &[time::format_description::BorrowedFormatItem<'_>] =
            format_description!("[year]-[month]-[day] [hour]:[minute]");
        self.updated_timestamp.format(&format).unwrap()
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned == 1
    }
}


impl RefreshToken {
    pub fn get_token(&self) -> &String { &self.token }
    pub fn get_client_id(&self) -> &String { &self.client_id }
    pub fn get_user_id(&self) -> i32 { self.user_id }
    pub fn get_created_timestamp(&self) -> &OffsetDateTime { &self.created_timestamp }
    pub fn get_expires_timestamp(&self) -> &OffsetDateTime { &self.expires_timestamp }

    pub fn is_expired(&self) -> bool {
        self.expires_timestamp < OffsetDateTime::now_utc()
    }
}

impl AuthCodeData {
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


pub async fn get_auth_code_data(
    pool: &MySqlPool,
    code: &String
) -> Result<Option<AuthCodeData>> {
    Ok(sqlx::query_as!(
            AuthCodeData,
            "SELECT id, user_id, client_id, code, expires_timestamp
            FROM auth_codes WHERE code = ?",
            code
        ).fetch_optional(pool).await?)
}


pub async fn get_post_by_id(
    pool: &MySqlPool,
    post_id: i64
) -> Result<Option<BlogPost>> {
    Ok(sqlx::query_as!(
            BlogPost,
            "SELECT id, author_name, title, body,
            created_timestamp, updated_timestamp, pinned
            FROM dev_blog WHERE id = ?",
            post_id
        ).fetch_optional(pool).await?)
}


pub async fn get_latest_pinned_post(
    pool: &MySqlPool
) -> Result<Option<BlogPost>> {
    Ok(sqlx::query_as!(
            BlogPost,
            "SELECT id, author_name, title, body,
            created_timestamp, updated_timestamp, pinned 
            FROM dev_blog WHERE pinned = ? ORDER BY created_timestamp DESC LIMIT 1",
            true
        ).fetch_optional(pool).await?)
}


pub async fn get_posts(
    pool: &MySqlPool
) -> Result<Vec<BlogPost>> {
    let blog_posts: Vec<BlogPost> = sqlx::query_as!(
        BlogPost,
        "SELECT id, author_name, title, body, created_timestamp, 
        updated_timestamp, pinned
        FROM dev_blog ORDER BY created_timestamp ASC"
    ).fetch_all(pool).await?;

    Ok(blog_posts)
}


pub async fn get_non_pinned_posts(
    pool: &MySqlPool
) -> Result<Vec<BlogPost>> {
    let blog_posts: Vec<BlogPost> = sqlx::query_as!(
        BlogPost,
        "SELECT id, author_name, title, body, created_timestamp, 
        updated_timestamp, pinned
        FROM dev_blog WHERE pinned = ? 
        ORDER BY created_timestamp ASC",
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


pub async fn get_redirect_uri(
    pool: &MySqlPool,
    client_id: &String
) -> Result<Option<String>> {
    let redirect_option: Option<RedirectUri> = sqlx::query_as!(
            RedirectUri,
            "SELECT redirect_uri FROM client_sites WHERE client_id = ?",
            client_id
        ).fetch_optional(pool).await?;
    
    match redirect_option {
        Some(redirect_obj) => {
            Ok(Some(redirect_obj.redirect_uri))
        },
        None => Ok(None)
    }
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
 * Get a refresh token for specified user and client site.
 */
pub async fn get_refresh_token(
    pool: &MySqlPool,
    user_id: i32,
    client_id: String
) -> Result<Option<RefreshToken>> {
    Ok(sqlx::query_as!(
        RefreshToken,
        "SELECT id, user_id, client_id,
            token, created_timestamp, expires_timestamp
            FROM refresh_tokens WHERE user_id = ? AND client_id = ?",
        user_id, client_id
    ).fetch_optional(pool).await?)
}

/**
 * Get a collection of all the client_ids and names in the client_sites table.
 * These are references for the sake of lists, where the client_id can also
 * provide a handle for a link to an edit page (or whatever)
 */
pub async fn get_client_refs(pool: &MySqlPool) -> Result<Vec<ClientRef>> {
    let client_refs: Vec<ClientRef> = sqlx::query_as!(
        ClientRef,
        "SELECT client_id, name, logo_url, is_active, is_internal 
        FROM client_sites ORDER BY is_active DESC"
    ).fetch_all(pool).await?;

    Ok(client_refs)
}


/**
 * Get a collection of link data (name, domain, description, logo) for sites
 * that are ACTIVE and are NOT the auth site.
 */
pub async fn get_client_links(pool: &MySqlPool) -> Result<Vec<ClientLinkData>> {
    let client_refs: Vec<ClientLinkData> = sqlx::query_as!(
        ClientLinkData,
        "SELECT name, logo_url, domain, description, redirect_uri, client_id FROM client_sites 
        WHERE is_active = 1 AND is_internal != 1"
    ).fetch_all(pool).await?;

    Ok(client_refs)
}



pub async fn get_client_by_client_id(
    pool: &MySqlPool,
    client_id: &String
) -> Result<Option<ClientData>> {
    Ok(sqlx::query_as!(
        ClientData,
        "SELECT id, client_id, hashed_client_secret,
            name, domain, redirect_uri,
            description, category, logo_url, is_active,
            client_type, is_internal, created_timestamp
            FROM client_sites WHERE client_id = ?",
        client_id
    ).fetch_optional(pool).await?)
}


pub async fn get_client_secret(
    pool: &MySqlPool,
    client_id: &String
) -> Result<Option<ClientSecret>> {
    Ok(sqlx::query_as!(
        ClientSecret,
        "SELECT hashed_client_secret
            FROM client_sites WHERE client_id = ?",
        client_id
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
  * Add a refresh token to the database.
  * for a particular user and particular client site.
  * Take ownership of token, because it should ONLY be given back
  * if it's saved successfully to the DB.
  */
 pub async fn add_refresh_token(
    pool: &MySqlPool,
    user_id: i32,
    client_id: String,
    refresh_token: String
) -> Result<String, anyhow::Error> {
    let expires_timestamp: OffsetDateTime =
        OffsetDateTime::now_utc() + Duration::days(14); // TODO: put this in resources?
    let created_timestamp: OffsetDateTime = OffsetDateTime::now_utc();

    let _result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "INSERT INTO refresh_tokens (
            user_id,
            client_id,
            token,
            created_timestamp,
            expires_timestamp)
        VALUES (?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            token = VALUES(token),
            created_timestamp = VALUES(created_timestamp),
            expires_timestamp = VALUES(expires_timestamp);
            ")
    .bind(user_id)
    .bind(client_id)
    .bind(&refresh_token)
    .bind(created_timestamp)
    .bind(expires_timestamp)
    .execute(pool).await.map_err(|e| {
        eprintln!("Failed to save refresh_token to database: {:?}", e);
        anyhow!("Could not save refresh_token to database: {e}")
    })?;

    // Return the refresh_token, because now it's safe to use (saved to DB)
    Ok(refresh_token)
 }


 
 /**
  * Add a auth token to the database.
  * for a particular user and particular client site.
  * Take ownership of token, because it should ONLY be given back
  * if it's saved successfully to the DB.
  */
 pub async fn add_auth_code(
    pool: &MySqlPool,
    user_id: i32,
    client_id: &String,
    auth_code: String
) -> Result<String, anyhow::Error> {
    let expires_timestamp: OffsetDateTime =
        OffsetDateTime::now_utc() + Duration::minutes(1);
    let created_timestamp: OffsetDateTime = OffsetDateTime::now_utc();

    let _result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "INSERT INTO auth_codes (
            user_id,
            client_id,
            code,
            created_timestamp,
            expires_timestamp)
        VALUES (?, ?, ?, ?, ?)")
    .bind(user_id)
    .bind(client_id)
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
 * When the server starts up we make sure the auth site (this site)
 * exists as a client_site in the DB.
 */
pub async fn create_self_client(pool: &MySqlPool) -> Result<bool, anyhow::Error> {
    let domain: String = std::env::var("AUTH_DOMAIN")?;
    // If site already exists, print their name and return false.

    let count_option: Option<Count> = match sqlx::query_as!(
        Count,
        "SELECT COUNT(*) as count FROM client_sites WHERE domain = ?",
        &domain
    ).fetch_optional(pool).await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to fetch client_sites count from DB: {:?}", e);
            return Err(anyhow!("Could not fetch auth client_sites count: {e}"));
        }
    };

    let count: i64 = count_option.unwrap_or(Count{count: 0}).count;
    if count > 0 {
        println!("Auth client_site already exists.");
        return Ok(false);
    }

    // Auth site does NOT already exist (if we reached this far in the function)
    // Create auth site
    let client_id: String = utils::auth_client_id();
    let client_secret: &str = "CLIENT_SECRET_PLACEHOLDER";
    let name: &str = "Auth Site";
    let redirect_uri: &str = "127.0.0.1:8080/dashboard";
    let client_type: &str = "confidential";
    let category: &str = "service";
    let is_internal: bool = true;


    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "INSERT INTO client_sites (
            client_id,
            hashed_client_secret,
            name,
            domain,
            redirect_uri,
            client_type,
            category,
            is_internal
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(client_id)
        .bind(client_secret)
        .bind(name)
        .bind(domain)
        .bind(redirect_uri)
        .bind(client_type)
        .bind(category)
        .bind(is_internal)
        .execute(pool).await.map_err(|e| {
            eprintln!("Failed to save FIRST AUTH client to database: {:?}", e);
            anyhow!("Could not save FIRST AUTH client to database: {e}")
        })?;

    Ok(result.rows_affected() > 0)
}



/**
 * When the server starts up we make sure the auth site (this site)
 * exists as a client_site in the DB.
 */
pub async fn add_external_client(
    pool: &MySqlPool,
    new_client_data: NewClientData
) -> Result<u64, anyhow::Error> {
    println!("In the DB to add a NEW CLIENT SITE!");

    // We trust that the data has already been checked. We simply enter it like obedient robots now.
    // Except that we will turn the bool into an int.
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "INSERT INTO client_sites (
            client_id,
            hashed_client_secret,
            name,
            domain,
            redirect_uri,
            logo_url,
            client_type,
            description,
            category,
            is_internal,
            is_active
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(new_client_data.client_id)
        .bind(new_client_data.hashed_client_secret)
        .bind(new_client_data.site_name)
        .bind(new_client_data.site_domain)
        .bind(new_client_data.redirect_uri)
        .bind(new_client_data.logo_url)
        .bind(new_client_data.client_type)
        .bind(new_client_data.description)
        .bind(new_client_data.category)
        .bind(0)
        .bind(new_client_data.is_active)
        .execute(pool).await.map_err(|e| {
            eprintln!("Failed to save EXTERNAL CLIENT to database: {:?}", e);
            anyhow!("Could not save EXTERNAL CLIENT to database: {e}")
        })?;
    
    Ok(result.rows_affected())
}


/**
 * Add a new post to the dev_blog
 */
pub async fn add_post(
    pool: &MySqlPool,
    post_title: &String,
    post_body: &String,
    author_name: String,
    pinned: bool
) -> Result<u64, anyhow::Error> {

    // We trust that the data has already been checked. We simply enter it like obedient robots now.
    // Except that we will turn the bool into an int.
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "INSERT INTO dev_blog (
            title, body, author_name, pinned
        ) VALUES (?, ?, ?, ?)")
        .bind(post_title)
        .bind(post_body)
        .bind(author_name)
        .bind(pinned)
        .execute(pool).await.map_err(|e| {
            eprintln!("Failed to save NEW POST to database: {:?}", e);
            anyhow!("Could not save NEW POST to database: {e}")
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




pub async fn update_external_client(
    pool: &MySqlPool,
    update_client_data: UpdateClientData
) -> Result<i32, anyhow::Error> {
    println!("Updating client in the database.");

    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
    "UPDATE client_sites SET name = ?, domain = ?, redirect_uri = ?,
            description = ?, logo_url = ?, is_active = ?,
            client_type = ?, category = ? WHERE client_id = ?")
        .bind(update_client_data.site_name)
        .bind(update_client_data.site_domain)
        .bind(update_client_data.redirect_uri)
        .bind(update_client_data.description)
        .bind(update_client_data.logo_url)
        .bind(update_client_data.is_active)
        .bind(update_client_data.client_type)
        .bind(update_client_data.category)
        .bind(update_client_data.client_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as i32)
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
    pinned: bool
) -> Result<i32, anyhow::Error> {
    let update_time: OffsetDateTime = OffsetDateTime::now_utc();
    let pinned_i8: i8 = if pinned { 1 } else { 0 };

    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "UPDATE dev_blog 
            SET title = ?, body = ?, updated_timestamp = ? ,
            pinned = ?
            WHERE id = ?")
        .bind(post_title)
        .bind(post_body)
        .bind(update_time)
        .bind(pinned_i8)
        .bind(post_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as i32)
}



/**
 * update just the "client_secret" field of a registered client site.
 */
pub async fn update_client_secret(
    pool: &MySqlPool,
    client_id: &String,
    hashed_client_secret: &String
) -> Result<i32, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE client_sites SET hashed_client_secret = ? WHERE client_id = ?")
            .bind(hashed_client_secret)
            .bind(client_id)
            .execute(pool)
            .await?;

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
    let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
        "DELETE FROM dev_blog WHERE id = ?")
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