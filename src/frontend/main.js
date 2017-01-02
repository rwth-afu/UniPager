var socket = new WebSocket("ws://" + location.hostname + ":2794");

socket.onmessage = function(event) {
    var received = document.getElementById("received");
    var br = document.createElement("BR");
    var text = document.createTextNode(event.data);
    received.appendChild(br);
    received.appendChild(text);
};

function send() {
    var addr = document.getElementById("addr");
    var message = document.getElementById("message");

    var req = {"SendMessage": {
        "addr": parseInt(addr.value) || 0,
        "message": message.value
    }};

    socket.send(JSON.stringify(req));
    message.value="";
}
