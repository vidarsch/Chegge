document.addEventListener('DOMContentLoaded', function() {
    const nameInput = document.querySelector('.name-input');
    const ws = new WebSocket('ws://92.113.145.13:8080');
    document.querySelector('.name-input').value = getCookie() || '';
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
    
    function sendMessage(content, name, isImage = false) {
        if (ws.readyState === WebSocket.OPEN && content.trim().length > 0) {
            const payload = {
                type: isImage ? "message-image" : "message",
                name: name.trim() || "Anonymous"
            };

            if (isImage) {
                payload.image = content;
            } else {
                payload.message = content.trim();
                messageInput.value = ''; // Clear input after sending
            }

            ws.send(JSON.stringify(payload));
            console.log(`Sent ${isImage ? "image" : "message"}:`, payload);
        } else {
            appendMessage("Laggbuggg", "Nej tack");
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
    });
    reader.onload = function(e) {
        const base64Image = e.target.result.split(',')[1]; 
        sendMessage(base64Image, nameInput.value, true);
        imageUpload.value = '';
    }

});


