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

use crate::resource_mgr::{AgreementTexts, BlogTexts, NewPostTexts, EditPostTexts};
// local modules, loaded as crates (declared as mods in main.rs)
use crate::{
    resources::get_translation,
    db, utils, auth,
    resource_mgr::{
        HomeTexts, LoginTexts, RegisterTexts, AdminTexts, VerifyTexts,
        ErrorTexts, EditClientTexts, NewClientTexts, DashboardTexts,
        ReqVerificationTexts, ErrorData
     },
     auth_code_shared::{
        AuthCodeSuccess,
        AuthCodeRequest,
        RefreshCheckRequest,
        RefreshCheckError,
        RefreshCheckSuccess,
        RefreshCheckResponse,
        SendVerificationEmailRequest,
        SendVerificationEmailResponse,
    },
    routes_utils::{*},
    email
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
  * Returns a redirect URL, either to client front page (if not logged in)
  * or to client reception (if logged in).
  * The JS receiving this must do the actual redirect.
  */
#[post("client_link/{client_id}")]
async fn link_to_client(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    client_id_obj: web::Path<ClientId>
) -> HttpResponse {
    // TODO: replace "server error" with dynamic error response
    let server_error: HttpResponse = get_server_error(&req).await;
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // Get the requested client site data
    let client_data: db::ClientData =
        match db::get_client_by_client_id(&pool, &client_id_obj.client_id).await {
            Ok(Some(data)) => data,
            Ok(None) => return server_error,
            Err(_e) => return server_error
        };

    if !user_req_data.logged_in {
        // just send them to the client url
        return HttpResponse::Ok().json(
            FullRedirectUri {
                redirect_uri: client_data.domain
            }
        )        
    }

    let user_id: i32 = user_req_data.id.unwrap();
    // They are already logged in.
    post_auth_client_site_redirect(
        req, user_id, pool,
        client_id_obj.client_id.to_string(),
        None
    ).await
}



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
        let _email_verified: bool =
            match db::verify_user(&pool, user.get_id()).await {
                Ok(affected_count) => affected_count > 0,
                Err(e) => {
                    eprint!("Error verifying user: {}", e);
                    false
                }
            };

        // set cookies, signal to redirect to dash
        return authenticate_user_response(
            req, user, pool,
            utils::auth_client_id(),
            false
        ).await
    } else {

        // Wrong code. Increment attempts.
        let _inc_obj_result: Result<auth::HashedVerificationCode, anyhow::Error> =
            db::increment_verification_attempt(
                &pool, user.get_id(), Some(code_hash_obj)
            ).await;
        
        let message: String = "The code does not match. 
            Get the correct code from your email, or request a new one.".to_string();
        let error_struct: ErrorResponse = ErrorResponse { error: message, code: 401 };
        return HttpResponse::TooManyRequests().json(error_struct)
    }
   
}



/** REGISTER
 * The user/client calls this API to register.
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

    authenticate_user_response(
        req, user, pool,
        info.client_id.clone(),
        false
    ).await
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

    // create auth_token if site is external
    println!("Client id: {}", info.client_id);

    authenticate_user_response(
        req, user, pool,
        info.client_id.clone(),
        false
    ).await
}



/**
 * The admin can update the client secret.
 * They receive the raw (unhashed) secret ONCE and they must put that
 * in the env variables of the client site.
 * We then hash the secret and store the hashed version in the DB.
 */
#[post("/req_new_client_secret")]
async fn req_secret_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    inputs: web::Json<ClientId>
) -> HttpResponse {
    println!("Seeking new Secret");
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    let raw_client_secret_json: RawClientSecret = RawClientSecret {
        raw_client_secret: utils::generate_client_secret()
    };

    let hashed_client_secret: String = auth::hash_password(
       raw_client_secret_json.raw_client_secret.to_owned()
    ).to_owned();

    match db::update_client_secret(
        &pool,
        &inputs.client_id,
        &hashed_client_secret
    ).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                HttpResponse::Ok()
                    .json(raw_client_secret_json)
            } else {
                return_internal_err_json()
            }
        },
        Err(_e) => {
            return_internal_err_json()
        }
    }
}


/**
 * The post route for adding new CLIENT SITE to the database.
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

    // Add the post to the database
    let post_succes_obj: BlogPostSuccess = match db::add_post(
        &pool,
        &blog_post_data.post_title,
        &blog_post_data.post_body,
        user_req_data.username.unwrap(),
        blog_post_data.pinned
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
 * The post route for adding new CLIENT SITE to the database.
 * Checks that the user is truly an admin, checks that all the 
 * data is legit, then adds it to the database.
 * Lots of opportunities to send errors.
 */
#[post("/add_client")]
async fn new_client_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut inputs: web::Json<ClientInputs>
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    // Trim every string
    inputs.trim_all_strings();

    // Make sure the required fields are not empty
    let domains_are_valid: bool =
        utils::validate_url(&inputs.redirect_uri) &&
        utils::validate_url(&inputs.site_domain);
    
    if !domains_are_valid {
        println!("domains are not valid");
        return HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .json(ErrorResponse{
                error: String::from("Invalid domain format"),
                code: 406
            })
    }

    // Check all the fields.

    let client_id_is_valid: bool = utils::string_length_valid(
        utils::StringRange{ min: 2, max: 20 },
        &inputs.client_id
    ) && utils::has_no_whitespace(
        &inputs.client_id
    );

    if !client_id_is_valid {
        eprintln!("CLIENT ID is not valid");
        return HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .json(ErrorResponse{
                error: String::from("Client ID: 2-20 characters, no spaces"),
                code: 406
            });
    }

    let name_is_valid: bool = utils::string_length_valid(
        utils::StringRange{ min: 2, max: 20 },
        &inputs.site_name
    );

    if !name_is_valid {
        eprintln!("NAME is not valid");
        return HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .json(ErrorResponse{
                error: String::from("Site name: 2-20 characters."),
                code: 406
            });
    }

    // String checks passed. Enter into DB, generate secret, show admin secret
    let raw_client_secret: String = utils::generate_client_secret();
    let hashed_secret: String = auth::hash_password(raw_client_secret.to_owned());        

    let client_data: db::NewClientData = db::NewClientData {
        site_domain: inputs.site_domain.to_owned(),
        site_name: inputs.site_name.to_owned(),
        client_id: inputs.client_id.to_owned(),
        redirect_uri: inputs.redirect_uri.to_owned(),
        hashed_client_secret: hashed_secret.to_owned(),
        logo_url: inputs.logo_url.to_owned(),
        description: inputs.description.to_owned(),
        category: inputs.category.to_owned(),
        client_type: inputs.client_type.to_owned(),
        is_active: inputs.is_active,
    };

    let new_client_result: Result<u64, anyhow::Error> =
        db::add_external_client(&pool, client_data).await;
    
    match new_client_result {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                // We added it to the DB. Send the admin their raw secret.
                let raw_client_secret_json: RawClientSecret = RawClientSecret {
                    raw_client_secret
                };
                return HttpResponse::Ok()
                    .json(raw_client_secret_json);
            } else {
                return return_internal_err_json();
            }
        },
        Err(e) => {
            // Database error
            eprintln!("Error: {e}");
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .json(ErrorResponse{
                    error: format!("Error: {e}"),
                    code: 500
                })
        }
    }
}

/**
 * The post route for adding new CLIENT SITE to the database.
 * Checks that the user is truly an admin, checks that all the 
 * data is legit, then adds it to the database.
 * Lots of opportunities to send errors.
 */
#[post("/update_client")]
async fn update_client_post(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut inputs: web::Json<ClientInputs>
) -> HttpResponse {
    println!("UPDATING CLIENT");

    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);
    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    // Trim every string
    inputs.trim_all_strings();

    // Make sure the required fields are not empty
    let domains_are_valid: bool =
        utils::validate_url(&inputs.redirect_uri) &&
        utils::validate_url(&inputs.site_domain);
    
    if !domains_are_valid {
        println!("domains are not valid");
        return HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .json(ErrorResponse{
                error: String::from("Invalid domain format"),
                code: 406
            })
    }

    // Check all the fields.

    let client_id_is_valid: bool = utils::string_length_valid(
        utils::StringRange{ min: 2, max: 20 },
        &inputs.client_id
    ) && utils::has_no_whitespace(
        &inputs.client_id
    );

    if !client_id_is_valid {
        println!("CLIENT ID is not valid");
        return HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .json(ErrorResponse{
                error: String::from("Site name: 2-20 characters."),
                code: 406
            });
    }

    let name_is_valid: bool = utils::string_length_valid(
        utils::StringRange{ min: 2, max: 20 },
        &inputs.site_name
    );

    if !name_is_valid {
        println!("NAME is not valid");
        return HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .json(ErrorResponse{
                error: String::from("Site name: 2-20 characters."),
                code: 406
            });
    }

    // If string checks passed, enter into DB, generate secret, show admin secret
    if domains_are_valid && client_id_is_valid && name_is_valid {
        let client_data: db::UpdateClientData = db::UpdateClientData {
            site_domain: inputs.site_domain.to_owned(),
            site_name: inputs.site_name.to_owned(),
            client_id: inputs.client_id.to_owned(),
            redirect_uri: inputs.redirect_uri.to_owned(),
            logo_url: inputs.logo_url.to_owned(),
            description: inputs.description.to_owned(),
            category: inputs.category.to_owned(),
            client_type: inputs.client_type.to_owned(),
            is_active: inputs.is_active,
        };
    
        let update_client_result: Result<i32, anyhow::Error> =
            db::update_external_client(&pool, client_data).await;
        
        match update_client_result {
            Ok(rows_affected) => {
                if rows_affected > 0 {
                    let update_data: UpdateData = UpdateData::new(true);

                    HttpResponse::Ok()
                        .json(update_data)
                } else {
                    return_internal_err_json()
                }

            },
            Err(e) => {
                // Database error
                eprintln!("Error: {e}");
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(ErrorResponse{
                        error: format!("Error: {e}"),
                        code: 500
                    })
            }
        }
    } else {
        HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .json(ErrorResponse{
                error: String::from("Invalid Inputs"),
                code: 406
            })
    }
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

    // Add the post to the database
    let post_succes_obj: BlogPostSuccess = match db::update_post(
        &pool,
        blog_post_data.post_id,
        &blog_post_data.post_title,
        &blog_post_data.post_body,
        blog_post_data.pinned
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



// if user just goes to /auth or /auth/
pub async fn auth_home() -> impl Responder {
    Redirect::to("/auth/login")
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
            return authenticate_user_response(
                req, user, pool,
                utils::auth_client_id(),
                true
            ).await
        }
    }

    // User was NOT verified by querystring. So load template
    let verify_template: VerifyTemplate = VerifyTemplate {
        texts: VerifyTexts::new(&user_req_data),
        user: user_req_data,
        message,
        request_new_code
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

    // get list of client sites
    // select ACTIVE and NOT AUTH_SITE
    // update HomeTemplate to include that list

    // Get client site references to list on admin site
    let client_links: Vec<db::ClientLinkData> =
        match db::get_client_links(&pool).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error retrieving client references: {e}");
                Vec::new()
            }
        };

    let pinned_post: Option<String> =
        match db::get_latest_pinned_post(&pool).await {
            Ok(Some(post)) => Some(post.body.to_owned()),
            Ok(None) => None,
            Err(e) => {
                eprintln!("Error retrieving pinned post: {e}");
                None
            }
        };

    let home_template: HomeTemplate = HomeTemplate {
        texts: HomeTexts::new(&user_req_data),
        user: user_req_data,
        pinned_post,
        client_links
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
        };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(login_template.render().unwrap())
}



/* LOGIN PAGE ROUTE FUNCTION */
pub async fn login_page(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<LoginQuery>
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // Get client site references to list on login site
    let client_refs: Vec<db::ClientRef> =
        match db::get_client_refs(&pool).await {
            Ok(refs) => refs,
            Err(e) => {
                eprintln!("Error retrieving client references: {e}");
                Vec::new()
            }
        };

    let selected_client_id: String = match &query.client_id {
        Some(client_id) => client_id.to_owned(),
        None => "".to_string()
    };

    // getting the querystring for link to register page
    let querystring: String = if selected_client_id.chars().count() > 0 
        { format!("?client_id={}", selected_client_id) }
        else { "".to_string() };

    // Make sure there's a site to login to.
    let login_is_available: bool = client_refs.len() > 0;

    let login_template: LoginTemplate = LoginTemplate {
        texts: LoginTexts::new(&user_req_data),
        user: user_req_data,
        client_refs,
        login_is_available,
        selected_client_id,
        querystring
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(login_template.render().unwrap())
}


/* REGISTER PAGE ROUTE FUNCTION */
pub async fn register_page(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<LoginQuery>
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

        // Get client site references to list on login site
    let client_refs: Vec<db::ClientRef> =
        match db::get_client_refs(&pool).await {
            Ok(refs) => refs,
            Err(e) => {
                eprintln!("Error retrieving client references: {e}");
                Vec::new()
            }
        };
    
    
    let selected_client_id: String = match &query.client_id {
        Some(client_id) => client_id.to_owned(),
        None => "".to_string()
    };

    // getting the querystring for link to register page
    let querystring: String = if selected_client_id.chars().count() > 0 
        { format!("?client_id={}", selected_client_id) }
        else { "".to_string() };

    let register_template: RegisterTemplate = RegisterTemplate {
        agreements: AgreementTexts::new(&user_req_data.lang),
        texts: RegisterTexts::new(&user_req_data),
        user: user_req_data,
        client_refs,
        selected_client_id,
        querystring,
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(register_template.render().unwrap())
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
    
    // Get client site references to list on admin site
    let client_refs: Vec<db::ClientRef> =
        match db::get_client_refs(&pool).await {
            Ok(refs) => refs,
            Err(e) => {
                eprintln!("Error retrieving client references: {e}");
                Vec::new()
            }
        };

    let posts: Vec<db::BlogPost> = match db::get_posts(&pool).await {
        Ok(b_posts) => b_posts,
        Err(_e) => return return_error_page(&req, 404)
    };

    let admin_template: AdminTemplate = AdminTemplate {
        texts: AdminTexts::new(&user_req_data),
        user: user_req_data,
        client_refs,
        posts
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(admin_template.render().unwrap())  
}


pub async fn admin_redirect() -> impl Responder {
    Redirect::to("/admin/dashboard")
}


/**
 * The page where an admin can enter information for a NEW client site.
 * This is just the form. Another (post) function will receive the data
 * submitted from this form and process it.
 */
pub async fn new_client_site_form_page(req: HttpRequest) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }
    
    let new_client_template: NewClientTemplate = NewClientTemplate {
        texts: NewClientTexts::new(&user_req_data),
        user: user_req_data
    };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(new_client_template.render().unwrap())
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
        user: user_req_data
    };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(new_post_template.render().unwrap())
}

#[get("/blog")]
pub async fn blog(
    pool: web::Data<MySqlPool>,
    req: HttpRequest
) -> HttpResponse {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    let posts: Vec<db::BlogPost> = match db::get_non_pinned_posts(&pool).await {
        Ok(b_posts) => b_posts,
        Err(_e) => return return_error_page(&req, 404)
    };

    let dev_blog_template: BlogTemplate = BlogTemplate {
        posts,
        texts: BlogTexts::new(&user_req_data),
        user: user_req_data
    };
    
    HttpResponse::Ok()
        .content_type("text/html")
        .body(dev_blog_template.render().unwrap())
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
    let post_obj_result: Result<Option<db::BlogPost>, anyhow::Error> =
        db::get_post_by_id(&pool, post_id_obj.id).await;

    if post_obj_result.is_err() {
        return HttpResponse::Unauthorized().json(error_response)
    }

    match post_obj_result.unwrap() {
        Some(post) => {
            let edit_post_template: EditPostTemplate = EditPostTemplate {
                texts: EditPostTexts::new(&user_req_data),
                user: user_req_data,
                post
            };

            HttpResponse::Ok()
                .content_type("text/html")
                .body(edit_post_template.render().unwrap())
        },
        None => HttpResponse::Unauthorized().json(error_response)
    }
}

/**
 * The page where an admin can EDIT information for an EXISTING client site.
 * This is just the form. Another (post) function will receive the data
 * submitted from this form and process it.
 */
#[get("/edit_client/{auth_id}")]
pub async fn edit_client_site_form_page(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    auth_id: web::Path<String>
) -> impl Responder {
    let user_req_data: auth::UserReqData = auth::get_user_req_data(&req);

    // check if they're admin
    if let Some(redirect_resp) = redirect_non_admin(&user_req_data, &req) {
        return redirect_resp;
    }

    // Get the requested client site data
    let client_data_result: Result<Option<db::ClientData>, anyhow::Error> =
        db::get_client_by_client_id(&pool, &auth_id).await;

    if client_data_result.is_err() {
        return return_error_page(&req, 404);
    }

    match client_data_result.unwrap() {
        Some(client_data) => {
            let new_client_template: EditClientTemplate = EditClientTemplate {
                texts: EditClientTexts::new(&user_req_data),
                user: user_req_data,
                client_data
            };
            
            HttpResponse::Ok()
                .content_type("text/html")
                .body(new_client_template.render().unwrap())
        },
        None => return_error_page(&req, 404)
    }
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
        return send_to_login();
    }

    let id: i32 = user_req_data.id.unwrap();

    match db::get_user_by_id(&pool, id).await {
        Ok(Some(user)) =>{
            let dashboard_template: DashboardTemplate<'_> = DashboardTemplate {
                user_data: &user,
                texts: DashboardTexts::new(&user_req_data),
                user: user_req_data
            };

            return HttpResponse::Ok()
                .content_type("text/html")
                .body(dashboard_template.render().unwrap());
        },
        Ok(None) => {
            eprintln!("Failed to find user");
            return send_to_login();
        },
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
        user: user_req_data
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



/* 
 * 
 * 
 * 
 * 
 * =============================
 * =============================
 * =====                   =====
 * =====  EXTERNAL ROUTES  =====
 * =====                   =====
 * =============================
 * =============================
 * 
 * 
 * 
 * Routes to be called by external client apps.
 * 
 * /login should actually have a dropdown of client sites (not in external scope actually)
 * /refresh will verify refresh token, return OK
 * 
 * 
 * FLOW:
 * 
 * For any client app, START from the auth app's login page.
 * User selects which site to login to.
 * User is redirected
 * 
*/


#[post("/verify_auth_code")]
async fn verify_auth_code(
    pool: web::Data<MySqlPool>,
    inputs: web::Json<AuthCodeRequest>
) -> HttpResponse {

    /* 
     * From DB gather:
     * * THINGS TO CHECK
     * * THINGS TO SEND TO USER
     * 
     * THINGS TO CHECK:
     * * auth_codes.user_id
     * * auth_codes.client_id
     * * auth_codes.expiry_date
     * 
     * THINGS TO SEND:
     * * user.sub (id)
     * * user.username
     * * user.role
     * * refresh_token
     * 
     * We will have to CREATE the refresh_token
     * 
     * We will need a custom error struct
     */

    let auth_code_data: db::AuthCodeData =
        match db::get_auth_code_data(&pool, &inputs.code).await {
            Ok(option) => {
                match option {
                    Some(data) => data,
                    None => { return return_not_found_err_json() }
                }
            },
            Err(_e) => { return return_internal_err_json() }
        };

    // Make sure it's not expired
    if auth_code_data.is_expired() {
        return return_authentication_err_json();
    }

    // GOT the auth_code_data. Now check it against the input data
    // make sure client_id and client_secret are the right ones.

    let hashed_client_secret: String =
        match db::get_client_secret(&pool, &inputs.client_id).await {
            Ok(option) => {
                match option {
                    Some(secret_obj) => secret_obj.hashed_client_secret,
                    None => { return return_not_found_err_json() }
                }
            },
            Err(_e) => { return return_internal_err_json() }
        };

    let secrets_match: bool = auth::verify_password(&inputs.client_secret, &hashed_client_secret);
    let client_ids_match: bool = inputs.client_id == auth_code_data.client_id;

    // TODO: check auth_code EXPIRY date

    if secrets_match && client_ids_match {
        let username_and_role: db::UsernameAndRole =
            match db::get_username_and_role_by_id(&pool, auth_code_data.user_id).await {
                Ok(option) => {
                    match option {
                        Some(data_obj) => data_obj,
                        None => { return return_not_found_err_json() }
                    }
                },
                Err(_e) => { return return_internal_err_json() }
            };

        // CREATE the refresh token and save to DB
        // create a refresh_token and put it in the DB
        let refresh_token: String = match db::add_refresh_token(
            &pool,
            auth_code_data.user_id,
            auth_code_data.client_id,
            auth::generate_refresh_token()
        ).await {
            Ok(refresh_token) => refresh_token,
            Err(_e) =>  return return_internal_err_json()
        };

        let user_data: AuthCodeSuccess = AuthCodeSuccess {
            email_verified: username_and_role.is_verified(),
            user_id: auth_code_data.user_id,
            username: username_and_role.username,
            user_role: username_and_role.role,
            refresh_token
        };

        // now DELETE the auth token
        return HttpResponse::Ok()
            .json(user_data);
    }

    // RETURN FAILURE
    return_authentication_err_json()
}


/**
 * when a user on a client app checks their refresh_token (in the cookies on the client app)
 * against the refresh token saved in the database.
 */
#[post("/check_refresh")]
async fn check_refresh(
    pool: web::Data<MySqlPool>,
    inputs: web::Json<RefreshCheckRequest>
) -> HttpResponse {

    // Saving the err_response for possible later use
    let make_err_response =
        |code: u16, msg: &str| -> HttpResponse {
            HttpResponse::Ok()
                .json(RefreshCheckResponse::Err(RefreshCheckError {
                    error_code: code,
                    message: msg.to_string()
                }))
        };

    // get the inputs and check them all

    let r_db_token: db::RefreshToken =
        match db::get_refresh_token(
            &pool,
            inputs.user_id,
            inputs.client_id.to_owned()
        ).await {
            Ok(option) => {
                match option {
                    Some(token) => token,
                    None => return make_err_response(404, "No token found")
                }
            },
            Err(_e) => return make_err_response(500, "Database Error")
        };

    let token_is_valid: bool = 
        inputs.token.as_str() == r_db_token.get_token() &&
        !r_db_token.is_expired();

    let token_response: RefreshCheckResponse =
        RefreshCheckResponse::Ok(RefreshCheckSuccess::new(token_is_valid));

    return HttpResponse::Ok()
        .json(token_response);
}



/**
 * when a user on a client app checks their refresh_token (in the cookies on the client app)
 * against the refresh token saved in the database.
 */
#[post("/req_ver_email")]
async fn req_ver_email(
    pool: web::Data<MySqlPool>,
    inputs: web::Json<SendVerificationEmailRequest>
) -> HttpResponse {

    // Saving the response function for later use
    let make_response =
        |msg: &str, success: bool| -> HttpResponse {
            HttpResponse::Ok()
                .json(SendVerificationEmailResponse {
                    success: success,
                    message: msg.to_string(),
                    user_id: inputs.user_id
                })
        };

    // make sure the client secret matches the client id
    // get the email address (and username) from user_id
    // send email with send_verification_email
    // return response
    // We should ALWAYS return the same kind of struct (SendVerificationEmailResponse)

    // Get hashed_client_secret from database
    let stored_hash: String = match db::get_client_secret(&pool, &inputs.client_id).await {
        Ok(Some(client_data)) => client_data.hashed_client_secret,
        Ok(None) => return make_response("No client found with that ID", false),
        Err(e) => {
            eprintln!("Database Error: {}", e.to_string());
            return make_response("Database error", false)
        }
    };

    // make sure hashes match 
    if !auth::verify_password(&inputs.client_secret, &stored_hash) {
        return make_response("Client secret does not match", false)
    }

    // get email address
    let user: db::User = match db::get_user_by_id(&pool, inputs.user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return make_response("No user with that id", false),
        Err(e) => {
            eprintln!("Database Error: {}", e.to_string());
            return make_response("Database error", false)
        }
    };

    // making sure the provided username matches that in the DB (not randomly sending user_ids)
    if user.get_username() != &inputs.username {
        println!("usernames: {}, {}", user.get_username(), inputs.username);
        return make_response("User id does not match username", false)
    }

    // user exists. See if there's already a code, and if enough time has elapsed
    let existing_code_option: Option<auth::HashedVerificationCode> =
        match db::get_verification_code(&pool, user.get_id()).await {
            Ok(code_option) => code_option,
            Err(_e) => return make_response("Database error", false)
        };

    if let Some(code_obj) = existing_code_option {
        if !code_obj.can_request_new() {
            let message: &str = "You must wait two mintues before requesting a new code.";
            return make_response(message, false)
        }
    }

    let email: &String = user.get_email();
    let email_sent: bool = true;
        // email::send_verification_email(&pool, user.get_username(), user.get_id(), email).await;

    if email_sent {
        make_response("Email sent. Check your inbox for verification code and link.", true)
    } else {
        make_response("Email failed to send.", false)
    }
}