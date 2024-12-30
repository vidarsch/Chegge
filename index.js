document.addEventListener('DOMContentLoaded', function() {
    const nameInput = document.querySelector('.name-input');
    const ws = new WebSocket('ws://92.113.145.13:8080');

    function sendName(name) {
        if (ws.readyState === WebSocket.OPEN && name.length > 0) {
            ws.send(JSON.stringify({
                type: "update_name",
                name: name
            }));
            //nameInput.value = '';
            console.log('Sent name:', name);
        }
    }

    nameInput.addEventListener('keypress', function(e) {
        let mess = messageInput = document.querySelector('.message-input').value;
        if (e.key === 'Enter' && mess.length > 0) {
            console.log('Enter pressed');
            sendName(this.value);
        }
    });

    const messageInput = document.querySelector('.message-input');

    function sendMessage(message, name) {
        if (ws.readyState === WebSocket.OPEN && message.length > 0) {
            name = name.length > 0 ? name : "Anonymous";
            ws.send(JSON.stringify({
                type: "message",
                message: message,
                name: name
            }));
            messageInput.value = '';
            
            console.log('Sent message:', message);
        } else {
            appendMessage("Laggbugg", "Något gick fel");
        }
    }

    messageInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter') {
            console.log('Enter pressed');
            sendMessage(this.value,document.querySelector('.name-input').value);
        }
    });

});

