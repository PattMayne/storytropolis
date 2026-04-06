/* REGEX for user inputs */

export const username_regex = /^[A-Za-z0-9_-]+$/
export const password_regex = /^[A-Za-z0-9!@#$%^&*()_\-+=\[\]{}:;<>.,?~`|]+$/
export const email_regex = /^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$/

/* LENGTH RANGES for inputs */

export const username_length_range = {
    min: 6,
    max: 20
}

export const password_length_range = {
    min: 6,
    max: 16
}

export const first_last_name_length_range = {
    min: 1,
    max: 50
}

/* Error messages for failed input valiadtion. */
export const email_reqs_msg = "Must be a legitimate email address. Check your formatting."
export const username_reqs_msg = "Username must be 6 to 20 characters. " +
    "Only letters, numbers, underscore, and hyphen allowed."
export const password_reqs_msg = "Password must be 6 to 16 characters with no spaces."
export const name_range_err_msg = "Names must be 2 to 50 characters in length"

export const new_client_req_fields_msg = "Site domain, name, auth_id, redirect_uri, " +
    "and type must not be empty. Also, redirect_uri and domain must be valid."

export const agreements_msg = "You must agree to the terms and conditions, privacy policy, " +
    "and to receive essential emails for account management."

// Make sure password matches regex and length requirements
export const check_password = (password, err_msgs) => {
    const password_is_legit = password_regex.test(password) &&
        string_in_range(password_length_range, password)

    if (!password_is_legit) { err_msgs.push(password_reqs_msg) }
    return password_is_legit
}


// Make sure email matches regex
export const check_email = (email, err_msgs) => {
    const email_is_legit = email_regex.test(email)

    if (!email_is_legit) { err_msgs.push(email_reqs_msg) }
    return email_is_legit
}


// Make sure user has agreed to all conditions
export const check_consent = (all_agreed, err_msgs) => {
    if (!all_agreed) { err_msgs.push(agreements_msg) }
    return all_agreed
}

// Make sure username matches regex and length requirements
export const check_username = (username, err_msgs) => {
    const username_is_legit = username_regex.test(username) &&
        string_in_range(username_length_range, username)

    if (!username_is_legit) { err_msgs.push(username_reqs_msg) }
    return username_is_legit
}

/**
 * Validate either first or last name.
 * Same rules apply to both.
 * No regex, just length.
 * @param {string} name 
 * @param {array} msgs 
 * @returns boolean
 */
export const check_real_name = (name, msgs) => {   
    let name_in_range =  string_in_range(first_last_name_length_range, name)
    if (!name_in_range) {
        msgs.push(name_range_err_msg)
    }
    return name_in_range    
}

// Make sure the input string is within a given length range
const string_in_range = (range_obj, string) =>
    string.length >= range_obj.min && string.length <= range_obj.max


/**
 * For any time we are using fetch to send a JSON object to a POST API.
 * @param {String} route 
 * @param {JSON object} json_obj 
 * @returns HTTP response from a fetch call
 */
export const fetch_json_post = async (route, json_obj) => {
    // First create a JSON string, doing checks to ensure the obj is legit.
    const json_string =
        (typeof json_obj === "object" && json_obj !== null)
            ? (() => {
                try {
                    return JSON.stringify(json_obj)
                } catch {
                    return json_simple_error_string()
                }
            })() // the () immediately invokes the function I just defined
        : (typeof json_obj === "string" && is_valid_json_string(json_obj))
            ? json_obj
            : json_simple_error_string()

    // now we return the HTTP response from a fetch call
    return fetch(route, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json; charset=utf-8' },
        body: json_string
    })
}


/* make sure that a string is a legit JSON string which can be parsed. */ 
const is_valid_json_string = (json_string) => {
    try {
        return JSON.parse(json_string)
    } catch {
        return false
    }
}

// In case we have an error parsing the JSON, notify of error
const json_simple_error_string = () => JSON.stringify({ "error": "JSON response error" })
