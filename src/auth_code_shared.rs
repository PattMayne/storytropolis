use serde::{ Serialize, Deserialize };

/* 
 * 
 * 
 * 
 * 
 * ==================================================
 * ==================================================
 * ==================================================
 * ==================================================
 * ==================================================
 * ===============                    ===============
 * ===============  SHARED AUTH CODE  ===============
 * ===============                    ===============
 * ==================================================
 * ==================================================
 * ==================================================
 * ==================================================
 * ==================================================
 * 
 * 
 * 
 * CODE shared between auth_app and client apps.
 * Structs which backend functions on auth_app sends to
 * the backend calling-functions of client apps.
 * 
 * Client expects specific success responses,
 * or specific errors. So that's what auth_app sends.
 * 
 * 
*/



// IO STRUCTURES FOR CHECKING AUTH CODES AND GETTING REFRESH TOKENS
// EACH CLIENT APP MUST ALSO HAVE THESE
// SO THESE SHOULD ACTUALLY GO IN THEIR OWN MODULE.

#[derive(Serialize, Deserialize)]
pub struct AuthCodeRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
}


#[derive(Serialize, Deserialize)]
pub struct AuthCodeSuccess {
    pub user_id: i32,
    pub username: String,
    pub user_role: String,
    pub refresh_token: String,
    pub email_verified: bool,
}


#[derive(Serialize, Deserialize)]
pub struct AuthCodeError {
    pub error_code: u16,
    pub message: String,
}

/* Unified response type enum.
 * "untagged" means the data within the struct will be 
 * directly available (NOT within type: err or whatever).
 * 
 * Putting longest structs FIRST b/c serde matches by
 * presence of all field names.
 */
#[derive(Serialize, Deserialize)]
#[serde(untagged)] 
pub enum AuthCodeResponse {
    Ok(AuthCodeSuccess),
    Err(AuthCodeError),
}



/* 
 * For sending refresh_tokens from client_app to auth_app,
 * and sending true/false validation back.
 */

#[derive(Serialize, Deserialize)]
pub struct RefreshCheckRequest {
    pub token: String,
    pub user_id: i32,
    pub client_id: String,
    pub client_secret: String,
}


#[derive(Serialize, Deserialize)]
pub struct RefreshCheckSuccess {
    is_valid: bool,
}


#[derive(Serialize, Deserialize)]
pub struct RefreshCheckError {
    pub error_code: u16,
    pub message: String,
}


#[derive(Serialize, Deserialize)]
#[serde(untagged)] 
pub enum RefreshCheckResponse {
    Ok(RefreshCheckSuccess),
    Err(RefreshCheckError)
}


impl RefreshCheckSuccess {
    pub fn new(is_valid: bool) -> Self {
        RefreshCheckSuccess { is_valid }
    }
}


/* 
 * 
 * 
 * 
 * 
 * ================================
 * ================================
 * =====                      =====
 * =====  EMAIL VERIFICATION  =====
 * =====                      =====
 * ================================
 * ================================
 * 
 * 
 * 
 * 
*/


#[derive(Serialize, Deserialize)]
pub struct SendVerificationEmailRequest {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub user_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SendVerificationEmailResponse {
    pub success: bool,
    pub message: String,
    pub user_id: i32,
}

