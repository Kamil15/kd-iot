pub mod mqttmsg {
    pub mod messages {
        include!(concat!(env!("OUT_DIR"), "/mqttmsg.messages.rs"));
    }
}