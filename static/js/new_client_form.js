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
    const site_domain = document.getElementById("site_domain").value.trim()
    const site_name = document.getElementById("site_name").value.trim()
    const client_id = document.getElementById("client_id").value.trim()
    const redirect_uri = document.getElementById("redirect_uri").value.trim()
    const description = document.getElementById("description").value.trim()
    const logo_url = document.getElementById("logo_url").value.trim()
    const client_type = document.getElementById("client_type").value.trim()
    const category = document.getElementById("category").value.trim()
    const is_active = document.getElementById("is_active").checked

    // make sure required fields are not empty
    let required_fields_are_filled =
        site_domain != "" &&
        site_name != "" &&
        client_id != "" &&
        redirect_uri != "" &&
        client_type != ""

    if (!required_fields_are_filled) {
        console.log("Error with input data")
        msgs.push(utils.new_client_req_fields_msg)
        show_msg_box()
        return
    }

    const client_data = {
        site_domain: site_domain,
        site_name: site_name,
        client_id: client_id,
        redirect_uri: redirect_uri,
        logo_url: logo_url,
        description: description,
        category: category,
        client_type: client_type,
        is_active: is_active
    }

    /*  KEEPING THE FETCH STUFF IN COMMENTS FOR LATER ADAPTATION */

    // now send it to the login route
    const route = "/admin/add_client"

    await utils.fetch_json_post(route, client_data)
        .then(response => {
            if(!response.ok) {
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

                throw new Error("Could not add client site, or server error.")
            }
            return response.json()
        }).then(secret_data => {
            console.log("secret_data: ", secret_data.raw_client_secret)
            if(!!secret_data.raw_client_secret){
                const secret_message = "Here is the CLIENT_SECRET for the new domain. " +
                    "We will never show this again, so COPY IT NOW and put it in " +
                    "the environment variables of the new client site."
                msgs.push(secret_message)
                msgs.push(secret_data.raw_client_secret)
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
document.addEventListener('DOMContentLoaded', () => hide_msg_box())

// Make functions available to the HTML elements (via window)
window.submit_data = submit_data