document.addEventListener('DOMContentLoaded', function() {
    // Connect to Rust WebSocket server (assuming it runs on localhost:8080)
    const ws = new WebSocket('ws://92.113.145.13:8080');
    const messagesDiv = document.getElementById('messages');

    ws.onopen = function() {
        console.log('Connected to WebSocket server');
        appendMessage('Connected to server');
        //getTerrain(document.getElementById('bottomDiv').offsetHeight, document.getElementById('bottomDiv').offsetWidth);
    };

    ws.onmessage = function(event) {
        console.log('Received:', event);
        if (event.type === "message") { 
            let data = JSON.parse(event.data);
            data.slice().reverse().forEach(message => {
                appendMessage(message[1], message[0]);
            });
        } else {
            console.log('Received:', event);
            console.log('Received:', event.data);
            appendMessage(`Received: ${event.data}`);
        }
    };

    ws.onclose = function() {
        console.log('Disconnected from WebSocket server');
        appendMessage('Disconnected from server');
    };

    ws.onerror = function(error) {
        console.error('WebSocket error:', error);
        appendMessage('Error: ' + error.message);
    };
    function fetchMessages() {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({type: "fetch_messages"}));
        }
    }
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

function appendMessage(user, message) {
    const messagesDiv = document.getElementById('messages');
    const messageElement = document.createElement('div');
    messageElement.className = 'message';
    messageElement.innerHTML = `<span class="user">${user}:</span><span class="text">${message}</span>`;
    messagesDiv.appendChild(messageElement);
    messagesDiv.scrollTop = messagesDiv.scrollHeight;
}
