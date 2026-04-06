$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


/**
 * Functions for the login input page
 **/

let err_msgs = []

const submit_login = async () => {
    err_msgs = []
    const pass_element = document.getElementById("password")
    const username_or_email_element = document.getElementById("username_or_email")
    const client_id_element = document.getElementById("client_id")

    const creds = {
        password: pass_element.value.trim(),
        username_or_email: username_or_email_element.value.trim(),
        client_id: client_id_element.value.trim()
    }

    // Check the inputs (identifier must match email OR username specifications)
    // The backend will figure out which thing we did
    let all_fields_legit =
        utils.check_username(creds.username_or_email, err_msgs) ||
        utils.check_email(creds.username_or_email, err_msgs)
    all_fields_legit = utils.check_password(creds.password, err_msgs) && all_fields_legit

    if (!all_fields_legit) {
        show_err_box()
        return
    } else {
        err_msgs = []
        hide_err_box()
    }    

    // now send it to the login route
    const route = "/auth/login"

    await utils.fetch_json_post(route, creds)
        .then(response => {
            if (!response.ok) {
                response.json().then(data => {
                    let msg = (!!data.code) ? (data.code.toString() + " ") : ""
                    msg += (!!data.error) ? data.error : " Error occurred"
                    err_msgs.push(msg)
                    show_err_box()
                })

                throw new Error("User not found or server error.")
            }
            return response.json()
        }).then(data => {
            console.log("data: ", data)
            if (!!data.username){
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


// Add event listeners

document.addEventListener('DOMContentLoaded', () => hide_err_box())
document.getElementById('username_or_email').addEventListener(
    'keydown', (e) => (e.key === 'Enter') && submit_login())
document.getElementById('password').addEventListener(
    'keydown', (e) => (e.key === 'Enter') && submit_login())


// Make functions available to the HTML elements (via window)

window.submit_login = submit_login
