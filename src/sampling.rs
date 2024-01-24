use a653rs::prelude::*;
use arrayvec::ArrayVec;
use postcard::de_flavors::Slice as DeSlice;
use postcard::ser_flavors::Slice as SerSlice;
use serde::{Deserialize, Serialize};

use crate::error::*;

pub trait SamplingPortSourceExt {
    fn send_type<T>(&self, p: T) -> Result<(), SendError>
    where
        T: Serialize;
}

pub trait SamplingPortDestinationExt<const MSG_SIZE: MessageSize> {
    fn recv_type<T>(&self) -> Result<(Validity, T), SamplingRecvError<MSG_SIZE>>
    where
        T: for<'a> Deserialize<'a>,
        [u8; MSG_SIZE as usize]:;
}

impl<const MSG_SIZE: MessageSize, Q: ApexSamplingPortP4Ext> SamplingPortSourceExt
    for SamplingPortSource<MSG_SIZE, Q>
where
    [u8; MSG_SIZE as usize]:,
{
    fn send_type<T>(&self, p: T) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let buf = &mut [0u8; MSG_SIZE as usize];
        let buf =
            postcard::serialize_with_flavor::<T, SerSlice, &mut [u8]>(&p, SerSlice::new(buf))?;
        self.send(buf).map_err(SendError::from)
    }
}

impl<const MSG_SIZE: MessageSize, Q: ApexSamplingPortP4Ext> SamplingPortDestinationExt<MSG_SIZE>
    for SamplingPortDestination<MSG_SIZE, Q>
where
    [u8; MSG_SIZE as usize]:,
{
    fn recv_type<T>(&self) -> Result<(Validity, T), SamplingRecvError<MSG_SIZE>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut msg_buf = [0u8; MSG_SIZE as usize];
        let (val, msg) = self.receive(&mut msg_buf)?;
        let msg_slice = DeSlice::new(msg);
        let mut deserializer = postcard::Deserializer::from_flavor(msg_slice);
        match T::deserialize(&mut deserializer) {
            Ok(t) => Ok((val, t)),
            Err(e) => {
                let mut msg = ArrayVec::from(msg_buf);
                msg.truncate(msg.len());
                Err(SamplingRecvError::Postcard(e, val, msg))
            }
        }
    }
}
