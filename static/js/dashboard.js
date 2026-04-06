$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


let msgs = []

/**
 * Save the user's new password to the database.
 * Checks that user "confirmed" the password (by typing it twice),
 * and that it meets regex requirements,
 * and either prints error messages or sends the password to the backend.
 * If the backend fails or rejects it for any reason,
 * display those errors.
 */
const save_password = async () => {
    msgs = []

    // get the elements where the names are stored
    const password_element = document.getElementById("new_password")
    const confirmed_password_element = document.getElementById("new_password_confirm")

    // get values from input elements
    const password_obj = { password: password_element.value.trim() }
    const confirmed_password = confirmed_password_element.value.trim()

    // check that passwords match
    if (!(password_obj.password === confirmed_password)) {
        msgs.push("Passwords do not match")
        show_msg_box()
        return
    } else {
        hide_msg_box() }   

    // Passwords match. Now validate the password format
    if (!utils.check_password(password_obj.password, msgs)) {
        console.log(msgs.length);
        show_msg_box()
        return
    } else { hide_msg_box() }   

    // Password is GOOD. Send it
    const route = "/auth/update_password"

    await utils.fetch_json_post(route,password_obj)
        .then(response => {
            if (!response.ok) {
                response.json().then(data => {

                    if (!!data.code) {
                        if (data.code == 422){
                            // If inputs were unacceptable, backend informs us, we show the message.
                            !data.names_valid && msgs.push(utils.password_reqs_msg)
                        } else if (data.code == 401){
                            // User is not authenticated
                            globals.logout()
                        } else {
                            let msg = (!!data.code) ? (data.code.toString() + " ") : ""
                            msg += (!!data.error) ? data.error : " Error occurred"
                            msgs.push(msg)
                        }
                    } else { msgs.push("Error.") }
                    show_msg_box()
                })

                throw new Error("Inputs invalid or server error.")
            }
            return response.json()
        }).then(update_data => {
            if (!!update_data.success) {
                msgs.push("Password updated.")
            } else {
                msgs.push("Update failed.")
            }
            show_msg_box()
        }).catch(error => {
            console.log('Error: ', error)
        })
}

/**
 * Save the user's first and last name to the database.
 * Gets the name inputs from the fields, validates the values,
 * and either prints error messages or sends the names to the backend.
 * If the backend fails or rejects them for any reason,
 * display those errors.
 */
const save_names = async () => {
    msgs = []

    // get the elements where the names are stored
    const first_name_element = document.getElementById("first_name")
    const last_name_element = document.getElementById("last_name")

    // get values from input elements
    const names = {
        first_name: first_name_element.value.trim(),
        last_name: last_name_element.value.trim()
    }

    // check inputs

    let both_names_legit = utils.check_real_name(names.first_name, msgs)
    both_names_legit = utils.check_real_name(names.last_name, msgs)

    // if any checks failed, show the error and return
    if (!both_names_legit) {
        console.log(msgs.length);
        show_msg_box()
        return
    } else { hide_msg_box() }   

    // checks passed. send names to update_names route
    const route = "/auth/update_names"

    await utils.fetch_json_post(route, names)
        .then(response => {
            if (!response.ok) {
                response.json().then(data => {

                    if (!!data.code) {
                        if (data.code == 422){
                            // If inputs were unacceptable, backend informs us, we show the message.
                            !data.names_valid && msgs.push(utils.username_reqs_msg)
                        } else if (data.code == 401){
                            // User is not authenticated
                            globals.logout()
                        } else {
                            let msg = (!!data.code) ? (data.code.toString() + " ") : ""
                            msg += (!!data.error) ? data.error : " Error occurred"
                            msgs.push(msg)
                        }
                    } else { msgs.push("Error.") }

                    show_msg_box()
                })

                throw new Error("Inputs invalid or server error.")
            }
            return response.json()
        }).then(update_data => {
            if (!!update_data.success) {
                msgs.push("Names updated.")
            } else {
                msgs.push("Update failed.")
            }
            show_msg_box()
        }).catch(error => {
            console.log('Error: ', error)
        })
}

// SHOW/HIDE ERROR BOX

const hide_msg_box = () =>
    document.getElementById("msg_box").classList.add('hidden')

const show_msg_box = () => {
    const msg_box = document.getElementById("msg_box")
    msg_box.innerHTML = "";

    for (let msg of msgs) {
        const msg_p = "<p>" + msg + "</p>"
        msg_box.innerHTML += msg_p
    }

    msg_box.classList.remove('hidden')
}


// Add event listeners

document.addEventListener('DOMContentLoaded', () => hide_msg_box())
document.getElementById('first_name').addEventListener(
    'keydown', (e) => (e.key === 'Enter') && save_names())

document.getElementById('last_name').addEventListener(
    'keydown', (e) => (e.key === 'Enter') && save_names())

document.getElementById('new_password').addEventListener(
    'keydown', (e) => (e.key === 'Enter') && save_password())

document.getElementById('new_password_confirm').addEventListener(
    'keydown', (e) => (e.key === 'Enter') && save_password())


window.save_names = save_names
window.save_password = save_password