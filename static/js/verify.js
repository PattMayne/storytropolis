$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


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

async function submit_verify() {
    console.log("SUBMITTING VERIFICATION DATA")
    const route = "/verify_post"

    const email_el = document.getElementById("email")
    const code_el = document.getElementById("v_code")
    const email = !!email_el ? email_el.value.trim() : ""
    const code = !!code_el ? code_el.value.trim() : ""

    if (!email || !code) {
        msgs.push("Please enter email and verification code")
        show_msg_box()
        return
    }

    let code_obj = {
        "email": email.toString(),
        "code": code.toString()
    }

    await utils.fetch_json_post(route,code_obj)
        .then(response => {
            if (!response.ok) {
                response.json().then(data => {
                    const msg = !!data.error && !!data.code ?
                        "<h3>" + data.code + "</h3><p>" + data.error + "</p>" :
                        "Error Occurred"
                    msgs.push(msg)
                    show_msg_box()
                })

                throw new Error("Inputs invalid or server error.")
            }
            return response.json()
        }).then(data => {
        if (!!data.username){
            window.location.href = "/dashboard";
        } else if (!!data.redirect_uri) {
            window.location.href = data.redirect_uri
        }
        }).catch(error => {
            console.log('Error: ', error)
        })
}

async function request_new_code() {
    console.log("REQUESTING NEW CODE")
    const route = "/req_new_code"
    let email = document.getElementById("email_req").value
    let inputs = {
        "email": email.toString()
    }

    await utils.fetch_json_post(route, inputs)
    .then(response => {
        if (!response.ok) {
            response.json().then(data => {
                const msg = !!data.error && !!data.code ?
                    "<h3>" + data.code + "</h3><p>" + data.error + "</p>" :
                    "Error Occurred"
                msgs.push(msg)
                show_msg_box()
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

document.addEventListener('DOMContentLoaded', async () => {
    let new_code_req_btn = document.getElementById("new_code_req_btn")
    let verify_btn = document.getElementById("submit_verify")

    new_code_req_btn.addEventListener('click', () => request_new_code())
    !!verify_btn && verify_btn.addEventListener('click', () => submit_verify())

    document.getElementById('email').addEventListener('keydown', (e) => (e.key === 'Enter') && submit_verify())
    document.getElementById('v_code').addEventListener('keydown', (e) => (e.key === 'Enter') && submit_verify())
})