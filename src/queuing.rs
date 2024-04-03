use a653rs::prelude::*;
use postcard::de_flavors::Slice as DeSlice;
use postcard::ser_flavors::Slice as SerSlice;
use serde::{Deserialize, Serialize};

use crate::error::*;

pub trait QueuingPortSenderExt {
    fn send_type_buf<T>(&self, p: T, timeout: SystemTime, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize;
}

pub trait QueuingPortReceiverExt {
    fn recv_type_buf<'a, T>(
        &'a self,
        timeout: SystemTime,
        buf: &'a mut [u8],
    ) -> Result<(T, QueueOverflow), QueuingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>;
}

impl<Q: ApexQueuingPortP4Ext> QueuingPortSenderExt for QueuingPortSender<Q> {
    fn send_type_buf<T>(&self, p: T, timeout: SystemTime, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let buf =
            postcard::serialize_with_flavor::<T, SerSlice, &mut [u8]>(&p, SerSlice::new(buf))?;
        self.send(buf, timeout).map_err(SendError::from)
    }
}

impl<Q: ApexQueuingPortP4Ext> QueuingPortReceiverExt for QueuingPortReceiver<Q> {
    fn recv_type_buf<'a, T>(
        &self,
        timeout: SystemTime,
        buf: &'a mut [u8],
    ) -> Result<(T, QueueOverflow), QueuingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>,
    {
        let (msg, overflow) = self.receive(buf, timeout)?;
        let msg_slice = DeSlice::new(msg);
        let mut deserializer = postcard::Deserializer::from_flavor(msg_slice);
        match T::deserialize(&mut deserializer) {
            Ok(t) => Ok((t, overflow)),
            Err(e) => Err(QueuingRecvBufError::Postcard(e, msg)),
        }
    }
}
