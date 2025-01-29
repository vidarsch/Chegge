document.addEventListener('DOMContentLoaded', function() {
    // Connect to Rust WebSocket server (assuming it runs on localhost:8080)
    const ws = new WebSocket('ws://92.113.145.13:8080');
    const messagesDiv = document.getElementById('messages');
    const uid = 0;

    ws.onopen = function() {
        console.log('Connected to WebSocket server');
        appendMessage('Connected to server', "halal");
        fetchMessages(ws);
        //getTerrain(document.getElementById('bottomDiv').offsetHeight, document.getElementById('bottomDiv').offsetWidth);
    };

    ws.onmessage = function(event) {
        console.log(event);
        try {
            const data = JSON.parse(event.data);
            console.log(data);
            if (data.length === undefined) {
                const { type, name, message, image } = data;
                appendMessage(name, message, image);
                
            } else {
                let datarev = data.reverse();
                for (const element of datarev) {
                    //console.log(element);
                    appendMessage(element.name, element.image, element.message);
                }
            }
           
        } catch (error) {
            console.error('Failed to parse incoming message:', error);
        }
    };

    ws.onclose = function() {
        console.log('Disconnected from WebSocket server');
        appendMessage('Disconnected from server');
        setTimeout(retryConnection, 30000);
    };

    ws.onerror = function(error) {
        console.error('WebSocket error:', error);
        appendMessage('Error: ' + error.message);
    };
    
    const tickrate = 500;
    /* setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send('Hello from client!');
            appendMessage('Sent: Hello from client!');
        }
    }, tickrate); */
    function getTerrain(height,width) {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({type: "get_terrain", height: height, width: width}));
            appendMessage('Generating terrain...');
            fetchMessages();
            return true;
        } else {
            return false;
        }
    }
});

function retryConnection() {
    ws = new WebSocket('ws://92.113.145.13:8080');
    console.log("retryConnection");
}

function appendMessage(user, message = null, image = null) {
    const messagesDiv = document.getElementById('messages');
    const messageElement = document.createElement('div');
    messageElement.className = 'message';

    const safeUser = escapeHtml(user);

    if (message) {
        const safeMessage = escapeHtml(message);
        messageElement.innerHTML = `<span class="user">${safeUser}:</span> <span class="text">${safeMessage}</span>`;
    } else if (image) {
        const imgSrc = `data:image/png;base64,${image}`;
        messageElement.innerHTML = `<span class="user">${safeUser}:</span> <img src="${imgSrc}" alt="Image" class="image">`;
    }

    messagesDiv.appendChild(messageElement);
    messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

function fetchMessages(ws) {
    console.log("fetchMessages");
    console.log("WebSocket state:", ws.readyState); 
    console.log("WebSocket.OPEN value:", WebSocket.OPEN);
    
    if (ws.readyState === WebSocket.OPEN) {
        try {
            const payload = { type: "fetch_messages" };
            console.log("Sending payload:", payload);
            ws.send(JSON.stringify(payload));
        } catch (error) {
            console.error("Error sending fetch_messages:", error);
        }
    } 
}

function escapeHtml(unsafe)
{
    return unsafe
         .replace(/&/g, "&amp;")
         .replace(/</g, "&lt;")
         .replace(/>/g, "&gt;")
         .replace(/"/g, "&quot;")
         .replace(/'/g, "&#039;");
 }
document.addEventListener('focus', function() {
    if (ws.readyState === WebSocket.OPEN) {
        return;
    } else {
        retryConnection();
        return;
    }
});

