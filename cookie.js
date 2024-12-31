function setCookie(name) {
    console.log(name);
    document.cookie = "username=" + encodeURIComponent(name) + "; path=/; max-age=2592000";
}

function getCookie() {
    const cookies = document.cookie.split(';');
    const [name, value] = cookies[0].trim().split('=');
    if (name === 'username') {
        return decodeURIComponent(value);
    }
    return "Dålig golfare";
}