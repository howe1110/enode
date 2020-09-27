use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub mstype: u16,
    pub msglen: u16,
    pub msgdata: Vec<u8>,
}
impl Message {}