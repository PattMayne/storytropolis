/* 
 * ====================
 * ====================
 * =====          =====
 * =====  ROUTES  =====
 * =====          =====
 * ====================
 * ====================
 * 
 * 
 * 
 * Functions to be called when user request hits endpoints listed
 * in the main function.
 * 
 * 
 * 
 */

use actix_web::{
    web, HttpResponse, HttpRequest,
    Responder, http::StatusCode, http::header,
    get, post, web::Redirect };
use actix_web::cookie::{ Cookie };
use askama::Template;
use sqlx::{ MySqlPool };

use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::fs;
use std::io::Write;

use crate::db::{ UnifiedPost, get_active_categories,
    get_categories_by_post_id, get_unified_post };
use crate::utils::vec_to_string;
// local modules, loaded as crates (declared as mods in main.rs)
use crate::{
    resources::get_translation,
    db, utils, auth,
    resource_mgr::{
        HomeTexts, LoginTexts, RegisterTexts, AdminTexts, VerifyTexts,
        AgreementTexts, BlogTexts, EditPostTexts, NewPostTexts, UploadTexts,
        ErrorTexts, DashboardTexts, NewBookTexts, PostTexts,
        ReqVerificationTexts, ErrorData
     },
    routes_utils::{*}
};


/*
 * 
 * 
 * 
 * 
 * 
 * 
 * =========================
 * =========================
 * =====               =====
 * =====  POST ROUTES  =====
 * =====               =====
 * =========================
 * =========================
 * 
 * 
 * 
 * 
 * 
 * 
 */


/**
 * When somebody has visited the page to verify their email,
 * that page calls this API to do the actual verification.
 */
#[post("verify_post")]
pub async fn verify_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    info: web::Json<VerifyQuery>
) -> HttpResponse {
    // If info is missing, get out and signal user
    if info.code.is_none() || info.email.is_none() {
        return error_post_response(&req, 422)
    }

    let verify_code: String = info.code.to_owned().unwrap();
    let email: String = info.email.to_owned().unwrap();

    /*  check the code
        either log the user in (and redirect to dashboard) (and update VERIFIED in DB)
        ...or skip and load the page for manual attempt */

    let user: db::User =
        match db::get_user_by_email(&pool, &email).await {
            Ok(Some(user)) => user,
            Ok(None) => return error_post_response(&req, 404),
            Err(e) => {
                eprintln!("DB Error: {}", e);
                return error_post_response(&req, 500)
            }
        };

    // get the saved, hashed verify code
    let code_hash_obj: auth::HashedVerificationCode =
        match db::get_verification_code(&pool, user.get_id()).await {
            Ok(Some(code_hash_obj)) => code_hash_obj,
            Ok(None) => return error_post_response(&req, 404),
            Err(e) => {
                eprintln!("DB Error: {}", e);
                return error_post_response(&req, 500)
            }
        };
    
    if code_hash_obj.has_exceeded_attempts() {
        let message: String = "Please wait two minutes before requesting a new code.".to_string();
        let error_struct: ErrorResponse = ErrorResponse { error: message, code: 429 };
        return HttpResponse::TooManyRequests().json(error_struct)
    } else if code_hash_obj.is_expired() {
        let message: String = "Your code has expired. You may request a new code.".to_string();
        let error_struct: ErrorResponse = ErrorResponse { error: message, code: 400 };
        return HttpResponse::TooManyRequests().json(error_struct)
    }

    // compare entered verify code to saved, hashed code, and return User if match.
    let code_match: bool = auth::verify_password(&verify_code, &code_hash_obj.code_hash);

    // if match, log user in. Otherwise, send rejection.
    if code_match {
         // verify user email:
        match db::verify_user(&pool, user.get_id()).await {
            Ok(affected_count) => if affected_count > 0 {
                authenticate_user_response(user, pool).await
            } else {
                eprint!("Error verifying user: no rows affected");
                return error_post_response(&req, 500)
            },
            Err(e) => {
                eprint!("Error verifying user: {}", e);
                return error_post_response(&req, 500)
            }
        }
    } else {
        // Wrong code. Increment attempts.
        let _inc_obj_result: Result<auth::HashedVerificationCode, anyhow::Error> =
            db::increment_verification_attempt(
                &pool, user.get_id(), Some(code_hash_obj)
            ).await;
        
        let message: String = "The code does not match. 
            Get the correct code from your email, or request a new one.".to_string();
        let error_struct: ErrorResponse = ErrorResponse { error: message, code: 401 };

        HttpResponse::TooManyRequests().json(error_struct)
    }
   
}



/** REGISTER
 * The register page/form calls this API to register.
 * We get user data, check it against regex for formatting,
 * and against the DB & see if it already exists.
*/
#[post("/register")]
async fn register_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    info: web::Json<RegisterCredentials>
) -> HttpResponse {
    let server_error: HttpResponse = get_server_error(&req).await;

    // first check if a bot filled in the "website" field (which is supposed to be empty)
    if !info.website.is_empty() {
        return server_error
    }

    // check credentials against regex and size ranges
    let username_valid: bool = utils::validate_username(&info.username);
    let email_valid: bool = utils::validate_email(&info.email);
    let password_valid: bool = utils::validate_password(&info.password);
    let credentials_are_ok: bool = username_valid && email_valid && password_valid;

    if !credentials_are_ok {
        // One of the fields doesn't match the regex
        let bad_creds_data: BadRegistrationInputs = BadRegistrationInputs {
            email_valid,
            username_valid,
            password_valid,
            code: 422,
        };

        return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).json(bad_creds_data);
    }

    /* Input credentials are acceptable format.
     * Try to enter them in the database.
     * if username or email already exists, send a 409
     * Do a pre-check first.
    */

    let username_exists: bool = db::username_taken(&pool, &info.username).await;
    let email_exists: bool = db::email_taken(&pool, &info.email).await;
    let username_or_email_already_exists: bool = username_exists || email_exists;

    if username_or_email_already_exists {
        let bad_creds_data: BadRegistrationInputs = BadRegistrationInputs {
            email_valid: !email_exists,
            username_valid: !username_exists,
            password_valid,
            code: 409,
        };

        return HttpResponse::Conflict().json(bad_creds_data);
    }

    // NOW we've done our pre-checks. Time to add User to DATABASE
    // We can still send errors if there's a duplicate or a problem
    
    let user_id_result: Result<i32, anyhow::Error> = db::add_user(
        &pool,
        &info.username,
        &info.email,
        info.password.clone(),
        info.has_agreed_terms
    ).await;

    let user_id: i32 = match user_id_result {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to save user to DB: {:?}", e);
            return server_error;
        }
    };

    // get user object from DB
    let user: db::User =
        match db::get_user_by_id(&pool, user_id).await {
            Ok(Some(user)) => user,
            Ok(None) =>  return server_error,
            Err(_e) => return server_error
        };
    
    // Send verification email.
    let _email_sent: bool = true;
        //email::send_verification_email(&pool, &info.username, user_id, &info.email).await;

    authenticate_user_response(user, pool).await
}


#[post("/req_new_code")]
async fn req_new_code(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    info: web::Json<NewCodeRequest>
) -> HttpResponse {
    // make sure user exists
    // check old code for being expired

    let user: db::User = match db::get_user_by_email(&pool, &info.email).await {
        Ok(Some(user)) => user,
        Ok(None) => return error_post_response(&req, 404),
        Err(_e) => return error_post_response(&req, 500)
    };

    // user exists. See if there's already a code, and if enough time has elapsed
    let existing_code_option: Option<auth::HashedVerificationCode> =
        match db::get_verification_code(&pool, user.get_id()).await {
            Ok(code_option) => code_option,
            Err(_e) => return error_post_response(&req, 500)
        };

    if let Some(code_obj) = existing_code_option {
        if !code_obj.can_request_new() {
            let message: String = "You must wait two minutes before requesting a new code.".to_string();
            return HttpResponse::Ok().json(Message {message})
        }
    }

    // NOW we can finally create a new verification code, and send verification email.
    let email_sent: bool = true;
        //email::send_verification_email(&pool, user.get_username(), user.get_id(), &info.email).await;


    if email_sent {
        let message: String = "New code sent. Check your email.".to_string();
        return HttpResponse::Ok().json(Message { message })
    } else {
        error_post_response(&req, 500)
    }
}


/** LOGIN
 * Get user data, check it against the DB & see if it's right.
*/
#[post("/login")]
async fn login_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    info: web::Json<LoginCredentials>
) -> HttpResponse {
    let server_error: HttpResponse = get_server_error(&req).await;

    // Check for empty fields
    if info.username_or_email.trim().is_empty() || info.password.trim().is_empty() {
        let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
        let error: String = get_translation("err.empty_creds", &user_req_data.lang, None);
        return HttpResponse::Unauthorized().json(
            ErrorResponse {
            error,
            code: 401
        });
    }

    // TRYING TO GET A USER:

    // Find out if pattern matches email (and retrieve use by email), else treat as username (and
    // retrieve by username)
    let user_result: Result<Option<db::User>, anyhow::Error> =
        if utils::validate_email(&info.username_or_email) {
            db::get_user_by_email(&pool, &info.username_or_email).await
        } else {
            db::get_user_by_username(&pool, &info.username_or_email).await
    };

    let user: db::User = match user_result {
        Ok(Some(user)) => {

            // Now check the input password against password from DB
            if auth::verify_password(&info.password, user.get_password_hash()) {
                user
            } else {
                // Auth clearly failed
                let code: u16 = 401;
                let lang: &utils::SupportedLangs = &auth::get_user_req_data(&req).clone_lang();
                let error: String = get_translation(
                    "err.invalid_creds", &lang, None);
                return HttpResponse::Unauthorized().json(ErrorResponse { error, code });
            }
        },
        Ok(None) => {
            let code: u16 = 404;
            let lang: &utils::SupportedLangs = &auth::get_user_req_data(&req).clone_lang();
            let error: String = get_translation(
                "err.user_not_found", &lang, None);
            return HttpResponse::NotFound().json(ErrorResponse { error, code });
        },
        Err(_e) => return server_error
    };

    authenticate_user_response(user, pool).await
}




/**
 * Checks that the user is truly an admin, checks that all the 
 * data is legit, then adds it to the database.
 * Lots of opportunities to send errors.
 */
#[post("/add_post")]
async fn new_blog_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut blog_post_data: web::Json<BlogPostData>
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    // Trim the body string
    blog_post_data.trim_all_strings();

    println!("Categories: {}", blog_post_data.categories);

    // Add the post to the database
    let post_succes_obj: BlogPostSuccess = match db::add_post(
        &pool,
        &blog_post_data.post_title,
        &blog_post_data.post_body,
        &blog_post_data.categories,
        user_req_data.username.unwrap(),
        blog_post_data.pinned,
        blog_post_data.pinned_to_blog
    ).await {
        Ok(post_id) => {
            BlogPostSuccess {
                success: true,
                message: "Blog post created".to_string(),
                post_id: post_id as i32
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            BlogPostSuccess {
                success: false,
                message: "ERROR: Blog post NOT SAVED".to_string(),
                post_id: 0
            }
        }
    };

    HttpResponse::Ok().json(post_succes_obj)
}


/**
 * Checks that the user is truly an admin, checks that all the 
 * data is legit, then adds it to the database.
 * Lots of opportunities to send errors.
 */
#[post("/add_book")]
async fn new_book(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    let mut title: Option<String> = None;
    let mut author: Option<String> = None;
    let mut publisher: Option<String> = None;
    let mut release_year: Option<u16> = None;
    let mut price: Option<f32> = None;
    let mut book_type: Option<String> = None;
    let mut description: Option<String> = None;
    let mut slug: Option<String> = None;

    while let Some(item) = payload.next().await {
        let field: actix_multipart::Field = item.unwrap();
        let name: &str = field.name().unwrap_or("");

        match name {
            "title" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                title = Some(String::from_utf8(bytes).unwrap());
            },
            "author" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                author = Some(String::from_utf8(bytes).unwrap());
            },
            "publisher" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                publisher = Some(String::from_utf8(bytes).unwrap());
            },
            "release_year" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                let s: String = String::from_utf8(bytes).unwrap();
                release_year = Some(s.parse::<u16>().unwrap());
            },
            "price" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                let s: String = String::from_utf8(bytes).unwrap();
                price = Some(s.parse::<f32>().unwrap());
            },"book_type" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                book_type = Some(String::from_utf8(bytes).unwrap());
            },
            "description" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                description = Some(String::from_utf8(bytes).unwrap());
            },
            "slug" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                slug = Some(String::from_utf8(bytes).unwrap());
            },
            "image" => {
                // handle file upload separately
            }
            _ => {}
        }
    }

    let mut book_data: NewBookData =
        NewBookData {
            title: title.unwrap(),
            author: author.unwrap(),
            publisher: publisher.unwrap(),
            release_year: release_year.unwrap(),
            price: price.unwrap(),
            book_type: book_type.unwrap(),
            description: description.unwrap(),
            slug: slug.unwrap(),
        };

    // cycle through the multi-part form data and extract the fields into a NewBookData struct
    // I need to MAKE the struct definition.
    // I need to SAVE the image to disk and save the path in the struct.
    // I need to save all that data to the DB.
    // I need to save the genres EACH to the genres table
    // Then make the association in the book_genres table
    // Then finally send a success response back to the JS that called this API.

    // check the file upload
    // if book_data.file.is_none() {
    //     return HttpResponse::BadRequest().json(BlogPostSuccess {
    //         success: false,
    //         message: "ERROR: No file uploaded".to_string(),
    //         post_id: 0
    //     });
    // }

    // Trim the body string
    book_data.trim_all_strings();

    // Add the post to the database
    let post_succes_obj: NewBookSuccess = match db::add_book(
        &pool,
        &book_data.title,
        &book_data.author,
        &book_data.publisher,
        book_data.release_year,
        book_data.price,
        &book_data.book_type,
        &book_data.description,
        &book_data.slug
    ).await {
        Ok(post_id) => {
            NewBookSuccess {
                success: true,
                message: "Book created".to_string(),
                book_id: post_id as i32
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            NewBookSuccess {
                success: false,
                message: "ERROR: Book NOT SAVED".to_string(),
                book_id: 0
            }
        }
    };

    HttpResponse::Ok().json(post_succes_obj)
}


/**
 * Checks that the user is truly an admin, checks that all the 
 * data is legit, then adds it to the database.
 * Lots of opportunities to send errors.
 */
#[post("/img_upload_post")]
async fn img_upload_post(
    req: HttpRequest,
    mut payload: Multipart
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    let mut filename: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut original_extension: Option<String> = None;

    // First get the raw data, and the original file extension
    while let Some(item) = payload.next().await {
        let mut field: actix_multipart::Field = item.unwrap();
        let name: &str = field.name().unwrap_or("");

        match name {
            "filename" => {
                let bytes: Vec<u8> = get_bytes_from_field(field).await;
                let string_result: Result<String, std::string::FromUtf8Error> =
                    String::from_utf8(bytes);
                let filename_string: String = match string_result {
                    Ok(fns) => fns,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return HttpResponse::Ok().json(FileUploadSuccess {
                            success: false,
                            message: "Error extracting filename".to_string(),
                            filename: None,
                        })
                    }
                };

                if filename_string == "" {
                    return HttpResponse::Ok().json(FileUploadSuccess {
                        success: false,
                        message: "Please enter a filename".to_string(),
                        filename: None,
                    })
                };

                filename = Some(filename_string);
            }
            "img_upload" => {
                // Get original extension
                if let Some(cd) = field.content_disposition() {
                    if let Some(orig_filename) = cd.get_filename() {
                        original_extension = std::path::Path::new(orig_filename)
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| s.to_owned());
                    }
                }
                // Read file to memory
                let mut bytes: Vec<u8> = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data: web::Bytes = chunk.unwrap();
                    bytes.extend_from_slice(&data);
                }
                file_bytes = Some(bytes);
            }
            _ => {}
        }
    }

    // Now combine and save if both fields exist
    if let (Some(name), Some(bytes)) = (filename, file_bytes) {

        let final_filename: String = if let Some(ext) = original_extension {
            format!("{}.{}", name, ext)
        } else { name };

        let filepath: String = format!("./uploads/{}", final_filename);
        let file_result: Result<fs::File, std::io::Error> = fs::File::create(&filepath);

        let mut file: fs::File = match file_result {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: {}", e);
                return HttpResponse::Ok().json(FileUploadSuccess {
                    success: false,
                    message: "Error creating file".to_string(),
                    filename: Some(final_filename),
                })
            }
        };

        match file.write_all(&bytes) {
            Ok(_) => {
                return HttpResponse::Ok().json(FileUploadSuccess {
                    success: true,
                    message: "File uploaded successfully".to_string(),
                    filename: Some(final_filename),
                })
            }, Err(e) => {
                eprintln!("Error: {}", e);
                return HttpResponse::Ok().json(FileUploadSuccess {
                    success: false,
                    message: "Error writing file".to_string(),
                    filename: Some(final_filename),
                })
            }
        };
    }

    let failure_json: FileUploadSuccess = FileUploadSuccess {
        success: false,
        message: "ERROR: No file uploaded".to_string(),
        filename: None
    };

    HttpResponse::Ok().json(failure_json)
}




#[post("/update_password")]
pub async fn update_password(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    password_obj: web::Json<NewPassword>
) -> HttpResponse {

    // make sure user is logged in
    let user_id: i32 = match auth::get_user_req_data(&req).id {
        Some(id) => id,
        None => return return_authentication_err_json()
    };

    match db::get_user_by_id(&pool, user_id).await {
        Ok(Some(_user)) =>{
            // User is real user
            // check credentials against regex and size ranges
            let password_valid: bool = utils::validate_password(&password_obj.password);

            if !password_valid {
                // One of the fields doesn't match the regex
                let bad_password_data: BadPassword = BadPassword::new(422);
                return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY)
                    .json(bad_password_data);
            }

            // Names are valid. Update the DB
            let update_password_result: Result<i32, anyhow::Error> =
                db::update_password(
                    &pool,
                    &password_obj.password,
                    user_id
                ).await;
            
            match update_password_result {
                Ok(rows_affected) => {
                    return HttpResponse::Ok()
                        .json(UpdateData::new(rows_affected > 0))
                },
                Err(_e) => {
                    return return_internal_err_json();
                }
            }
        },
        Ok(None) => { return return_authentication_err_json(); },
        Err(_e) => {
            return return_internal_err_json();
        }
    };
}



#[post("/update_post")]
pub async fn update_blog_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut blog_post_data: web::Json<BlogPostUpdateData>
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    // Trim the body string
    blog_post_data.trim_all_strings();


    println!("Categories: {}", blog_post_data.categories);

    // Add the post to the database
    let post_succes_obj: BlogPostSuccess = match db::update_post(
        &pool,
        blog_post_data.post_id,
        &blog_post_data.post_title,
        &blog_post_data.post_body,
        &blog_post_data.categories,
        blog_post_data.pinned,
        blog_post_data.pinned_to_blog
    ).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                BlogPostSuccess {
                    success: true,
                    message: "Blog post updated".to_string(),
                    post_id: blog_post_data.post_id as i32
                }
            } else {
                BlogPostSuccess {
                    success: false,
                    message: "Blog post NOT updated".to_string(),
                    post_id: blog_post_data.post_id as i32
                }
            }
        },
        Err(e) => {
            eprintln!("DB ERROR: {}", e);
            BlogPostSuccess {
                success: false,
                message: "DATABASE ERROR: Blog post NOT UPDATED".to_string(),
                    post_id: blog_post_data.post_id as i32
            }
        }
    };

    HttpResponse::Ok().json(post_succes_obj)
}


#[post("/delete_post")]
pub async fn delete_blog_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    blog_post_data: web::Json<DeletePostId>
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    // Add the post to the database
    let post_succes_obj: BlogPostSuccess = match db::delete_post(
        &pool,
        blog_post_data.post_id
    ).await {
        Ok(_rows_affected) => {
            BlogPostSuccess {
                success: true,
                message: "Blog post deleted".to_string(),
                post_id: blog_post_data.post_id
            }
        },
        Err(e) => {
            eprintln!("DB ERROR: {}", e);
            BlogPostSuccess {
                success: false,
                message: "DATABASE ERROR: Blog post NOT UPDATED".to_string(),
                post_id: blog_post_data.post_id
            }
        }
    };

    HttpResponse::Ok().json(post_succes_obj)
}


#[post("/update_names")]
pub async fn update_names(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    names: web::Json<RealNames>
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // make sure user is logged in
    let user_id: i32 = match user_req_data.id {
        Some(id) => id,
        None => return return_authentication_err_json()
    };
    
    // get out if err or none
    let _user: db::User = match db::get_user_by_id(&pool, user_id).await {
        Ok(Some(user)) => user,
        _ => return return_authentication_err_json(),
    };

    // User is real user
    // get json from the req, and names from json
    // check names for length. Send back if too long or short

    // check credentials against regex and size ranges
    let names_valid: bool = utils::validate_real_name(&names.first_name) &&
        utils::validate_real_name(&names.last_name);

    if !names_valid {
        // One of the fields doesn't match the regex
        let bad_names_data: BadNames = BadNames::new(422);
        return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY)
            .json(bad_names_data);
    }

    // Names are valid. Update the DB
    let update_names_result: Result<i32, anyhow::Error> =
        db::update_real_names(
            &pool,
            &names.first_name,
            &names.last_name,
            user_id
    ).await;
    
    match update_names_result {
        Ok(rows_affected) => HttpResponse::Ok().json(UpdateData::new(rows_affected > 0)),
        Err(_e) => return return_internal_err_json()
    }
}


#[post("/logout")]
pub async fn logout_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    let user_id = match user_req_data.id {
        Some(id) => id,
        None => 0
    };

    // delete cookies
    
    let jwt_cookie: Cookie<'_> = Cookie::build("jwt", "")
        .path("/")
        .max_age(time::Duration::seconds(0))
        .http_only(true)
        .finish();

    let refresh_cookie: Cookie<'_> = Cookie::build("refresh_token", "")
        .path("/")
        .max_age(time::Duration::seconds(0))
        .http_only(true)
        .finish();

    // delete refresh_token from DB
    match db::delete_refresh_token(&pool, user_id).await {
        Ok(_rows_deleted) => {},
        Err(e) => {eprint!("Database error: {e}")}
    }

    HttpResponse::Ok()
        .cookie(jwt_cookie)
        .cookie(refresh_cookie)
        .json(LogoutData::new())
}




/* 
 * 
 * 
 * 
 * 
 * 
 * 
 * 
 * 
 * 
 * ========================
 * ========================
 * =====              =====
 * =====  GET ROUTES  =====
 * =====              =====
 * ========================
 * ========================
 * 
 * 
 * 
 * 
 * 
 * 
 * 
 * 
 * 
*/


#[get("/rss")]
pub async fn get_rss(
    pool: web::Data<MySqlPool>,
    req: HttpRequest
) -> HttpResponse {

    // get all non-pinned uposts
    let uposts: Vec<db::UnifiedPost> =
        match get_unified_posts(&pool, false).await {
            Ok(uposts) => uposts,
            Err(e) => {
                eprintln!("Error retrieving posts: {e}");
                return return_error_page(&req, 404)
            }
        };

    // create an rss xml
    let xml: String = get_rss_from_uposts(&req, &uposts).await;

    // send it
    HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(xml)
}


// if user just goes to /auth or /auth/
pub async fn auth_home() -> impl Responder {
    Redirect::to("/auth/login")
}


#[get("/upload_page")]
pub async fn upload_img_page(req: HttpRequest) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    let template: UploadTemplate = UploadTemplate {
        texts: UploadTexts::new(&user_req_data),
        nav_data: NavData::new( "admin".to_string() ),
        user:  user_req_data
    };

    /*
     * TO DO:
     * 
     * Need a TEMPLATE (just with nav, nav texts & user for now)
     * Need a PAGE (just an input field for file upload, and an input for rename)
     * Need a POST FUNCTION to receive the img upload
     * ---- protect the function by requiring admin role!!!
     */

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}

/**
 * The page where the user comes to verify their email address.
 * We assume they received an email with a verification code.
 * The email and code might arrive by querystring,
 * in which case validate, verify, and redirect to dashboard.
 * Otherwise load a template where they can enter the info.
 */
#[get("/verify")]
pub async fn verify(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<VerifyQuery>
) -> HttpResponse {
    // It doesn't matter is user is already logged in. Must still verify email.
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    let mut message: String = "Please enter your email and verification code.".to_string();
    let mut request_new_code: bool = false;

    // Can user be verified by querystring input? If so, no remplate. Verify and redirect
    if query.code.is_some() && query.email.is_some() {
        let verify_code: String = query.code.to_owned().unwrap();
        let email: String = query.email.to_owned().unwrap();

        /*  check the code
            either log the user in (and redirect to dashboard) (and update VERIFIED in DB)
            ...or skip and load the page for manual attempt */

        // to avoid nested ifs, using labeled block which returns an Option containing User
        let validated_user: Option<db::User> = 'query_validation_block: {

            let user: db::User =
                match db::get_user_by_email(&pool, &email).await {
                    Ok(Some(user)) => user,
                    _ => break 'query_validation_block None
                };

            // get the saved, hashed verify code
            let code_hash_obj: auth::HashedVerificationCode =
                match db::get_verification_code(&pool, user.get_id()).await {
                    Ok(Some(code_hash_obj)) => code_hash_obj,
                    _ => break 'query_validation_block None
                };
            
            if code_hash_obj.has_exceeded_attempts() || code_hash_obj.is_expired() {
                message = "Your code is no longer valid. You may request a new code.".to_string();
                request_new_code = true;
                break 'query_validation_block None
            }

            // compare entered verify code to saved, hashed code, and return User if match.
            let code_match: bool =
                auth::verify_password(&verify_code, &code_hash_obj.code_hash);

            if code_match {
                Some(user)
            } else {
                // increment attemps and return None
                let _inc_obj_result: Result<auth::HashedVerificationCode, anyhow::Error> =
                    db::increment_verification_attempt(
                        &pool, user.get_id(), Some(code_hash_obj)
                    ).await;
                None
            }
        };

        // verify email give them cookies, redirect (to dashboard)
        if let Some(user) = validated_user {
            // verify user email:
            let _email_verified: bool =
                match db::verify_user(&pool, user.get_id()).await {
                    Ok(affected_count) => affected_count > 0,
                    Err(e) => {
                        eprint!("Error verifying user: {}", e);
                        false
                    }
                };

            // redirect to dash with auth cookies
            return authenticate_user_response(user, pool).await
        }
    }

    // User was NOT verified by querystring. So load template
    let verify_template: VerifyTemplate = VerifyTemplate {
        texts: VerifyTexts::new(&user_req_data),
        user: user_req_data,
        message,
        request_new_code,
        nav_data: NavData::new( "verify".to_string() ),
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(verify_template.render().unwrap())
}


/* ROOT DOMAIN */
#[get("/")]
async fn home(
    pool: web::Data<MySqlPool>,
    req: HttpRequest
) -> HttpResponse {

    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    let texts: HomeTexts = HomeTexts::new(&user_req_data);
    let pinned_post: String =
        match db::get_latest_pinned_post(&pool).await {
            Ok(Some(post)) => post.get_body_as_html(),
            Ok(None) => texts.default_pinned.clone(),
            Err(e) => {
                eprintln!("Error retrieving pinned post: {e}");
                texts.default_pinned.clone()
            }
        };

    // we need the posts to get the unified posts uposts
    let uposts: Vec<db::UnifiedPost> =
        match get_unified_posts(&pool, false).await {
            Ok(uposts) => uposts,
            Err(e) => {
                eprintln!("Error retrieving posts: {e}");
                return return_error_page(&req, 404)
            }
        };

    let categories: Vec<String> = match get_active_categories(&pool).await {
        Ok(cats) => cats,
        Err(e) => {
            eprintln!("Error retrieving categories: {e}");
            return return_error_page(&req, 404)
        }
    };

    // get categories html so we can print it wherever needed
    let insert_categories_template: InsertCategoriesTemplate = 
        InsertCategoriesTemplate { categories };

    let categories_html: String = match insert_categories_template.render() {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Error rendering categories: {e}");
            return return_error_page(&req, 404)
        }
    };

    let home_template: HomeTemplate = HomeTemplate {
        texts, uposts, categories_html,
        user: user_req_data,
        pinned_post,
        nav_data: NavData::new( "about".to_string() ),
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(home_template.render().unwrap())
}



/* LOGIN PAGE ROUTE FUNCTION */
pub async fn request_verification_page(
    req: HttpRequest
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    let login_template: ReqVerificationTemplate =
        ReqVerificationTemplate {
            texts: ReqVerificationTexts::new(&user_req_data),
            user: user_req_data,
            nav_data: NavData::new( "verify".to_string() ),
        };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(login_template.render().unwrap())
}



/* LOGIN PAGE ROUTE FUNCTION */
pub async fn login_page(
    req: HttpRequest
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    let login_template: LoginTemplate = LoginTemplate {
        texts: LoginTexts::new(&user_req_data),
        user: user_req_data,
        nav_data: NavData::new( "login".to_string() )
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(login_template.render().unwrap())
}


/* REGISTER PAGE ROUTE FUNCTION */
pub async fn register_page(
    req: HttpRequest
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    let register_template: RegisterTemplate = RegisterTemplate {
        agreements: AgreementTexts::new(&user_req_data.lang),
        texts: RegisterTexts::new(&user_req_data),
        user: user_req_data,
        nav_data: NavData::new( "register".to_string() )
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(register_template.render().unwrap())
}

/**
 * Main admin dashboard
 * if user just goes to /auth
 */
#[get("/view_images")]
pub async fn view_images_page(req: HttpRequest) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    // first get the images
    let dir: &str = "./uploads";
    let mut image_filenames: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    image_filenames.push(name.to_string());
                }
            }
        }
    }

    let template: ImagesTemplate =
        ImagesTemplate::new(user_req_data, image_filenames);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())  
}


/**
 * Main admin dashboard
 * if user just goes to /auth
 */
pub async fn admin_home(
    pool: web::Data<MySqlPool>,
    req: HttpRequest
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    let uposts: Vec<db::UnifiedPost> =
        match get_unified_posts(&pool, false).await {
            Ok(uposts) => uposts,
            Err(e) => {
                eprintln!("Error retrieving posts: {e}");
                return return_error_page(&req, 404)
            }
        };

    let admin_template: AdminTemplate = AdminTemplate {
        texts: AdminTexts::new(&user_req_data),
        user: user_req_data,
        uposts,
        nav_data: NavData::new( "admin".to_string() )
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(admin_template.render().unwrap())  
}


pub async fn admin_redirect() -> impl Responder {
    Redirect::to("/admin/dashboard")
}



/**
 * Show the page where the user can create a new post
 */
#[get("/new_book_page")]
pub async fn new_book_page(req: HttpRequest) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }
    
    let new_book_template: NewBookTemplate = NewBookTemplate {
        texts: NewBookTexts::new(&user_req_data),
        user: user_req_data,
        nav_data: NavData::new( "new_book".to_string() )
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(new_book_template.render().unwrap())
}



/**
 * Show the page where the user can create a new post
 */
#[get("/new_post")]
pub async fn new_post_page(req: HttpRequest) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }
    
    let new_post_template: NewPostTemplate = NewPostTemplate {
        texts: NewPostTexts::new(&user_req_data),
        user: user_req_data,
        nav_data: NavData::new( "new_post".to_string() )
    };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(new_post_template.render().unwrap())
}


#[get("/blog")]
pub async fn blog(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<CategoryQuery>
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    let mut category_option: Option<String> = None;

    // get all non-pinned uposts
    let uposts: Vec<db::UnifiedPost> =
        match get_unified_posts(&pool, false).await {
            Ok(uposts) => uposts,
            Err(e) => {
                eprintln!("Error retrieving posts: {e}");
                return return_error_page(&req, 404)
            }
        };

    // optionally shadow with filter if query for category exists
    let uposts: Vec<UnifiedPost> =
        if query.category.is_none() { uposts }
        else {
            let category: String = query.category.to_owned().unwrap().to_string();
            category_option = Some(category.to_owned());
            uposts.into_iter()
                .filter(|upost|
                upost.categories.contains(&category))
                .collect()
        };   

    let blog_post_template: BlogTemplate = BlogTemplate {
        uposts,
        category: category_option,
        texts: BlogTexts::new(&user_req_data),
        user: user_req_data,
        nav_data: NavData::new( "blog".to_string() )
    };
    
    HttpResponse::Ok()
        .content_type("text/html")
        .body(blog_post_template.render().unwrap())
}


/**
 * User views a single post page.
 */
#[get("/post/{id}")]
pub async fn view_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    post_id_obj: web::Path<PostId>
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // Painstaking extraction because we keep getting errors

    let post_option: Option<db::BlogPost> =
        match db::get_post_by_id(&pool, post_id_obj.id).await {
            Ok(option) => option,
            Err(e) => {
                eprintln!("Error retrieving unified post 1: {e}");
                return return_error_page(&req, 404)
            }
        };

    let post: db::BlogPost =
        if post_option.is_some() {
            post_option.unwrap()
        } else {
            eprintln!("Post not found");
            return error_post_response(&req, 404);
        };

    // Get the requested post and package it with its categories
    let upost: UnifiedPost =
        match get_unified_post(&pool, post).await {
            Ok(upost) => upost,
            Err(e) => {
                eprintln!("Error retrieving unified post 1: {e}");
                return return_error_page(&req, 404)
            }
        };

    let template: PostTemplate = PostTemplate {
        texts: PostTexts::new(&user_req_data),
        user: user_req_data,
        upost,
        nav_data: NavData::new( "blog".to_string() )
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}

/**
 * Show the page where the user can create a new post
 */
#[get("/edit_post/{id}")]
pub async fn edit_post_page(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    post_id_obj: web::Path<PostId>
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    let error_response: ErrorResponse = ErrorResponse {
        error: "Error retrieving post".to_string(),
        code: 404
    };

    // Get the requested post
    let post: db::BlogPost =
        match db::get_post_by_id(&pool, post_id_obj.id).await {
            Ok(Some(post)) => post,
            _ => return HttpResponse::NotFound().json(error_response)
        };

    // get the categories
    let categories: Vec<String> =
        get_categories_by_post_id(post.id as i64, &pool)
        .await.unwrap_or_default();
    
    let categories_string: String = vec_to_string(&categories);

    let edit_post_template: EditPostTemplate = EditPostTemplate {
        texts: EditPostTexts::new(&user_req_data),
        user: user_req_data,
        post, categories_string,
        nav_data: NavData::new( "edit_post".to_string() )
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(edit_post_template.render().unwrap())

}


/**
 * Main page for user account info.
 * */
#[get("/dashboard")]
pub async fn dashboard_page(
    pool: web::Data<MySqlPool>,
    req: HttpRequest
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    if user_req_data.id.is_none() {
        return send_to_login()
    }

    let id: i32 = user_req_data.id.unwrap();

    match db::get_user_by_id(&pool, id).await {
        Ok(Some(user)) =>{
            let dashboard_template: DashboardTemplate<'_> = DashboardTemplate {
                user_data: &user,
                texts: DashboardTexts::new(&user_req_data),
                user: user_req_data,
                nav_data: NavData::new( "dashboard".to_string() )
            };

            return HttpResponse::Ok()
                .content_type("text/html")
                .body(dashboard_template.render().unwrap());
        },
        Ok(None) => return send_to_login(),
        Err(_e) => {
            // redirect to ERROR PAGE
            return HttpResponse::Found()
                .append_header((header::LOCATION, "/error"))
                .finish();
        }
    };
}


// Function for the catch-all "not found" route
pub async fn not_found() -> impl Responder {
    Redirect::to("/error/404")
}


#[get("/error/{code}")]
async fn error_page(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    let code: String = match path.into_inner().parse::<String>() {
        Ok(code) => code,
        Err(_) => "400".to_string()
    };

    let error_data: ErrorData = ErrorData::new(
        code,
        &user_req_data.lang
    );

    let error_template: ErrorTemplate<> = ErrorTemplate {
        error_data,
        texts: ErrorTexts::new(&user_req_data),
        user: user_req_data,
        nav_data: NavData::new( "error".to_string() )
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(error_template.render().unwrap())
}


#[get("/error")]
async fn error_root() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/error/500"))
        .finish()
}


#[get("/error/")]
async fn error_root_2() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/error"))
        .finish()
}
