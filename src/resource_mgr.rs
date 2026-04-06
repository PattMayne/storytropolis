/* 
 * 
 * 
 * 
 * 
 * ==============================
 * ==============================
 * =====                    =====
 * =====  RESOURCE MANAGER  =====
 * =====                    =====
 * ==============================
 * ==============================
 * 
 * 
 * Gather translations into structs in this script
 * to keep that logic out of the routes script.
 * 
 * Some pages or templates require custom functions
 * to build their structs.
 * Most can simply use the get_translation function.
 * 
 * 
*/


use crate::{
    auth::UserReqData,
    resources::{ get_translation, raw_trans_or_missing, TRANSLATIONS },
    utils::SupportedLangs
};


/* 
 * 
 * 
 * 
 * 
 * ==================================
 * ==================================
 * =====                        =====
 * =====  TRANSLATIONS STRUCTS  =====
 * =====                        =====
 * ==================================
 * ==================================
 * 
 * 
 * Each askama template will have a struct
 * designed to hold all necessary text.
 * 
 * 
 * 
*/



/**
 * route: get "/"
 */
pub struct HomeTexts {
    pub title: String,
    pub message: String,
    pub default_pinned: String,
    pub nav: NavTexts,
}

impl HomeTexts {
    pub fn new(user_req_data: &UserReqData) -> HomeTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let default_pinned: String = get_translation("frontpage", lang, None);
        let title: String = get_translation("home.title", lang, None);
        let message: String = get_translation(
            "home.greeting",
            lang,
            Some(&[&user_req_data.get_role()]));
        let nav: NavTexts = NavTexts::new(lang);

        HomeTexts {
            title,
            message,
            default_pinned,
            nav
        }
    }
}

/**
 * route: get "/request_verification"
 */
pub struct ReqVerificationTexts {
    pub title: String,
    pub message: String,
    pub email: String,
    pub submit_btn: String,
    pub nav: NavTexts,
}

impl ReqVerificationTexts {
    pub fn new(user_req_data: &UserReqData) -> ReqVerificationTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation("reqver.title", lang, None);
        let message: String = get_translation("reqver.message", lang, None);
        let email: String = get_translation("reqver.email", lang, None);
        let submit_btn: String = get_translation("reqver.submit_btn", lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        ReqVerificationTexts {
            title,
            message,
            email,
            submit_btn,
            nav
        }
    }
}


/**
 * route: get "/login"
 */
pub struct LoginTexts {
    pub title: String,
    pub message: String,
    pub username_email: String,
    pub password: String,
    pub login_btn: String,
    pub sites: String,
    pub nav: NavTexts,
}

impl LoginTexts {
    pub fn new(user_req_data: &UserReqData) -> LoginTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation("login.title", lang, None);
        let message: String = get_translation("login.message", lang, None);
        let username_email: String = get_translation("login.username.email.label", lang, None);
        let password: String = get_translation("login.password.label", lang, None);
        let login_btn: String = get_translation("login.btn", lang, None);
        let sites: String = get_translation("login.client_site.label", lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        LoginTexts {
            title,
            message,
            username_email,
            password,
            login_btn,
            sites,
            nav,
        }
    }
}



/**
 * route: get "/register"
 */
pub struct RegisterTexts {
    pub title: String,
    pub message: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub register_btn: String,
    pub empty_field: String,
    pub sites: String,
    pub nav: NavTexts,
}

impl RegisterTexts {
    pub fn new(user_req_data: &UserReqData) -> RegisterTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation("register.title", lang, None);
        let message: String = get_translation("register.message", lang, None);
        let username: String = get_translation("register.username.label", lang, None);
        let email: String = get_translation("register.email.label", lang, None);
        let password: String = get_translation("register.password.label", lang, None);
        let empty_field: String = get_translation("register.empty.label", lang, None);
        let register_btn: String = get_translation("register.btn", lang, None);
        let sites: String = get_translation("login.client_site.label", lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        RegisterTexts {
            title,
            message,
            username,
            email,
            password,
            empty_field,
            register_btn,
            nav,
            sites
        }
    }
}



/**
 * route: get "/admin"
 */
pub struct AdminTexts {
    pub title: String,
    pub message: String,
    pub actions_label: String,
    pub new_client_btn: String,
    pub edit_clients_label: String,
    pub nav: NavTexts
}

impl AdminTexts {
    pub fn new(user_req_data: &UserReqData) -> AdminTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation("admin.title", lang, None);
        let message: String = get_translation("admin.message", lang,None);
        let actions_label: String = get_translation("admin.actions.label", lang,None);
        let new_client_btn: String = get_translation("admin.newclient.btn", lang,None);
        let edit_clients_label: String = get_translation("admin.editclients.label", lang,None);
        let nav = NavTexts::new(lang);

        AdminTexts {
            title,
            message,
            nav,
            actions_label,
            new_client_btn,
            edit_clients_label,
        }
    }
}


/**
 * route: get "/admin/new_post"
 */
pub struct NewPostTexts {
    pub title: String,
    pub nav: NavTexts,
}


impl NewPostTexts {
    pub fn new(user_req_data: &UserReqData) -> NewPostTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation(
            "new_post.title", &user_req_data.lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        NewPostTexts {
            title,
            nav
        }
    }
}

/**
 * route: get "/admin/edit_post/{id}"
 */
pub struct EditPostTexts {
    pub title: String,
    pub nav: NavTexts,
}


impl EditPostTexts {
    pub fn new(user_req_data: &UserReqData) -> EditPostTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation(
            "edit_post.title", &user_req_data.lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        EditPostTexts { title, nav }
    }
}


/**
 * route: get "/verify"
 */
pub struct VerifyTexts {
    pub title: String,
    pub email: String,
    pub code: String,
    pub submit_btn: String,
    pub empty_field: String,
    pub message: String,
    pub nav: NavTexts,
}


impl VerifyTexts {
    pub fn new(user_req_data: &UserReqData) -> VerifyTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation(
            "verify.title", &user_req_data.lang, None);
        let email: String = get_translation(
            "verify.email.label", &user_req_data.lang, None);
        let code: String = get_translation(
            "verify.code", &user_req_data.lang, None);
        let submit_btn: String = get_translation(
            "verify.submit_btn", &user_req_data.lang, None);
        let empty_field: String = get_translation(
            "verify.empty.label", &user_req_data.lang, None);
        let message: String = get_translation(
            "verify.message.label", &user_req_data.lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        VerifyTexts { title, email, code, submit_btn, nav, empty_field, message }
    }
}






/**
 * route: get "/blog"
 */
pub struct BlogTexts {
    pub title: String,
    pub nav: NavTexts
}

impl BlogTexts {
    pub fn new(user_req_data: &UserReqData) -> BlogTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation("blog.title", lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        BlogTexts {
            title,
            nav
        }
    }
}



/**
 * route: get "/admin/new_client"
 */
pub struct NewClientTexts {
    pub title: String,
    pub message: String,
    pub domain: String,
    pub name: String,
    pub id: String,
    pub red_uri: String,
    pub logo_url: String,
    pub cli_type: String,
    pub cat: String,
    pub desc: String,
    pub is_active: String,
    pub submit_btn: String,
    pub nav: NavTexts,
}

impl NewClientTexts {
    pub fn new(user_req_data: &UserReqData) -> NewClientTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation("new_client.title", &user_req_data.lang, None);
        let message: String = get_translation("new_client.message",lang,None);
        let domain: String = get_translation("clientform.domain",lang,None);
        let name: String = get_translation("clientform.name",lang,None);
        let id: String = get_translation("clientform.id",lang,None);
        let red_uri: String = get_translation("clientform.red_uri",lang,None);
        let logo_url: String = get_translation("clientform.logo_url",lang,None);
        let cli_type: String = get_translation("clientform.type", lang, None);
        let cat: String = get_translation("clientform.cat", lang, None);
        let desc: String = get_translation("clientform.desc", lang, None);
        let is_active: String = get_translation("clientform.isactive", lang, None);
        let submit_btn: String = get_translation("clientform.submit", lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        NewClientTexts {
            title,
            message,
            domain,
            name,
            id,
            red_uri,
            logo_url,
            cli_type,
            cat,
            desc,
            is_active,
            submit_btn,
            nav
        }
    }
}


/**
 * route: get "/admin/edit_client"
 */
pub struct EditClientTexts {
    pub title: String,
    pub message: String,
    pub domain: String,
    pub name: String,
    pub id: String,
    pub red_uri: String,
    pub logo_url: String,
    pub cli_type: String,
    pub cat: String,
    pub desc: String,
    pub is_active: String,
    pub save_btn: String,
    pub new_scret_btn: String,
    pub nav: NavTexts
}

impl EditClientTexts {
    pub fn new(user_req_data: &UserReqData) -> EditClientTexts {
        let lang: &SupportedLangs = &user_req_data.lang;
        let title: String = get_translation("edit_client.title", &user_req_data.lang, None);
        let message: String = get_translation("edit_client.message", lang, None);
        let domain: String = get_translation("clientform.domain", lang, None);
        let name: String = get_translation("clientform.name", lang, None);
        let id: String = get_translation("clientform.id", lang, None);
        let red_uri: String = get_translation("clientform.red_uri", lang, None);
        let logo_url: String = get_translation("clientform.logo_url", lang, None);
        let cli_type: String = get_translation("clientform.type", lang, None);
        let cat: String = get_translation("clientform.cat", lang, None);
        let desc: String = get_translation("clientform.desc", lang, None);
        let is_active: String = get_translation("clientform.isactive", lang, None);
        let save_btn: String = get_translation("clientform.save_changes", lang, None);
        let new_scret_btn: String = get_translation("clientform.gen_secret", lang, None);
        let nav: NavTexts = NavTexts::new(lang);


        EditClientTexts {
            title,
            message,
            domain,
            name,
            id,
            red_uri,
            logo_url,
            cli_type,
            cat,
            desc,
            is_active,
            save_btn,
            new_scret_btn,
            nav
        }
    }
}


/**
 * route: get "/error"
 */
pub struct ErrorTexts {
    pub nav: NavTexts
}

impl ErrorTexts {
    pub fn new(user_req_data: &UserReqData) -> ErrorTexts {
        let nav: NavTexts = NavTexts::new(&user_req_data.lang);
        ErrorTexts { nav }
    }
}


/**
 * route: get "/auth/dashboard"
 */
pub struct DashboardTexts {
    pub title: String,
    pub message: String,
    pub first_name_label: String,
    pub last_name_label: String,
    pub password_label: String,
    pub confirm_password_label: String,
    pub update_names_btn: String,
    pub update_password_btn: String,
    pub nav: NavTexts
}

impl DashboardTexts {
    pub fn new(user_req_data: &UserReqData) -> DashboardTexts {
        let title: String = get_translation("dash.title", &user_req_data.lang, None);
        let lang: &SupportedLangs = &user_req_data.lang;

        let message: String = get_translation(
            "dash.greeting", lang, Some(&[&user_req_data.get_role()]));

        let first_name_label: String = get_translation("dash.firstname", lang,None);
        let last_name_label: String = get_translation("dash.lastname", lang,None);
        let password_label: String = get_translation("dash.password1",lang,None);
        let confirm_password_label: String = get_translation("dash.password2", lang, None);
        let update_names_btn: String = get_translation("dash.updatenames.btn", lang, None);
        let update_password_btn: String = get_translation("dash.updatepass.btn", lang, None);
        let nav: NavTexts = NavTexts::new(lang);

        DashboardTexts {
            title,
            message,
            first_name_label,
            last_name_label,
            password_label,
            confirm_password_label,
            update_names_btn,
            update_password_btn,
            nav
        }
    }
}



/* 
 * 
 * 
 * 
 * 
 * =====================
 * =====================
 * =====           =====
 * =====  TOP NAV  =====
 * =====           =====
 * =====================
 * =====================
 * 
 * 
 * 
 * 
 * The top-nav bar is loaded on every page, so here is a struct to gather
 * all of its button translations together.
 * They can be static references because they will never build by replacing
 * placeholders. Simple strings.
 */
pub struct NavTexts {
    pub home: &'static str,
    pub blog: &'static str,
    pub admin: &'static str,
    pub dashboard: &'static str,
    pub login: &'static str,
    pub register: &'static str,
    pub logout: &'static str,
}


impl NavTexts {

    /**
     * Just pass in a language to this constructor and get the right language version
     * of all the strings for the top-nav buttons.
     */
    pub fn new(lang: &SupportedLangs) -> NavTexts {
        let lang_suffix: &str = lang.suffix();

        let home_key: String = format!("{}.{}", "nav.home", lang_suffix);
        let blog_key: String = format!("{}.{}", "nav.blog", lang_suffix);
        let admin_key: String = format!("{}.{}", "nav.admin", lang_suffix);
        let dash_key: String = format!("{}.{}", "nav.dashboard", lang_suffix);
        let login_key: String = format!("{}.{}", "nav.login", lang_suffix);
        let register_key: String = format!("{}.{}", "nav.register", lang_suffix);
        let logout_key: String = format!("{}.{}", "nav.logout", lang_suffix);

        let home: &'static str = raw_trans_or_missing(home_key.as_str(), lang);
        let blog: &'static str = raw_trans_or_missing(blog_key.as_str(), lang);
        let admin: &'static str = raw_trans_or_missing(admin_key.as_str(), lang);
        let dashboard: &'static str = raw_trans_or_missing(dash_key.as_str(), lang);
        let login: &'static str = raw_trans_or_missing(login_key.as_str(), lang);
        let register: &'static str = raw_trans_or_missing(register_key.as_str(), lang);
        let logout: &'static str = raw_trans_or_missing(logout_key.as_str(), lang);

        NavTexts {
            home,
            admin,
            dashboard,
            login,
            register,
            logout,
            blog,
        }
    }
}


pub struct AgreementTexts {
    pub terms_label: &'static str,
    pub terms_link: &'static str,
    pub privacy_label: &'static str,
    pub privacy_link: &'static str,
    pub email_consent_label: &'static str,
}


impl AgreementTexts {

    /**
     * Just pass in a language to this constructor and get the right language version
     * of all the user agreements texts.
     */
    pub fn new(lang: &SupportedLangs) -> AgreementTexts {
        let lang_suffix: &str = lang.suffix();

        let terms_label_key: String = format!("{}.{}", "terms.label", lang_suffix);
        let terms_link_key: String = format!("{}.{}", "terms.linktext", lang_suffix);
        let privacy_label_key: String = format!("{}.{}", "privacy.label", lang_suffix);
        let privacy_link_key: String = format!("{}.{}", "privacy.linktext", lang_suffix);
        let email_consent_label_key: String = format!("{}.{}", "emailconsent.label", lang_suffix);

        let terms_label: &'static str = raw_trans_or_missing(terms_label_key.as_str(), lang);
        let terms_link: &'static str = raw_trans_or_missing(terms_link_key.as_str(), lang);
        let privacy_label: &'static str = raw_trans_or_missing(privacy_label_key.as_str(), lang);
        let privacy_link: &'static str = raw_trans_or_missing(privacy_link_key.as_str(), lang);
        let email_consent_label: &'static str = raw_trans_or_missing(email_consent_label_key.as_str(), lang);

        AgreementTexts {
            terms_label,
            terms_link,
            privacy_label,
            privacy_link,
            email_consent_label,
        }
    }
}



/* 
 * 
 * 
 * 
 * 
 * =========================
 * =========================
 * =====               =====
 * =====  ERROR CODES  =====
 * =====               =====
 * =========================
 * =========================
 * 
 * 
 * 
 * Custom logic to get Error page text.
 * The "custom" part is getting default data for
 * unknown or invalid error codes.
 * 
*/


// Text for Error page
pub struct ErrorData {
    pub code: String,
    pub title: &'static str,
    pub message: &'static str,
}

impl ErrorData {
    pub fn new(code: String, lang: &SupportedLangs) -> Self {
        let lang_suffix: &str = lang.suffix();
        let title_key: String = format!("{}.{}.{}.{}", "err", code, "title", lang_suffix);
        let body_key: String = format!("{}.{}.{}.{}", "err", code, "body", lang_suffix);

        // Get the option first so we can check if it's a known error code
        let title_option: Option<&&str> = TRANSLATIONS.get(title_key.as_str());
        let body_option: Option<&&str> = TRANSLATIONS.get(body_key.as_str());

        // Just hardcode the missing errors here
        if title_option.is_none() || body_option.is_none() {
            match lang {
                SupportedLangs::English => {
                    return ErrorData {
                        code: code,
                        title: "Unknown Error",
                        message: "An unknown error has occurred.",
                    };
                },
                SupportedLangs::French => {
                    return ErrorData {
                        code: code,
                        title: "Erreur inconnue",
                        message: "Une erreur inconnue s'est produite.",
                    };
                }
            }
        }

        // The error code is known, text is retrieved. Create and return struct.
        ErrorData {
            code: code,
            title: title_option.unwrap(),
            message: body_option.unwrap(),
        }
    }
}

fn missing_error(lang: &SupportedLangs) -> &'static str {
    match lang {
        SupportedLangs::English => "Error",
        SupportedLangs::French => "Erreur"
    }
}

/**
 * Uses the title of the Error Page error data for simple error messages.
 */
pub fn error_by_code(code: String, lang: &SupportedLangs) -> &'static str {
    let key: String = format!("{}.{}.{}.{}", "err", code, "title", lang.suffix());

    match TRANSLATIONS.get(&key) {
        Some(translation) => translation,
        None => missing_error(lang)
    }
}