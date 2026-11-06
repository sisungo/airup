use crate::system::{ConnectionExt, Event};

#[unsafe(no_mangle)]
pub extern "C" fn Airup_Notify() {
    if let Some(mut conn) = connect() {
        if let Ok(service_name) = std::env::var("AIRUP_SERVICE") {
            _ = conn.trigger_event(&Event::new("notify_active".into(), service_name));
        }
    }
}

fn connect() -> Option<crate::blocking::Connection> {
    crate::blocking::Connection::connect(crate::socket_path()).ok()
}
