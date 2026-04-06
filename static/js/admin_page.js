$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


let msgs = []


// SHOW/HIDE MESSAGE BOX

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