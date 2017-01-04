var socket;

function send() {
    var addr = $("#addr").val();
    var message = $("#message").val();

    localStorage && (localStorage.pager_addr = addr);

    var req = {"SendMessage": {
        "addr": parseInt(addr) || 0,
        "data": message
    }};

    socket.send(JSON.stringify(req));
    $("#message").val("");
}

function log(data) {
    var level = data[0] || "info";
    var msg = data[1] || "";

    switch (data[0]) {
    case 1: level = "error"; break;
    case 2: level = "warn"; break;
    case 3: level = "info"; break;
    case 4: level = "debug"; break;
    case 5: level = "trace"; break;
    default: level = "info";
    }

    var log = $("#log");
    log.append('<p><span class="log-level log-' + level + '">' + level + '</span> ' + msg + '</p>');
    log.scrollTop(log.height());
}

function version(data) {
    $("#version").html(data);
}

function shutdown() {
    socket.send('"Shutdown"');
}

function restart() {
    socket.send('"Restart"');
}

$(document).ready(function() {
    localStorage && $("#addr").val(localStorage.pager_addr || 0);

    socket = new WebSocket("ws://" + location.hostname + ":8055");

    socket.onopen = function(event) {
        socket.send('"GetVersion"');
        socket.send('"GetConfig"');
    };

    socket.onmessage = function(event) {
        var response = JSON.parse(event.data) || {};
        $.each(response, function(key, value) {
            switch (key) {
            case "Log": log(value); break;
            case "Version": version(value); break;
            default: console.log("Unknown: ", key, value);
            }
        });
    };

    socket.onclose = function(event) {

    };
});
