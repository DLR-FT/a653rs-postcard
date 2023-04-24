use a653rs::bindings::*;
use a653rs::prelude::*;
use arrayvec::ArrayVec;
use postcard::de_flavors::Slice as DeSlice;
use postcard::ser_flavors::Slice as SerSlice;
use serde::{Deserialize, Serialize};

use crate::error::*;

pub trait QueuingPortSenderExt {
    fn send_type<T>(&self, p: T, timeout: SystemTime) -> Result<(), SendError>
    where
        T: Serialize;
}

pub trait QueuingPortReceiverExt<const MSG_SIZE: MessageSize> {
    fn recv_type<T>(&self, timeout: SystemTime) -> Result<T, QueuingRecvError<MSG_SIZE>>
    where
        T: for<'a> Deserialize<'a>,
        [u8; MSG_SIZE as usize]:;
}

impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, Q: ApexQueuingPortP4>
    QueuingPortSenderExt for QueuingPortSender<MSG_SIZE, NB_MSGS, Q>
where
    [u8; MSG_SIZE as usize]:,
{
    fn send_type<T>(&self, p: T, timeout: SystemTime) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let buf = &mut [0u8; MSG_SIZE as usize];
        let buf =
            postcard::serialize_with_flavor::<T, SerSlice, &mut [u8]>(&p, SerSlice::new(buf))?;
        self.send(buf, timeout).map_err(SendError::from)
    }
}

impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, Q: ApexQueuingPortP4>
    QueuingPortReceiverExt<MSG_SIZE> for QueuingPortReceiver<MSG_SIZE, NB_MSGS, Q>
where
    [u8; MSG_SIZE as usize]:,
{
    fn recv_type<T>(&self, timeout: SystemTime) -> Result<T, QueuingRecvError<MSG_SIZE>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut msg_buf = [0u8; MSG_SIZE as usize];
        let msg = self.receive(&mut msg_buf, timeout)?;
        let msg_slice = DeSlice::new(msg);
        let mut deserializer = postcard::Deserializer::from_flavor(msg_slice);
        match T::deserialize(&mut deserializer) {
            Ok(t) => Ok(t),
            Err(e) => {
                let mut msg = ArrayVec::from(msg_buf);
                msg.truncate(msg.len());
                Err(QueuingRecvError::Postcard(e, msg))
            }
        }
    }
}
