#![allow(dead_code)] // dead code come on I'm just not using the fields yet.

use actix_web::{ App, HttpServer, middleware::{from_fn}, web };
use actix_files::Files;
use dotenvy;
use sqlx::{ MySqlPool };
use std::io;

// Local mods (they can use each other as crates instead of mods)
mod routes;
mod db;
mod utils;
mod auth;
mod middleware;
mod resources;
mod resource_mgr;
mod routes_utils;
mod email;


/**
 * The main function logs all the routes as routes or "services".
 * A service route takes a route function which has used a macro to declare
 * its path.
 * Other routes ( .route) take a path and then a function to call when the path
 * is requested.
 * 
 * Add middleware at the point in the chain where its changes will become needed.
 * If you add middleware before static it will be called multiple times (bad, don't do).
 * Add it too late and its changes won't be available where needed.
 */
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // dotenvy loads env variables for whole app
    // after this, just call std::env::var(variable_name)
    dotenvy::dotenv().ok();

    // Create the database pool that every function will use
    let pool: MySqlPool = match create_pool().await {
        Ok(pool) => pool,
        Err(_e) => return database_pool_err().await
    };

    db_first_entries(&pool).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(Files::new("/static", "./static"))
            .service(Files::new("/uploads", "./uploads"))
            .wrap(from_fn(middleware::login_status_middleware))
            .service(routes::home)
            .service(routes::blog)
            .service(routes::dashboard_page)
            .service(routes::error_root)
            .service(routes::error_root_2)
            .service(routes::error_page)
            .service(routes::verify)
            .service(routes::verify_post)
            .service(routes::req_new_code)
            .service(
                web::scope("/auth")
                    .route("/login", web::get().to(routes::login_page))
                    .route("/register", web::get().to(routes::register_page))
                    .route("/request_verification", web::get().to(routes::request_verification_page))
                    .route("/", web::get().to(routes::auth_home))
                    .route("", web::get().to(routes::auth_home))
                    .service(routes::login_post)
                    .service(routes::register_post)
                    .service(routes::logout_post)
                    .service(routes::update_names)
                    .service(routes::update_password)
            )
            .service(
                web::scope("/admin")
                    .route("/", web::get().to(routes::admin_redirect))
                    .route("", web::get().to(routes::admin_redirect))
                    .route("/dashboard", web::get().to(routes::admin_home))
                    .service(routes::new_post_page)
                    .service(routes::new_blog_post) // post data to create new blog post
                    .service(routes::edit_post_page)
                    .service(routes::update_blog_post)
                    .service(routes::delete_blog_post)
            )
            .default_service(web::get().to(routes::not_found)) // <- catch-all
            .wrap(from_fn(middleware::jwt_cookie_middleware))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/**
 * When the server first starts, make sure admin exists in users.
 */
async fn db_first_entries(pool: &MySqlPool) {
    // add the admin user if they don't exist
    match db::create_primary_admin(pool).await {
        Ok(user_created) => {
            if user_created {
                println!("New admin created.");
            } else {
                println!("Admin already exists.")
            }            
        },
        Err(e) => {
            eprintln!("DB Error: {e}");
        }
    };
}


async fn database_pool_err() -> std::io::Result<()> {
    eprintln!("ERROR: NO HASH ID SECRET.");
    return Err(
        io::Error::new(
            io::ErrorKind::Other, "HASHID_SECRET not set")
    );
}


/**
 * Create the database thread pool that every function will use
*/
async fn create_pool() -> Result<MySqlPool, String> {
    let database_url: String = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_e) => return Err("Database Error".to_string())
    };

    let pool = match MySqlPool::connect(database_url.as_str()).await {
        Ok(pool) => pool,
        Err(_e) => return Err("Database Error".to_string())
    };
    
    Ok(pool)
}
