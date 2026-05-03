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
    const title = document.getElementById("title").value.trim()
    const author = document.getElementById("author").value.trim()
    const genres = document.getElementById("genres").value.trim()
    const publisher = document.getElementById("publisher").value.trim()
    const release_year = document.getElementById("release_year").value.trim()
    const price = document.getElementById("price").value.trim()
    const slug = document.getElementById("slug").value.trim()
    const description = document.getElementById("description").value.trim()

    const img_input = document.getElementById("cover_img_upload")

    // make sure required fields are not empty
    let required_fields_are_filled =
        !!title && !!author && !!genres && !!publisher &&
        !!release_year && !!price && !!slug && !!description &&
        !!img_input.files && img_input.files.length > 0

    if (!required_fields_are_filled) {
        console.log("Error with input data")
        msgs.push("Please fill all fields.")
        show_msg_box()
        return
    }

    // NOT doing JSON. Re-make this as form data
    const data = {
        title: title,
        author: author,
        genres: genres,
        publisher: publisher,
        release_year: release_year,
        price: price,
        slug: slug,
        description: description,
        cover_img: img_input.files[0]
    }

    const form_data = new FormData()
    form_data.append("title", title)
    form_data.append("author", author)
    form_data.append("genres", genres)
    form_data.append("publisher", publisher)
    form_data.append("release_year", release_year)
    form_data.append("price", price)
    form_data.append("slug", slug)
    form_data.append("description", description)
    form_data.append("cover_img", img_input.files[0])

    /*  KEEPING THE FETCH STUFF IN COMMENTS FOR LATER ADAPTATION */

    // now send it to the login route
    const route = "/admin/add_book"

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
