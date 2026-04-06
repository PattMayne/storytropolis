
/* Log the user out and redirect back to the home page. */
export const logout = async () => {

    // send it to the login route
    const route = "/auth/logout"

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
    }).then(logout_data => {
        console.log("Logout data: ", logout_data)
        if(!!logout_data.logout){
            window.location.href = "/";
        }
        
    }).catch(error => {
        console.log('Error: ', error)
        window.location.href = "/error";
    })
}


// Everything we might want to run on page load, for any (or most) page(s).
document.addEventListener('DOMContentLoaded', () => {
  const button = document.getElementById('logout_nav_button')
  // Checking for the button first in case use is logged in (and button doesn't exist)
  if (!!button) button.addEventListener('click', logout)
})


window.logout = logout