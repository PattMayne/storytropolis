use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::{ Resend };
use sqlx::{ MySqlPool };
use askama::Template;

use crate::{ db, auth };



#[derive(Template)]
#[template(path ="verification_email.html")]
struct VerifyEmailTemplate {
    pub username: String,
    pub code: String,
    pub link: String,
}



pub async fn send_verification_email(
    pool: &MySqlPool,
    username: &str,
    user_id: i32,
    email_address: &str
) -> bool {
    // first get the verification code
    let new_verification_code: auth::NewVerificationCode = auth::NewVerificationCode::new(user_id);

    // save verification code to database
    let _code_saved: bool =
        match db::create_verification_code(&pool, &new_verification_code).await {
            Ok(rows_affected) => rows_affected > 0,
            Err(e) => {
                eprintln!("Database Error: Failed to save verification code: {:?}", e);
                false
            }
        };

    let verification_link: String = format!(
        "https://crankade.com/verify?email={}&code={}",
            email_address, &new_verification_code.raw_code
    );

    let template: VerifyEmailTemplate = VerifyEmailTemplate {
        username: username.to_string(),
        link: verification_link,
        code: new_verification_code.raw_code
    };

    let email_body: String = template.render().unwrap();

    // PROCESS:
    // 2. create verification route
    // 2    send LINK to verification route
    // 3. verification check updates DB
    // 4. Use askama template

    // put "create and send verification" in a function
    // user can send NEW verification
    // once per minute

    // non-verified accounts cannot create a new game
    //
    // "reset password" is really just "login through email"
    // verification link can STILL be used here to log user in
    let resend_api: String = match std::env::var("RESEND_API") {
        Ok(api) => api,
        Err(e) => {
            eprintln!("Email Error: Failed to retrieve API key: {:?}", e);
            return false
        }
    };

    let resend: Resend = Resend::new(&resend_api);

    let from: &str = "Crankade Admin <noreply@mail.crankade.com>";
    let to: [&str; 1] = [email_address];
    let subject: &str = "Welcome to Crankade!";

    let email_object: CreateEmailBaseOptions =
        CreateEmailBaseOptions::new(from, to, subject)
        .with_html(&email_body);

    match resend.emails.send(email_object).await {
        Ok(email_response) => println!("{:?}", email_response),
        Err(e) => eprintln!("Email Error: {:?}", e)
    };

    true
}