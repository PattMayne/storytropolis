$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


/**
 * Functions for the request verification page
 **/


let msgs = []

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
    msgs = []
}


async function request_new_code() {
    console.log("REQUESTING NEW CODE")
    const route = "/req_new_code"
    const email = document.getElementById("email_req").value.toString().trim()

    console.log("email: " + email)

    const inputs = {
        "email": email
    }

    await utils.fetch_json_post(route, inputs)
    .then(response => {
        if (!response.ok) {
            response.json().then(data => {

                if (!!data.code && (data.code == 404 || data.code.toString().trim() == "404")) {
                    const msg = "<h3>Email Address Not Found</h3>" +
                        "<p>That address is not registered on our site. Try a different address, or " +
                        "<a href='/auth/register'>Register.</a></p>"
                    msgs.push(msg)
                    show_msg_box()
                } else {
                    const msg = !!data.error && !!data.code ?
                        "<h3>" + data.code + "</h3><p>" + data.error + "</p>" :
                        "Error Occurred"
                    msgs.push(msg)
                    show_msg_box()
                }
            })

            throw new Error("Inputs invalid or server error.")
        }
        return response.json()
    }).then(update_data => {
        if (!!update_data.message) {
            msgs.push(update_data.message)
            show_msg_box()
        }        
    }).catch(error => {
        console.log('Error: ', error)
    })
}



// Add event listeners

document.addEventListener('DOMContentLoaded', () => hide_msg_box())

document.getElementById('email_req').addEventListener(
    'keydown', e => {
        console.log(e, typeof e)
        if (e.key === 'Enter') {
            //e.preventDefault()
            request_new_code()
        }
    })

document.getElementById('submit_btn').addEventListener(
    'click', () => request_new_code())

