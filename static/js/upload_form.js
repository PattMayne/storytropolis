$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


/**
 * Functions for the new book upload page
 * Must use FORM DATA instead of JSON.
 **/

let msgs = []



const submit_data = async () => {
    msgs = []

    console.log("SUBMITTING DATA")
    hide_msg_box()

    // Gather data 
    const filename = document.getElementById("filename").value.trim()
    const img_upload = document.getElementById("img_upload")

    // make sure required fields are not empty
    let required_fields_are_filled =
        !!filename &&
        !!img_upload.files && img_upload.files.length > 0

    if (!required_fields_are_filled) {
        console.log("Error with input data")
        msgs.push("Please fill all fields.")
        show_msg_box()
        return
    }

    const form_data = new FormData()
    form_data.append("filename", filename)
    form_data.append("img_upload", img_upload.files[0])

    /*  KEEPING THE FETCH STUFF IN COMMENTS FOR LATER ADAPTATION */

    // now send it to the login route
    //const route = "/admin/add_book"
    const route = "/admin/img_upload_post"

    fetch(route, {
        method: 'POST',
        body: form_data
    }).then(response => response.json()) // or response.text(), etc.
    .then(data => {
        // Handle the response data here
        console.log(data)
        // redirect to the new book's page if successful, otherwise show error message
    })
    .catch(error => {
        // Handle errors here
        console.error('Error:', error)
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
document.addEventListener('DOMContentLoaded', () => {
    hide_msg_box()
    const submit_button = document.getElementById("submit_button")
    !!submit_button && submit_button.addEventListener("click", () => {
        console.log("pressed le buton")
        submit_data()
    })
})
