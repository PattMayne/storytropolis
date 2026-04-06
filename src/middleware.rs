/* 
 * ========================
 * ========================
 * =====              =====
 * =====  MIDDLEWARE  =====
 * =====              =====
 * ========================
 * ========================
 * 
 * 
 * 
 * If multiple middleware functions are chained in the App::new() chain (inside a wrap() function)
 * they are each called in sequence, and they can each act upon the request and change the request.
 * In any function, post-processing can happen after the next.call(req).await call.
 * That post-processing happens AFTER all the later calls
 */

use actix_web::{
    web, error, Error, HttpMessage,
    body::MessageBody, dev::{ServiceRequest, ServiceResponse},
    middleware::{ Next }
};
use sqlx::{MySqlPool };

use crate::{ auth, db, utils };


pub struct NewJwtObj {
    token: String
}

impl NewJwtObj {
    pub fn new(token: String) -> Self {
        NewJwtObj { token }
    }

    pub fn get_token(&self) -> &String { &self.token }
}


/* 
 * 
 * 
 * 
 * 
 * ============================
 * ============================
 * =====                  =====
 * =====  PRE-PROCESSING  =====
 * =====                  =====
 * ============================
 * ============================
 * 
 * 
 * 
 * 
*/


/**
 * Pre-processing to make user data available for all routes.
 * Check for JSON web token in req's cookies, and validate the token.
 * Create a UserReqData object indicating whether user is logged in,
 * or a guest (based on whether JWT is valid).
 * Put that UserReqData object into the response for later functions.
*/
pub async fn login_status_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // Get the pool from app_data

    let pool = match req.app_data::<web::Data<MySqlPool>>() {
        Some(p) => p,
        None => return Err(error::ErrorInternalServerError("MYSQL pool error".to_string()))
    };


    let guest_data: auth::UserReqData = auth::UserReqData::new(None);
    let user_req_data_opt: Option<actix_web::cookie::Cookie<'_>> = req.cookie("jwt");
    let user_req_data: auth::UserReqData = get_user_req_data_from_opt(
        &pool,
        user_req_data_opt,
        &req,
        guest_data
    ).await?;

    // Put UserReqData into the request object to identify user to all routes.
    req.extensions_mut().insert(user_req_data);
    next.call(req).await
}



/**
 * Assists the login_status_middleware function by getting
 * user data for request (UserReqData).
 */
async fn get_user_req_data_from_opt(
    pool: &MySqlPool,
    option: Option<actix_web::cookie::Cookie<'_>>,
    req: &ServiceRequest,
    guest_data: auth::UserReqData
) -> Result<auth::UserReqData, Error> {
    // This was deeply nested match expressions, so we're checking for none/err instead

    if option.is_none() { return Ok(guest_data); }
    let jwt_cookie: actix_web::cookie::Cookie<'_> = option.unwrap();

    // Must use match here because of multiple enums
    match auth::verify_jwt(jwt_cookie.value()).await {
        auth::JwtVerification::Invalid => Ok(guest_data),
        auth::JwtVerification::Valid(claims) => {
            Ok(auth::UserReqData::new(Some(claims)))
        },
        auth::JwtVerification::Expired(claims) => {
            //println!("JWT expired. will check refresh token and generate new JWT");
            // JWT is expired but otherwise valid.
            // set an object in the req to send a new cookie
            /* 
             * PROCESS:
             * => check REFRESH TOKEN
             * => if that is valid (and non-expired):
             * ====> set FLAG for setting the new JWT
             * => ELSE
             * ====> set FLAG to make user log in again
            */
            
            // Check the cookies for a refresh_token
            let r_token_optn = req.cookie("refresh_token");
            if r_token_optn.is_none() { return Ok(guest_data); }
            let r_tkn_ckie: actix_web::cookie::Cookie<'_> = r_token_optn.unwrap();

            // check DB for refresh_token to compare
            let r_db_token_result: Result<Option<db::RefreshToken>, anyhow::Error> =
                db::get_refresh_token(
                    &pool,
                    claims.get_sub(),
                    utils::auth_client_id()
                ).await;

            if let Err(e) = r_db_token_result {
                return Err(error::ErrorInternalServerError(e.to_string()));
            }

            let r_db_token_option: Option<db::RefreshToken> = r_db_token_result.unwrap();
            if r_db_token_option.is_none() { return Ok(guest_data); }
            let r_db_token: db::RefreshToken = r_db_token_option.unwrap();

            let r_tkn_valid: bool = 
                r_tkn_ckie.value() == r_db_token.get_token() &&
                !r_db_token.is_expired();

            if r_tkn_valid {
                // CREATE and GIVE NEW JWT
                let new_jwt_rslt: Result<String, auth::AuthError> =
                    auth::generate_jwt(
                        claims.get_sub(),
                        claims.get_username().to_owned(),
                        claims.get_role().to_owned(),
                        claims.email_verified
                    );

                if let Err(e) = new_jwt_rslt {
                    return Err(error::ErrorInternalServerError(e.to_string()));
                }

                let new_jwt = new_jwt_rslt.unwrap();
                req.extensions_mut().insert(NewJwtObj::new(new_jwt));
                return Ok(auth::UserReqData::new(Some(claims)));
            } else {
                Ok(guest_data)
            }                   
        }
    }
}



/* 
 * 
 * 
 * 
 * 
 * =============================
 * =============================
 * =====                   =====
 * =====  POST-PROCESSING  =====
 * =====                   =====
 * =============================
 * =============================
 * 
 * 
 * 
 * 
*/


/**
 * Post-processing middleware to catch a "make new JWT" flag,
 * then make a new JWT and put it in a cookie in the response.
 */
pub async fn jwt_cookie_middleware<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> where B: MessageBody, {
    let mut res: ServiceResponse<B> = next.call(req).await?;

    let new_jwt: Option<String> = res
        .request()
        .extensions()
        .get::<NewJwtObj>()
        .map(|obj| obj.get_token().to_owned());

    // After handler, check for the NewJwt flag and add cookie if present
    if let Some(token) = new_jwt {
        let cookie: actix_web::cookie::Cookie<'_> =
            auth::build_token_cookie(
                token,
                String::from("jwt")
            );

        res.response_mut().add_cookie(&cookie).ok();
    }
    Ok(res)
}