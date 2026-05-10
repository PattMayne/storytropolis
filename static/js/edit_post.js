$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


/**
 * Functions for the login input page
 **/

let msgs = []

const submit_data = async () => {
    msgs = []
    hide_msg_box()

    // Gather data 
    const post_body = document.getElementById("post_body").value.trim()
    const post_title = document.getElementById("title_input").value.trim()
    const categories = document.getElementById("categories").value.trim()
    const post_id = document.getElementById("post_id").value
    const pin_checked = document.getElementById("pin_check").checked
    const pinned_to_blog_checked = document.getElementById("pinned_to_blog_check").checked

    if (pin_checked) { console.log("CHECKED") } else { console.log("NOT CHECKED") }
    if (pinned_to_blog_checked) { console.log("PINNED TO BLOG CHECKED") } else { console.log("PINNED TO BLOG NOT CHECKED") }

    // make sure required fields are not empty
    let required_fields_are_filled = post_body != ""

    if (!required_fields_are_filled) {
        console.log("Error with input data")
        msgs.push("Please write an actual post")
        show_msg_box()
        return
    }

    const data = {
        post_id: parseInt(post_id, 10),
        post_title: post_title,
        categories: categories,
        post_body: post_body,
        pinned: pin_checked,
        pinned_to_blog: pinned_to_blog_checked
    }

    /*  KEEPING THE FETCH STUFF IN COMMENTS FOR LATER ADAPTATION */

    // now send it to the update route
    const route = "/admin/update_post"

    await utils.fetch_json_post(route, data)
        .then(response => {
            if(!response.ok) {
                console.log("NOT OK")
                response.json().then(data => {
                    if (!!data.code && data.code == 403 || data.code == 401) {
                        const redirect_uri = "/error/" + data.code;
                        window.location.href = redirect_uri;
                    } else {
                        let msg = (!!data.code) ? (data.code.toString() + " ") : ""
                        msg += (!!data.error) ? data.error : " Error occurred"
                        msgs.push(msg)
                        show_msg_box()
                    }
                })

                throw new Error("Could not update blog post, or server error.")
            }
            return response.json()
        }).then(response => {
            if(!!response.message){
                msgs.push(response.message)
                show_msg_box()
            }
            
        }).catch(error => {
            console.log('Error: ', error)
        })
}


const delete_post = async () => {
    msgs = []
    hide_msg_box()

    // Gather data 
    const post_id = document.getElementById("post_id").value
    const data = { post_id: parseInt(post_id, 10) }

    /*  KEEPING THE FETCH STUFF IN COMMENTS FOR LATER ADAPTATION */

    // now send it to the delete route
    const route = "/admin/delete_post"

    await utils.fetch_json_post(route, data)
        .then(response => {
            if(!response.ok) {
                console.log("NOT OK")
                response.json().then(data => {
                    if (!!data.code && data.code == 403 || data.code == 401) {
                        const redirect_uri = "/error/" + data.code
                        window.location.href = redirect_uri
                    } else {
                        let msg = (!!data.code) ? (data.code.toString() + " ") : ""
                        msg += (!!data.error) ? data.error : " Error occurred"
                        msgs.push(msg)
                        show_msg_box()
                    }
                })

                throw new Error("Could not update blog post, or server error.")
            }
            return response.json()
        }).then(response => {
            if(!!response.success) {
                window.location.href = "/admin"
            } else if (!!response.message) {
                msgs.push(response.message)
                show_msg_box()
            }
            
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
document.addEventListener('DOMContentLoaded', () => {
    hide_msg_box()
    const submit_button = document.getElementById("submit_button")
    const really_delete_button = document.getElementById("really_delete_button")
    const ask_delete_button = document.getElementById("ask_delete_button")
    const cancel_button = document.getElementById("cancel_button")
    const really_delete_panel = document.getElementById("really_delete_panel")

    !!submit_button &&
        submit_button.addEventListener("click", () => {
            console.log("submitting")
            submit_data()
        })

    !!really_delete_button &&
        really_delete_button.addEventListener("click", () => {
            console.log("deleting")
            delete_post()
        })

    !!ask_delete_button &&
        ask_delete_button.addEventListener("click", () => show_hide_delete())
    
    !!cancel_button &&
        cancel_button.addEventListener("click", () => show_hide_delete())

    const show_hide_delete = () => {
        if (!really_delete_panel.classList.contains("hidden")) {
            really_delete_panel.classList.add("hidden")
            ask_delete_button.classList.remove("hidden")
        } else {
            really_delete_panel.classList.remove("hidden")
            ask_delete_button.classList.add("hidden")
        }
    }
})