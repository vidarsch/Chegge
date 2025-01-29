document.addEventListener('DOMContentLoaded', function() {
    const nameInput = document.querySelector('.name-input');
    const ws = new WebSocket('ws://92.113.145.13:8080');
    document.querySelector('.name-input').value = getCookie() || '';
    const reader = new FileReader();
    const imageScaler = 1200; // image max width and/or height
    if (Math.floor(Math.random()*7) == 2) {
        window.location.href = "hahahaha.html";
    }

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

    // Bildhantering

    var picSize = 0;
    imageUpload.addEventListener('change', function(e) {
        reader.readAsDataURL(e.target.files[0]);
        //console.log(e.target.files[0]);
        //picSize = (Math.round(e.target.files[0].size/1024)/1000).toFixed(2);
    });
    
    reader.onload = function(e) {
        //const base64Image = e.target.result.split(',')[1]; // orginal storlek
        imageToDataUri(e.target.result,function(dataUri) { 
            sendMessage(dataUri.split(',')[1], nameInput.value, true);
        });

        imageUpload.value = '';
    }
    
    // maybe need image scaling / resizing? haha ja

    async function imageToDataUri(imgage, callback) {
        var img = new Image();
        img.src = imgage;
        var canvas = document.createElement('canvas');
        img.onload = function() {
            var ctx = canvas.getContext('2d');
            let scale = 1;
            if (img.width > imageScaler || img.height > imageScaler) {
                scale = Math.min(imageScaler / img.width, imageScaler / img.height);
            }
            canvas.width = img.width * scale;
            canvas.height = img.height * scale;
            ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
            callback(canvas.toDataURL());
           
        };
    }

    // IOS / Android websocket hantering


});


