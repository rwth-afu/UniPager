use pocsag::{TimeSlot, TimeSlots};
use std::sync::{Mutex, RwLock};

lazy_static! {
    pub static ref STATUS: RwLock<Status> = RwLock::new(Status::new());
//    pub static ref RESPONDER: Mutex<Option<Responder>> = Mutex::new(None);
}

#[derive(Serialize, Clone, Debug)]
pub struct Status {
    pub connected: bool,
    pub transmitting: bool,
    pub timeslots: TimeSlots,
    pub timeslot: TimeSlot,
    pub queue: usize,
    pub master: Option<String>,
    pub version: String,
    pub calls_tx: usize,
    pub calls_rx: usize
}

impl Status {
    pub fn new() -> Status {
        Status {
            connected: false,
            transmitting: false,
            timeslots: TimeSlots::new(),
            timeslot: TimeSlot::current(),
            queue: 0,
            master: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            calls_rx: 0,
            calls_tx: 0
        }
    }
}

/*pub fn subscribe(responder: Responder) {
    *RESPONDER.lock().unwrap() = Some(responder);
}*/

pub fn get() -> Status {
    STATUS.read().unwrap().clone()
}

macro_rules! status {
    ( $( $key:ident: $value:expr),* ) => ({
        let mut status = $crate::status::STATUS.write().unwrap();
        $(
            // Update only if the value has changed
            if status.$key != $value {
                status.$key = $value;
                /*
                // Send an update to connected frontent clients
                let res = $crate::status::RESPONDER.lock().unwrap();
                if let Some(ref res) = *res {
                    res.send(
                        $crate::frontend::Response::StatusUpdate(
                            stringify!($key).to_owned(),
                            json!($value)
                        )
                    );
                }
                 */
            }
        )*
    });
}

macro_rules! status_silent {
    ( $( $key:ident: $value:expr),* ) => ({
        let mut status = $crate::status::STATUS.write().unwrap();
        $(
            // Update only if the value has changed
            if status.$key != $value {
                status.$key = $value;
            }
        )*
    });
}

macro_rules! status_inc {
    ( $( $key:ident: $value:expr),* ) => ({
        let mut status = $crate::status::STATUS.write().unwrap();
        $(
            status.$key += $value;
        )*
    });
}
