use phf::phf_map;

use crate::utils::SupportedLangs;

/* 
 * 
 * 
 * 
 * 
 * =======================
 * =======================
 * =====             =====
 * =====  RESOURCES  =====
 * =====             =====
 * =======================
 * =======================
 * 
 * 
 * Text strings and functions to retrieve them.
 * Text is stored in static string references in a phf_map constant.
 * They can be retrieved by calling their keys.
 * 
 * We will have French and English versions of everything.
 * 
 * 
 * 
*/


/**
 * Text strings to use on website.
 * Placeholders must be NUMBERED starting with ZERO:
 * ie:  Hello, {0}! I hope you're having a good {1}!
 * --> where {0} and {1} can later be replaced with username and Morning/Afternoon
 */
pub static TRANSLATIONS: phf::Map<&'static str, &'static str> = phf_map! {

    // HOME PAGE
    "home.title.fr" => "Crankade",
    "home.title.en" => "Crankade",
    "home.greeting.en" => "Hello, {0}!",
    "home.greeting.fr" => "Bonjour, {0}!",

    // DASHBOARD PAGE
    "dash.title.fr" => "TABLEAU DE BORD",
    "dash.title.en" => "DASHBOARD",
    "dash.greeting.en" => "Edit your details, {0}!",
    "dash.greeting.fr" => "Modifiez vos informations, {0}!",
    // DASHBOARD LABELS
    "dash.firstname.en" => "First Name:",
    "dash.firstname.fr" => "Prénom:",
    "dash.lastname.en" => "Last Name:",
    "dash.lastname.fr" => "Nom:",
    "dash.password1.en" => "New Password:",
    "dash.password1.fr" => "Nouveau mot de passe :",
    "dash.password2.en" => "Confirm Password:",
    "dash.password2.fr" => "Confirmer le mot de passe:",
    // DASHBOARD BUTTONS
    "dash.updatenames.btn.en" => "UPDATE NAMES",
    "dash.updatenames.btn.fr" => "MAJ NOMS",
    "dash.updatepass.btn.en" => "UPDATE PASSWORD",
    "dash.updatepass.btn.fr" => "MAJ MOT DE PASSE",

    // ADMIN DASHBOARD PAGE
    "admin.title.en" => "ADMIN HOME",
    "admin.title.fr" => "TABLEAU DE BORD ADMIN",
    "admin.message.en" => "Perform admin actions",
    "admin.message.fr" => "Effectuer des actions administratives",
    "admin.actions.label.en" => "ADMIN ACTIONS",
    "admin.actions.label.fr" => "ACTIONS ADMIN",
    "admin.editclients.label.en" => "EDIT CLIENT SITES",
    "admin.editclients.label.fr" => "MODIFIER LES SITES CLIENTS",
    "admin.newclient.btn.en" => "ADD NEW CLIENT",
    "admin.newclient.btn.fr" => "AJOUTEZ SITE CLIENT",


    // LOGIN PAGE
    "login.title.en" => "LOGIN",
    "login.title.fr" => "ACCUEIL",
    "login.message.en" => "Please Log In",
    "login.message.fr" => "Veuillez vous connecter",
    // LOGIN LABELS
    "login.username.email.label.en" => "Username or Email:",
    "login.username.email.label.fr" => "Nom d'utilisateur ou e-mail:",
    "login.password.label.en" => "Password:",
    "login.password.label.fr" => "Mot de passe:",
    "login.client_site.label.en" => "Destination Site:",
    "login.client_site.label.fe" => "Site de destination:",
    // LOGIN BUTTONS
    "login.btn.en" => "LOGIN",
    "login.btn.fr" => "ACCUEIL",

    // REGISTER PAGE
    "register.title.en" => "REGISTER",
    "register.title.fr" => "INSCRIPTION",
    "register.message.en" => "Please register",
    "register.message.fr" => "Veuillez vous connecter",
    // REGISTER LABELS
    "register.username.label.en" => "Username:",
    "register.username.label.fr" => "Nom d'utilisateur:",
    "register.email.label.en" => " Email:",
    "register.email.label.fr" => "E-mail:",
    "register.password.label.en" => "Password:",
    "register.password.label.fr" => "Mot de passe:",
    "register.empty.label.en" => "Leave this field empty:",
    "register.empty.label.fr" => "Laissez ce champ vide:",
    // REGISTER BUTTONS
    "register.btn.en" => "REGISTER",
    "register.btn.fr" => "INSCRIPTION",

    // EMAIL VERIFY PAGE
    "verify.title.en" => "Enter Verification Code",
    "verify.title.fr" => "Saisissez le code de vérification",
    "verify.email.label.en" => " Email:",
    "verify.email.label.fr" => "E-mail:",
    "verify.code.en" => "Verification Code",
    "verify.code.fr" => "Code de vérification",
    "verify.submit_btn.en" => "Submit",
    "verify.submit_btn.fr" => "Soumettre",
    "verify.empty.label.en" => "Leave this field empty:",
    "verify.empty.label.fr" => "Laissez ce champ vide:",
    "verify.message.en" => "Enter your email and your verification code.",
    "verify.message.fr" => "Saisissez votre adresse e-mail et votre code de vérification.",

    // REQUEST VERIFICATION TEXTS
    "reqver.title.en" => "Request Email Login",
    "reqver.title.fr" => "Request Email Login",
    "reqver.message.en" => "Request Email Login. Then you can verify your email and/or reset your password.",
    "reqver.message.fr" => "Request Email Login. Then you can verify your email and/or reset your password.",
    "reqver.email.en" => "Email Address",
    "reqver.email.fr" => "Email Address",
    "reqver.submit_btn.en" => "Submit",
    "reqver.submit_btn.fr" => "Submit",

    // DEV BLOG
    "blog.title.en" => "DEVELOPER BLOG",
    "blog.title.fr" => "BLOG DU DÉVELOPPEUR",

    // NEW CLIENT SITE
    "new_client.title.en" => "NEW CLIENT SITE",
    "new_client.title.fr" => "NOUVEAU SITE CLIENT",
    "new_client.message.en" => "Add a new client site to the network.",
    "new_client.message.fr" => "Ajoutez un nouveau site client au réseau.",

    // EDIT CLIENT SITE
    "edit_client.title.en" => "EDIT CLIENT SITE",
    "edit_client.title.fr" => "MODIFIER LE SITE DU CLIENT",
    "edit_client.message.en" => "Update existing client.",
    "edit_client.message.fr" => "Mettez à jour le client existant.",

    // NEW POST
    "new_post.title.en" => "NEW BLOG POST",
    "new_post.title.fr" => "NOUVEAU POST D'BLOGUETTE !",

    // EDIT POST
    "edit_post.title.en" => "EDIT BLOG POST",
    "edit_post.title.fr" => "EDIT POST D'BLOGUETTE !",

    // CLIENT FORM LABELS (for both NEW and EDIT)
    "clientform.domain.en" => "Site Domain:",
    "clientform.domain.fr" => "Domaine du site:",
    "clientform.name.en" => "Site Name (public-facing title):",
    "clientform.name.fr" => "Nom du site (titre public) :",
    "clientform.id.en" => "Client ID (random unique identifier):",
    "clientform.id.fr" => "Client ID (unique identifier aléatoire):",
    "clientform.red_uri.en" => "Redirect URI:",
    "clientform.red_uri.fr" => "Redirection URI :",
    "clientform.logo_url.en" => "Logo URL:",
    "clientform.logo_url.fr" => "URL du logo :",
    "clientform.type.en" => "Type:",
    "clientform.type.fr" => "Type:",
    "clientform.cat.en" => "Category:",
    "clientform.cat.fr" => "Catégorie:",
    "clientform.desc.en" => "Description:",
    "clientform.desc.fr" => "Description:",
    "clientform.isactive.en" => "Is Active:",
    "clientform.isactive.fr" => "Est actif:",
    // CLIENT FORM BUTTONS
    "clientform.submit.en" => "SUBMIT",
    "clientform.submit.fr" => "SUBMIT",
    "clientform.save_changes.en" => "SUBMIT",
    "clientform.save_changes.fr" => "ENVOYER",
    "clientform.gen_secret.en" => "GENERATE NEW SECRET",
    "clientform.gen_secret.fr" => "GÉNÉRER UN NOUVEAU SECRET",

    // NAV BUTTONS
    "nav.home.en" => "HOME",
    "nav.home.fr" => "ACCUEIL",
    "nav.blog.en" => "BLOG",
    "nav.blog.fr" => "BLOG",
    "nav.admin.en" => "ADMIN",
    "nav.admin.fr" => "ADMIN",
    "nav.login.en" => "LOGIN",
    "nav.login.fr" => "CONNEXION",
    "nav.register.en" => "REGISTER",
    "nav.register.fr" => "INSCRIPTION",
    "nav.logout.en" => "LOGOUT",
    "nav.logout.fr" => "DÉCONNEXION",
    "nav.dashboard.en" => "DASHBOARD",
    "nav.dashboard.fr" => "TABLEAU DE BORD",

    // TERMS & CONDITIONS ETC (using HTML for actual body)

    "terms.label.en" => "I have read and agree to the&nbsp;",
    "terms.label.fr" => "J'ai lu et accepte les&nbsp;",
    "terms.linktext.en" => "Terms and Conditions",
    "terms.linktext.fr" => "Conditions générales",

    "privacy.label.en" => "I have read and agree to the&nbsp;",
    "privacy.label.fr" => "J'ai lu et accepte la&nbsp;",
    "privacy.linktext.en" => "Privacy Policy",
    "privacy.linktext.fr" => "Politique de confidentialité",

    "emailconsent.label.en" => "I consent to receive essential emails for account management.",
    "emailconsent.label.fr" => "J'accepte de recevoir des emails essentiels pour la gestion du compte.",

    // ERROR CODES AND TITLES FOR ERROR PAGE
    "err.400.title.en" => "Bad Request",
    "err.400.title.fr" => "Mauvaise demande",
    "err.400.body.en" => "The request was malformed or otherwise bad.",
    "err.400.body.fr" => "La demande était mal formulée ou autrement mauvaise.",

    "err.401.title.en" => "Unauthorized",
    "err.401.title.fr" => "Non autorisé",
    "err.401.body.en" => "User is not authenticated.",
    "err.401.body.fr" => "L'utilisateur n'est pas authentifié.",

    "err.403.title.en" => "Forbidden",
    "err.403.title.fr" => "Interdit",
    "err.403.body.en" => "You do not have permission to view this page.",
    "err.403.body.fr" => "Vous n'avez pas la permission de consulter cette page.",

    "err.404.title.en" => "Not Found",
    "err.404.title.fr" => "Non trouvé",
    "err.404.body.en" => "The page you are looking for was not found.",
    "err.404.body.fr" => "La page que vous cherchez n'a pas été trouvée.",

    "err.408.title.en" => "Request Timeout",
    "err.408.title.fr" => "Délai de demande",
    "err.408.body.en" => "Server is shutting down connection.",
    "err.408.body.fr" => "Le serveur coupe la connexion.",

    "err.409.title.en" => "Conflict",
    "err.409.title.fr" => "Conflit",
    "err.409.body.en" => "Unacceptable duplicate input.",
    "err.409.body.fr" => "Entrée doublée inacceptable.",

    "err.422.title.en" => "Unprocessable Content",
    "err.422.title.fr" => "Contenu non traitable",
    "err.422.body.en" => "Request was well formed but content contains semantic errors.",
    "err.422.body.fr" => "La demande était bien formulée, mais le contenu contient des erreurs sémantiques.",

    "err.429.title.en" => "Too Many Requests",
    "err.429.title.fr" => "Trop de demandes",
    "err.429.body.en" => "User has sent too many requests.",
    "err.429.body.fr" => "L'utilisateur a envoyé trop de demandes.",

    "err.500.title.en" => "Internal Server Error",
    "err.500.title.fr" => "Erreur interne du serveur",
    "err.500.body.en" => "An unexpected error occurred.",
    "err.500.body.fr" => "Une erreur inattendue s'est produite.",

    "err.502.title.en" => "Bad Gateway",
    "err.502.title.fr" => "Mauvaise passerelle",
    "err.502.body.en" => "Gateway server received an invalid response.",
    "err.502.body.fr" => "Le serveur passerelle a reçu une réponse invalide.",

    "err.503.title.en" => "Service Unavailable",
    "err.503.title.fr" => "Service indisponible",
    "err.503.body.en" => "Server is not ready to handle the request. Please check back later.",
    "err.503.body.fr" => "Le serveur n'est pas prêt à gérer la demande. Veuillez revenir plus tard.",

    "err.504.title.en" => "Gateway Timeout",
    "err.504.title.fr" => "Délai d'attente de la passerelle",
    "err.504.body.en" => "Server did not respond in time.",
    "err.504.body.fr" => "Le serveur n'a pas répondu à temps.",

    // AD-HOC ERRORS FOR JSON
    "err.empty_creds.en" => "Invalid Credentials: Empty Field.",
    "err.empty_creds.fr" => "Identifiants invalides : champ vide.",
    "err.invalid_creds.en" => "Invalid Credentials.",
    "err.invalid_creds.fr" => "Identifiants invalides.",
    "err.user_not_found.en" => "User not found.",
    "err.user_not_found.fr" => "Utilisateur non trouvé.",

    // FRONT PAGE DEFAULT TEXT
    "frontpage.en" => "CRANKADE is the central hub for where I (Matt Payne) will host any web apps and games that I want to share with you. Right now there's just one: CRANKWORD, a multi-player, turn-based word-guessing game. Please enjoy!",
    "frontpage.fr" => "CRANKADE est le centre central où je (Matt Payne) hébergerai toutes les applications web et jeux que je souhaite partager avec vous. Pour l'instant, il n'y en a qu'un seul : CRANKWORD, un jeu multijoueur au tour par tour où on devine des mots. Bonne lecture !"
};


/**
 * For missing translations, or mis-typed keys.
 */
fn missing_trans(lang: &SupportedLangs) -> &'static str {
    match lang {
        SupportedLangs::English => "[ translation missing ]",
        SupportedLangs::French => "[ traduction manquante ]"
    }
}


/**
 * Take the keyword for the translation to call,
 * a language enum so we know the language suffix to add,
 * and an optional set of &str slices for placeholder phrases
 */
pub fn get_translation(
    keyword: &str,
    lang: &SupportedLangs,
    params_option: Option<&[&str]>
) -> String {
    let full_key: String = format!("{}.{}", keyword, lang.suffix());
    
    match TRANSLATIONS.get(full_key.as_str()) {
        Some(translation) => {
            let translation: String = translation.to_string();

            // replace placeholders with text from args
            match params_option {
                None => translation,
                Some(args) => {
                    let mut translation: String = translation;
                    for (i, arg) in args.iter().enumerate() {
                        let placeholder: String = format!("{{{}}}", i);
                        translation = translation.replace(&placeholder, arg);
                    }
                    translation
                }
            }
        },
        None => missing_trans(lang).to_string()
    }
}


/**
 * A quick and dirty retrieval of translations
 * which do NOT have placeholders.
 * Primarily for the nav translations.
 */
pub fn raw_trans_or_missing(
    key: &str,
    lang: &SupportedLangs
) -> &'static str {
    match TRANSLATIONS.get(key) {
        Some(translation) => translation,
        None => missing_trans(lang)
    }
}
