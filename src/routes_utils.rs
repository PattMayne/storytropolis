/* 
 * 
 * 
 * 
 * 
 * ==========================
 * ==========================
 * =====                =====
 * =====  ROUTES UTILS  =====
 * =====                =====
 * ==========================
 * ==========================
 * 
 * Functions and structs to support the routes.rs module.
 * 
 * 
*/


use actix_web::{
    HttpResponse, HttpRequest, web::Redirect, web,
    Responder, http::StatusCode, http::header };
use actix_web::cookie::{ Cookie };
use askama::Template;
use serde::{ Deserialize, Serialize };
use sqlx::{MySqlPool };

use crate::db::BlogPost;
use crate::resource_mgr::AgreementTexts;
// local modules, loaded as crates (declared as mods in main.rs)
use crate::{
    db, utils,
    auth::{ self, UserReqData },
    resources::get_translation,
    resource_mgr::{
        HomeTexts, LoginTexts, RegisterTexts, AdminTexts, BlogTexts,
        ErrorTexts, EditClientTexts, NewClientTexts, DashboardTexts,
        NewPostTexts, EditPostTexts, VerifyTexts, ReqVerificationTexts,
        ErrorData, error_by_code
     }
};


/* 
 * 
 * 
 * 
 * 
 * 
 * =======================================
 * =======================================
 * =====                             =====
 * =====  STRUCTS for SERIALIZATION  =====
 * =====                             =====
 * =======================================
 * =======================================
 * 
 * 
 * (outgoing data)
 * 
 * 
 * 
 * 
*/


#[derive(Serialize)]
pub struct BlogPostSuccess {
    pub success: bool,
    pub message: String,
    pub post_id: i32,
}


#[derive(Serialize)]
pub struct UserData {
    pub user_id: i32,
    pub username: String,
    pub refresh_token: String,
}


#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}


#[derive(Serialize)]
pub struct SendToError {
    pub send_to_error_page: bool,
    pub code: u16,
}

#[derive(Serialize)]
pub struct BadRegistrationInputs {
    pub email_valid: bool,
    pub username_valid: bool,
    pub password_valid: bool,
    pub code: u16,
}


#[derive(Serialize)]
pub struct BadNames {
    pub names_valid: bool,
    pub code: u16,
}


#[derive(Serialize)]
pub struct RawClientSecret {
    pub raw_client_secret: String,
}

#[derive(Serialize)]
pub struct BadPassword {
    pub password_valid: bool,
    pub code: u16,
}


// Upon successful Registration or login, send back auth token (JWT token)
#[derive(Serialize)]
pub struct FreshLoginData {
    pub username: String,
}


#[derive(Serialize)]
pub struct LogoutData {
    pub logout: bool,
}


#[derive(Serialize)]
pub struct UpdateData {
    pub success: bool,
}

#[derive(Deserialize)]
pub struct PostId {
    pub id: i64,
}


#[derive(Serialize)]
pub struct FullRedirectUri {
    pub redirect_uri: String,
}

#[derive(Serialize)]
pub struct Message {
    pub message: String,
}


impl LogoutData {
    pub fn new() -> Self {
        LogoutData { logout: true } }
}


impl UpdateData {
    pub fn new(success: bool) -> Self {
        UpdateData { success } }
}


impl BadNames {
    pub fn new(code: u16) -> Self {
        BadNames {
            code,
            names_valid: false,
        }
    }
}

impl BadPassword {
    pub fn new(code: u16) -> Self {
        BadPassword {
            code,
            password_valid: false,
        }
    }
}

impl SendToError {
    pub fn new(code: u16) -> Self {
        SendToError {
            code,
            send_to_error_page: true,
        }
    }
}


/* 
 * 
 * 
 * 
 * 
 * 
 * 
 * 
 * ==========================================
 * ==========================================
 * =====                                =====
 * =====  STRUCTS for DE-SERIALIZATION  =====
 * =====                                =====
 * ==========================================
 * ==========================================
 * 
 * for JSON DE-SERIALIZATION (incoming data)
 * 
 * 
 * 
 * 
 * 
 */


#[derive(Deserialize)]
pub struct LoginQuery {
    pub client_id: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub code: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct ClientId {
    pub client_id: String,
}

// Store credentials when user tries to register
#[derive(Deserialize)]
pub struct RegisterCredentials {
    pub username: String,
    pub email: String,
    pub password: String,
    pub client_id: String,
    pub has_agreed_terms: bool,
    pub website: String,
}

#[derive(Deserialize)]
pub struct NewCodeRequest {
    pub email: String,
}

// Store credentials when a user tries to login
#[derive(Deserialize)]
pub struct LoginCredentials{
    pub username_or_email: String,
    pub password: String,
    pub client_id: String,
}

#[derive(Deserialize)]
pub struct RealNames {
    pub first_name: String,
    pub last_name: String,
}


#[derive(Deserialize)]
pub struct NewPassword {
    pub password: String,
}


#[derive(Deserialize)]
pub struct ClientDataReq {
    pub client_id: String,
}


#[derive(Deserialize)]
pub struct BlogPostData {
    pub post_title: String,
    pub post_body: String,
    pub pinned: bool
}

impl BlogPostData {
    pub fn trim_all_strings(&mut self) {
        self.post_title = self.post_title.trim().to_string();
        self.post_body = self.post_body.trim().to_string();
    }
}


#[derive(Deserialize)]
pub struct DeletePostId {
    pub post_id: i32,
}

#[derive(Deserialize)]
pub struct BlogPostUpdateData {
    pub post_id: i64,
    pub post_title: String,
    pub post_body: String,
    pub pinned: bool,
}

impl BlogPostUpdateData {
    pub fn trim_all_strings(&mut self) {
        self.post_title = self.post_title.trim().to_string();
        self.post_body = self.post_body.trim().to_string();
    }
}


#[derive(Deserialize)]
pub struct ClientInputs {
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

impl ClientInputs {
    pub fn trim_all_strings(&mut self) {
        self.client_id = self.client_id.trim().to_string();
        self.site_domain = self.site_domain.trim().to_string();
        self.site_name = self.site_name.trim().to_string();
        self.redirect_uri = self.redirect_uri.trim().to_string();
        self.logo_url = self.logo_url.trim().to_string();
        self.client_type = self.client_type.trim().to_string();
        self.description = self.description.trim().to_string();
        self.category = self.category.trim().to_string();
    }

    pub fn print_all_strings(&self) {
        println!("client_id: {}", self.client_id);
        println!("domain: {}", self.site_domain);
        println!("site_name: {}", self.site_name);
        println!("r_uri: {}", self.redirect_uri);
        println!("logo_url: {}", self.logo_url);
        println!("client_type: {}", self.client_type);
        println!("desc: {}", self.description);
        println!("category: {}", self.category);
    }
}

// OTHER STRUCTS


pub struct TwoAuthCookies {
    pub jwt_cookie: Cookie<'static>,
    pub refresh_token_cookie: Cookie<'static>,
}



/* 
 * 
 * 
 * 
 * 
 * ===================================
 * ===================================
 * =====                         =====
 * =====  ASKAMA HTML TEMPLATES  =====
 * =====                         =====
 * ===================================
 * ===================================
 * 
 * 
 * 
 * 
 */


#[derive(Template)]
#[template(path ="index.html")]
pub struct HomeTemplate {
    pub texts: HomeTexts,
    pub user: auth::UserReqData,
    pub client_links: Vec<db::ClientLinkData>,
    pub pinned_post: Option<String>,
}


#[derive(Template)]
#[template(path ="verify.html")]
pub struct VerifyTemplate {
    pub texts: VerifyTexts,
    pub user: auth::UserReqData,
    pub message: String,
    pub request_new_code: bool,
}




#[derive(Template)]
#[template(path ="dev_blog.html")]
pub struct BlogTemplate {
    pub texts: BlogTexts,
    pub user: auth::UserReqData,
    pub posts: Vec<db::BlogPost>,
}

#[derive(Template)]
#[template(path ="request_verification.html")]
pub struct ReqVerificationTemplate {
    pub texts: ReqVerificationTexts,
    pub user: auth::UserReqData,
}

#[derive(Template)]
#[template(path ="login.html")]
pub struct LoginTemplate {
    pub texts: LoginTexts,
    pub user: auth::UserReqData,
    pub client_refs: Vec<db::ClientRef>,
    pub login_is_available: bool,
    pub selected_client_id: String,
    pub querystring: String,
}

#[derive(Template)]
#[template(path ="admin_page.html")]
pub struct AdminTemplate {
    pub texts: AdminTexts,
    pub user: auth::UserReqData,
    pub client_refs: Vec<db::ClientRef>,
    pub posts: Vec<db::BlogPost>,
}


#[derive(Template)]
#[template(path ="new_client_form_page.html")]
pub struct NewClientTemplate {
    pub user: auth::UserReqData,
    pub texts: NewClientTexts,
}


#[derive(Template)]
#[template(path ="new_post.html")]
pub struct NewPostTemplate {
    pub user: auth::UserReqData,
    pub texts: NewPostTexts,
}


#[derive(Template)]
#[template(path ="edit_post.html")]
pub struct EditPostTemplate {
    pub user: auth::UserReqData,
    pub texts: EditPostTexts,
    pub post: BlogPost,
}


#[derive(Template)]
#[template(path ="edit_client_form_page.html")]
pub struct EditClientTemplate {
    pub user: auth::UserReqData,
    pub texts: EditClientTexts,
    pub client_data: db::ClientData,
}


#[derive(Template)]
#[template(path ="register.html")]
pub struct RegisterTemplate {
    pub texts: RegisterTexts,
    pub user: auth::UserReqData,
    pub client_refs: Vec<db::ClientRef>,
    pub selected_client_id: String,
    pub querystring: String,
    pub agreements: AgreementTexts,
}


#[derive(Template)]
#[template(path ="error.html")]
pub struct ErrorTemplate {
    pub error_data: ErrorData,
    pub user: auth::UserReqData,
    pub texts: ErrorTexts,
}


#[derive(Template)]
#[template(path ="dashboard.html")]
pub struct DashboardTemplate<'a> {
    pub texts: DashboardTexts,
    pub user_data: &'a db::User,
    pub user: auth::UserReqData,
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
 * =====  HELPER FUNCTIONS  =====
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

pub async fn get_server_error(req: &HttpRequest) -> HttpResponse {
    // Worse than not finding something. Something broke.
    let code: u16 = 500;
    let lang: &utils::SupportedLangs = &auth::get_user_req_data(req).clone_lang();
    let error: String = error_by_code(code.to_string(), &lang).to_string();
    HttpResponse::InternalServerError().json(ErrorResponse { error, code })
    
}

/**
 * Login and Register both end up with this authentication flow.
 */
pub async fn authenticate_user_response(
    req: HttpRequest,
    user: db::User,
    pool: web::Data<MySqlPool>,
    client_id: String,
    go_to_dash: bool
) -> HttpResponse {
    // get cookies for local login
    let two_auth_cookies: TwoAuthCookies =
        match get_user_auth_cookies(&pool, &user).await {
            Ok(cookies) => cookies,
            Err(error_response) => 
                return HttpResponse::InternalServerError()
                    .json(error_response)
        };

    /* 
     * Either log the user in locally,
     * or send them to their chosen client site.
     */
    if client_id == utils::auth_client_id() {
        if go_to_dash {
            // redirect to DASHBOARD
            HttpResponse::Found()
                .cookie(two_auth_cookies.jwt_cookie)
                .cookie(two_auth_cookies.refresh_token_cookie)
                .append_header((header::LOCATION, "/dashboard"))
                .finish()
        } else {
            // client_id is auth_site login now and redirect
            // User may now receive JWT and refresh token.
            HttpResponse::Ok()
                .cookie(two_auth_cookies.jwt_cookie)
                .cookie(two_auth_cookies.refresh_token_cookie)
                .json(FreshLoginData {
                    username: user.get_username().to_owned()
            })
        }
    } else {
        // It's an external site. So let's get an auth_token and redirect
        post_auth_client_site_redirect(
            req, user.get_id(),
            pool, client_id,
            Some(two_auth_cookies)
        ).await
    }
}


/**
 * We have checked the user's login credentials.
 * The user wants to login on an external site.
 * We will STILL log them in locally, but then also send them
 * to the client site.
 */
pub async fn post_auth_client_site_redirect(
    req: HttpRequest,
    user_id: i32,
    pool: web::Data<MySqlPool>,
    client_id: String,
    cookies_option: Option<TwoAuthCookies>
) -> HttpResponse {
    let server_error: HttpResponse = get_server_error(&req).await;

    let auth_code: String = match db::add_auth_code(
        &pool,
        user_id,
        &client_id,
        auth::generate_auth_code()
    ).await {
        Ok(code) => code,
        Err(_e) => return server_error
    };

    let redirect_uri_option: Option<String>  =
        match db::get_redirect_uri(&pool, &client_id).await {
            Ok(option) => option,
            Err(_e) => return server_error
    };

    match redirect_uri_option {
        Some(redirect_uri) => {
            /* we have the code and the redirect_uri.
             * Build the full URL with querystring and send to frontend for redirect. */
            let query_key_string: &str = "?code=";
            let full_uri: FullRedirectUri = FullRedirectUri {
                redirect_uri: format!("{}{}{}",
                    &redirect_uri,
                    query_key_string,
                    &auth_code
            )};

            // Do we need to set cookies?
            match cookies_option {
                Some(two_auth_cookies) => {
                    // Set local cookies.
                    HttpResponse::Ok()
                        .cookie(two_auth_cookies.jwt_cookie)
                        .cookie(two_auth_cookies.refresh_token_cookie)
                        .json(full_uri)
                },
                None => HttpResponse::Ok().json(full_uri)
            }
        },
        None => {
            let code: u16 = 404;
            let lang: &utils::SupportedLangs =
                &auth::get_user_req_data(&req).clone_lang();
            let error: String = get_translation(
                "err.404.title", &lang, None);
            HttpResponse::NotFound().json(ErrorResponse { error, code })
        }
    }
}


/**
 * Rather than rewrite this over and over, for situations where a guest tries to access
 * restricted pages, this function returns the login route in an HttpResponse.
 */
pub fn send_to_login() -> HttpResponse {
    HttpResponse::Found()
        .append_header((header::LOCATION, "/auth/login"))
        .finish()
}

pub fn non_admin_rejection(req: &HttpRequest) -> HttpResponse {
    // send to unauthorized error page
    return redirect_to_err(String::from("403"))
        .respond_to(req)
        .map_into_boxed_body();
}


/**
 * Redirect to error page with a simple and easy function
 */
pub fn return_error_page(req: &HttpRequest, code: u16) -> HttpResponse {
    return redirect_to_err(code.to_string())
        .respond_to(req)
        .map_into_boxed_body();
}


/**
 * We often check if the user is admin.
 * This returns the appropriate redirect depending on
 * which kind of non-admin the user is.
 */
pub fn redirect_non_admin(
    user_req_data: &UserReqData,
    req: &HttpRequest
) -> Option<HttpResponse> {
    // Send guest to login
    if user_req_data.id.is_none() {
        return Some(send_to_login());
    }

    // If they're not admin send them to error page
    if !user_req_data.is_admin() {
        return Some(non_admin_rejection(&req));
    }

    // The user is an admin
    None
}


/**
 * Sometimes we don't know what went wrong and we need to return a JSON
 * object which says so.
 */
pub fn return_internal_err_json() -> HttpResponse {
    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .json(ErrorResponse{
            error: String::from("Internal server error"),
            code: 500
        })
}

// If authentication failed and user must log back in
pub fn return_authentication_err_json() -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse{
        error: String::from("Authentication required"),
        code: 401
    })
}


// If something is not found
pub fn return_not_found_err_json() -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse{
        error: String::from("Not Found"),
        code: 406
    })
}

fn error_post_struct(req: &HttpRequest, code: u16) -> ErrorResponse {
    let lang: &utils::SupportedLangs = &auth::get_user_req_data(&req).clone_lang();
    let code_str: String = code.to_string();
    let keyword: String = format!("err.{}.title", code_str);
    let error: String = get_translation(&keyword, &lang, None);

    ErrorResponse { code, error }
}

pub fn error_post_response(req: &HttpRequest, code: u16) -> HttpResponse {
    let e_struct: ErrorResponse = error_post_struct(req, code);

    match code {
        500 => HttpResponse::InternalServerError().json(e_struct),
        404 => HttpResponse::NotFound().json(e_struct),
        401 => HttpResponse::Unauthorized().json(e_struct),
        403 => HttpResponse::Forbidden().json(e_struct),
        400 => HttpResponse::BadRequest().json(e_struct),
        429 => HttpResponse::TooManyRequests().json(e_struct),
        _ => HttpResponse::BadRequest().json(e_struct),
    }
}


/**
 * We only do this once the user has been authenticated.
 * Calls functions to generate JWT and refresh token,
 * puts them in cookies and sends the response,
 * along with some user info in a JSON.
 * 
 * 
 */
pub async fn get_user_auth_cookies(
    pool: &MySqlPool,
    user: &db::User
) -> Result<TwoAuthCookies, ErrorResponse> {
    // generate JWT. Don't send user obj (with password) back
    let jwt_err_500: ErrorResponse = ErrorResponse {
        error: String::from("Access Token Generation Error."),
        code: 500
    };

    // Generate a token String
    let jwt: String = match auth::generate_jwt(
        user.get_id(),
        user.get_username().to_owned(),
        user.get_role().to_owned(),
        user.get_email_verified()
    ) {
        Ok(token) => token,
        Err(e) => {
            eprint!("Internal Server Error: {e}");
            return Err(jwt_err_500)
        }
    };

    // create a refresh_token and put it in the DB
    match db::add_refresh_token(
        &pool,
        user.get_id(),
        utils::auth_client_id(),
        auth::generate_refresh_token()
    ).await {
        Ok(refresh_token) => {
            // Refresh token successfully inserted into DB
            // Now make the cookies and set them in the response
            let jwt_cookie: Cookie<'_> = auth::build_token_cookie(
                jwt,
                String::from("jwt"));
            
            let refresh_token_cookie: Cookie<'_> = auth::build_token_cookie(
                refresh_token,
                String::from("refresh_token"));
            
            let two_auth_cookies: TwoAuthCookies = TwoAuthCookies {
                jwt_cookie, refresh_token_cookie
            };

            Ok(two_auth_cookies)
        },
        Err(e) => {
            eprint!("Internal Server Error: {e}");
            Err(jwt_err_500)
        }
    }
}


/* 
 * 
 * 
 * 
 * 
 * =======================
 * =======================
 * =====             =====
 * =====  REDIRECTS  =====
 * =====             =====
 * =======================
 * =======================
 * 
 * 
 * 
 * 
*/


// if user just goes to /auth
pub fn redirect_to_err(err_code: String) -> impl Responder {
    Redirect::to(format!("/error/{}", err_code))
}
