$(document).foundation()
import { logout } from './globals.js'


export function toggle_nav() {
    const nav = document.getElementById("menu_to_toggle")
    if (nav.classList.contains("hidden")) {
        nav.classList.remove("hidden")
    } else {
        nav.classList.add("hidden")
    }
}

document.getElementById('toggle_nav_button').addEventListener('click', ()=> {
    toggle_nav()
})

window.toggle_nav = toggle_nav