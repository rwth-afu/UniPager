var vm = new Vue({
    el: "#wrapper",
    created() {
        this.connect();
    },
    data: {
        connected: false,
        version: "",
        log: [],
        socket: null,
        config: {
            master: {},
            audio_gpio: {},
            c9000: {},
            raspager: {}
        },
        status: {},
        message: "",
        addr: localStorage ? (localStorage.pager_addr || 0) : 0
    },
    methods: {
        connect: function(event) {
            this.socket = new WebSocket("ws://" + location.hostname + ":8055");
            this.socket.onopen = this.onopen;
            this.socket.onmessage = this.onmessage;
            this.socket.onclose = this.onclose;
        },
        onopen: function(event) {
            this.connected = true;
            this.log.push({msg: "Connected to RustPager."});
            this.log_scroll();
            this.send("GetVersion");
            this.send("GetConfig");
            this.send("GetStatus");
        },
        onmessage: function(event) {
            var response = JSON.parse(event.data) || {};
            for (var key in response) {
                var value = response[key];
                switch (key) {
                    case "Log": this.log_append(value); break;
                    case "Version": this.version = value; break;
                    case "Config": this.config = value; break;
                    case "Status": this.status = value; break;
                    default: console.log("Unknown Key: ", key);
                }
            }
        },
        onclose: function(event) {
            if (this.connected) {
                this.log.push({msg: "Disconnected from RustPager."});
                this.log_scroll();
            }
            this.connected = false;
            setTimeout(function() { this.connect(); }.bind(this), 1000);
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
            this.log_scroll();
        },
        log_scroll: function() {
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
