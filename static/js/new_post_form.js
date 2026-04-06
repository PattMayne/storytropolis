$(document).foundation()
import * as utils from './utils.js'
import * as globals from './globals.js'


/**
 * Functions for the login input page
 **/

let msgs = []


const submit_data = async () => {
    msgs = []

    console.log("SUBMITTING DATA")
    hide_msg_box()

    // Gather data 
    const post_body = document.getElementById("post_body").value.trim()
    const post_title = document.getElementById("title").value.trim()
    const pin_checked = document.getElementById("pin_check").checked

    // make sure required fields are not empty
    let required_fields_are_filled = post_body != ""

    if (!required_fields_are_filled) {
        console.log("Error with input data")
        msgs.push("Please write an actual post")
        show_msg_box()
        return
    }

    const data = {
        post_title: post_title,
        post_body: post_body,
        pinned: pin_checked
    }

    console.log("Post body is: " + data.post_body)

    /*  KEEPING THE FETCH STUFF IN COMMENTS FOR LATER ADAPTATION */

    // now send it to the login route
    const route = "/admin/add_post"

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

                throw new Error("Could not add blog post, or server error.")
            }
            return response.json()
        }).then(response => {
            if (!!response.message){
                msgs.push(response.message)
                show_msg_box()
            }

            if (!!response.success && !! response.post_id) {
                const redirect_url = "/admin/edit_post/" + response.post_id

                setTimeout(() => {
                    window.location.href = redirect_url
                }, 1400)
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
    !!submit_button && submit_button.addEventListener("click", () => {
        console.log("pressed le buton")
        submit_data()
    })
})
