$(document).foundation()
import * as utils from './utils.js'
//import * as globals from './globals.js'


/**
 * Functions for the login input page
 **/

let err_msgs = []

const redirect_to_client = async client_id => {
    const route = "/client_link/" + client_id

    await fetch(route, {
        method: 'POST',
        credentials: 'include'
    }).then(response => {
        if(!response.ok) {
            response.json().then(data => {
                window.location.href = "/error";
            })

            throw new Error("Unable to logout.")
        }
        return response.json()
    }).then(data => {
        console.log("Logout data: ", data)
        if(!!data.redirect_uri){
            window.location.href = data.redirect_uri
        } else if (!!data.error) {
            err_msgs.push(data.error)
            show_err_box()
        }
        
    }).catch(error => {
        console.log('Error: ', error)
        window.location.href = "/error";
    })
}


document.addEventListener("click", function (event) {
    const link_element = event.target.closest(".client-link")
    if (!link_element) return

    event.preventDefault()
    redirect_to_client(link_element.dataset.clientId)
});


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
    err_msgs = []
}


// Add event listeners
document.addEventListener('DOMContentLoaded', () => hide_err_box())