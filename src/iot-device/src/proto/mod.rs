pub mod mqttmsg {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/mqttmsg.items.rs"));
    }
}