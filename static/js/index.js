$(document).foundation()
import * as utils from './utils.js'
//import * as globals from './globals.js'


/**
 * Functions for the login input page
 **/

let err_msgs = []



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