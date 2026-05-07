$(document).foundation()
import { logout } from './globals.js'


export const toggle_nav = () => {
    const nav = document.getElementById("menu_to_toggle")
    if (!nav) return

    if (nav.classList.contains("hidden")) {
        nav.classList.remove("hidden")
    } else {
        nav.classList.add("hidden")
    }
}

const nav_toggle_btn = document.getElementById('toggle_nav_button')

!!nav_toggle_btn && nav_toggle_btn.addEventListener('click', ()=> {
    toggle_nav()
})

window.toggle_nav = toggle_nav