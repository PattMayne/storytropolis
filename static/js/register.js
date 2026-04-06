$(document).foundation()
import * as utils from './utils.js'

/**
 * Functions for the user registration input page.
 */
let err_msgs = []


/**
 * User has attempted to input registration details.
 * Gather all the user input, check if it's ok, send it to the backend.
 */
const submit_register = async () => {
    // reset error messages array with each attempt
    err_msgs = [];
    // get input elements
    const pass_element = document.getElementById("password")
    const username_element = document.getElementById("username")
    const email_element = document.getElementById("email")
    const client_id_element = document.getElementById("client_id")
    const website_element = document.getElementById("website")

    // agreements
    const terms_check = document.getElementById("terms_check")
    const privacy_check = document.getElementById("privacy_check")
    const email_consent_check = document.getElementById("email_consent_check")
    const all_checked = terms_check.checked && privacy_check.checked && email_consent_check.checked

    // get data (values) from input elements
    const creds = {
        password: pass_element.value.trim(),
        email: email_element.value.trim(),
        username: username_element.value.trim(),
        client_id: client_id_element.value.trim(),
        website: website_element.value.trim(),
        has_agreed_terms: all_checked,
    }

    // const ch_msgg = all_checked ? "ALL CHECKED" : "NOT ALL CHECKED"
    // console.log(ch_msgg)

    // Check the inputs
    let all_fields_legit = utils.check_username(creds.username, err_msgs)
    all_fields_legit = utils.check_password(creds.password, err_msgs) && all_fields_legit
    all_fields_legit = utils.check_email(creds.email, err_msgs) && all_fields_legit
    all_fields_legit = utils.check_consent(creds.has_agreed_terms, err_msgs) && all_fields_legit

    // if any checks failed, show the error and return
    if (!all_fields_legit) {
        console.log(err_msgs.length);
        show_err_box()
        return
    } else { hide_err_box() }

    // now send it to the register route
    const route = "/auth/register"

    await utils.fetch_json_post(route, creds)
        .then(response => {
            if (!response.ok) {
                response.json().then(data => {

                    if (!!data.code) {
                        if (data.code == 422){
                            // If inputs were unacceptable, backend informs us, we show the message.
                            !data.username_valid && err_msgs.push(utils.username_reqs_msg)
                            !data.email_valid && err_msgs.push(utils.email_reqs_msg)
                            !data.password_valid && err_msgs.push(utils.password_reqs_msg)
                        } else if (data.code == 409){
                            // If inputs were duplicates, backend informs us, we show the message.
                            !data.username_valid && err_msgs.push("Username already taken.")
                            !data.email_valid && err_msgs.push("Email already taken.")
                        } else {
                            let msg = (!!data.code) ? (data.code.toString() + " ") : ""
                            msg += (!!data.error) ? data.error : " Error occurred"
                            err_msgs.push(msg)
                        }
                    } else {
                        err_msgs.push("Error.")
                    }

                    show_err_box()
                })

                throw new Error("Inputs invalid or server error.")
            }
            return response.json()
        }).then(data => {
            // THIS WILL BE AUTH DATA NOT USER (change "user" to "auth_data")
            console.log("Incoming data: ", data)
            // do something with the user
            if (!!data.user_id || !!data.username){
                window.location.href = "/dashboard";
            } else if (!!data.redirect_uri) {
                window.location.href = data.redirect_uri
            }
        }).catch(error => {
            console.log('Error: ', error)
        })
}

// SHOW/HIDE ERROR BOX

const hide_err_box = () =>
    document.getElementById("err_msg_box").classList.add('hidden')

const show_err_box = () => {
    const err_box = document.getElementById("err_msg_box")
    err_box.innerHTML = "";

    for (let err_msg of err_msgs) {
        const msg_p = "<p>" + err_msg + "</p>"
        err_box.innerHTML += msg_p
    }

    err_box.classList.remove('hidden')
}


document.addEventListener('DOMContentLoaded', () => hide_err_box())
document.getElementById('username').addEventListener('keydown', (e) => (e.key === 'Enter') && submit_register())
document.getElementById('email').addEventListener('keydown', (e) => (e.key === 'Enter') && submit_register())
document.getElementById('password').addEventListener('keydown', (e) => (e.key === 'Enter') && submit_register())

window.submit_register = submit_register
