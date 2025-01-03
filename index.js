document.addEventListener('DOMContentLoaded', function() {
    const nameInput = document.querySelector('.name-input');
    const ws = new WebSocket('ws://92.113.145.13:8080');
    document.querySelector('.name-input').value = getCookie();
    const reader = new FileReader();

    function sendName(name) {
        //document.cookie = "username=hej"; // + name;
        /* if (ws.readyState === WebSocket.OPEN && name.length > 0) {
            ws.send(JSON.stringify({
                type: "update_name",
                name: name
            }));
            //nameInput.value = '';
            console.log('Sent name:', name);
        } */
    }

    nameInput.addEventListener('focusout', function(e) {
        //let mess = messageInput = document.querySelector('.message-input').value;
        if (this.value.length > 0) {
            console.log('name saved in cookie');
            setCookie(this.value);
        }
    });

    const messageInput = document.querySelector('.message-input');
    const imageUpload = document.querySelector('#image-upload');

    function sendMessage(message, name, image = false) {
        if (ws.readyState === WebSocket.OPEN && message.length > 0) {
            if (image) {
                name = name.length > 0 ? name : "Anonymous";
                ws.send(JSON.stringify({
                    type: "message-image",
                    message: message, // base64 kanske korkat?
                    name: name
                }));
            } else {
            name = name.length > 0 ? name : "Anonymous";
            ws.send(JSON.stringify({
                type: "message",
                message: message,
                name: name
            }));
            messageInput.value = '';
            }
            console.log('Sent message:', message);

        } else {
            appendMessage("Laggbugg", "Något gick fel");
        }
    }

    messageInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter' && this.value.length > 0) {
            console.log('Enter pressed');
            sendMessage(this.value,document.querySelector('.name-input').value);
        }
    });
    messageInput.addEventListener('focusout', function(e) {
        if (this.value.length > 0) {
            sendMessage(this.value,document.querySelector('.name-input').value);
        }

    });

    imageUpload.addEventListener('change', function(e) {
        reader.readAsDataURL(e.target.files[0]);
        reader.onload = function(e) {
            console.log(e.target.result);
            sendMessage(e.target.result,document.querySelector('.name-input').value,true);
        }
    });

});


