var vm = new Vue({
    el: "#wrapper",
    created() {
        this.socket = new WebSocket("ws://" + location.hostname + ":8055");
        this.socket.onopen = this.onopen;
        this.socket.onmessage = this.onmessage;
        this.socket.onclose = this.onclose;
    },
    data: {
        version: "",
        log: [],
        socket: null,
        config: {
            master: {},
            audio: {},
            c9000: {},
            raspager: {}
        },
        message: "",
        addr: localStorage ? (localStorage.pager_addr || 0) : 0
    },
    methods: {
        onopen: function(event) {
            this.send("GetVersion");
            this.send("GetConfig");
        },
        onmessage: function(event) {
            var response = JSON.parse(event.data) || {};
            for (var key in response) {
                var value = response[key];
                switch (key) {
                    case "Log": this.log_append(value); break;
                    case "Version": this.version = value; break;
                    case "Config": this.config = value; break;
                    default: console.log("Unknown Key: ", key);
                }
            }
        },
        onclose: function(event) {

        },
        send: function(data) {
            this.socket.send(JSON.stringify(data));
        },
        log_append: function(record) {
            var level = record[0] || "info";
            var msg = record[1] || "";
            switch (record[0]) {
                case 1: level = "error"; break;
                case 2: level = "warn"; break;
                case 3: level = "info"; break;
                case 4: level = "debug"; break;
                case 5: level = "trace"; break;
                default: level = "info";
            }
            this.log.push({level: level, msg: msg});

            var container = this.$el.querySelector("#log");
            container.scrollTop = container.scrollHeight + 1e10;
        },
        restart: function(event) {
            this.send("Restart");
        },
        shutdown: function(event) {
            this.send("Shutdown");
        },
        save_config: function(event) {
            if (this.config) {
                this.send({"SetConfig": this.config});
            }
        },
        default_config: function(event) {
            this.send("DefaultConfig");
        },
        send_message: function(event)  {
            localStorage && (localStorage.pager_addr = this.addr);

            var req = {"SendMessage": {
                "addr": parseInt(this.addr) || 0,
                "data": this.message
            }};

            this.send(req);
            this.message = "";
        }
    }
});
