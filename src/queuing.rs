#[cfg(feature = "alloc")]
use alloc::vec;

#[cfg(feature = "alloc")]
extern crate alloc;

use a653rs::prelude::*;
use postcard::de_flavors::Slice as DeSlice;
use postcard::ser_flavors::Slice as SerSlice;
use serde::{Deserialize, Serialize};

use crate::error::*;

pub trait QueuingPortSenderExt {
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T, timeout: SystemTime) -> Result<(), SendError>
    where
        T: Serialize;

    fn send_type_buf<T>(&self, p: T, timeout: SystemTime, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize;
}

pub trait QueuingPortReceiverExt {
    #[cfg(feature = "alloc")]
    fn recv_type<T>(&self, timeout: SystemTime) -> Result<(T, QueueOverflow), QueuingRecvError>
    where
        T: for<'a> Deserialize<'a>;

    fn recv_type_buf<'a, T>(
        &'a self,
        timeout: SystemTime,
        buf: &'a mut [u8],
    ) -> Result<(T, QueueOverflow), QueuingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>;
}

impl<Q: ApexQueuingPortP4Ext> QueuingPortSenderExt for QueuingPortSender<Q> {
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T, timeout: SystemTime) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let msg = postcard::to_allocvec(&p)?;
        self.send(&msg, timeout).map_err(SendError::from)
    }

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
    #[cfg(feature = "alloc")]
    fn recv_type<T>(&self, timeout: SystemTime) -> Result<(T, QueueOverflow), QueuingRecvError>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut buf = vec![0; self.size()];
        let (msg, overflow) = self.receive(&mut buf, timeout)?;
        match postcard::from_bytes(msg) {
            Ok(t) => Ok((t, overflow)),
            Err(e) => {
                let msg_len = msg.len();
                buf.truncate(msg_len);
                Err(QueuingRecvError::Postcard(e, buf))
            }
        }
    }

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
