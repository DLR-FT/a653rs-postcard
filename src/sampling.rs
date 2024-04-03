#[cfg(feature = "alloc")]
use alloc::vec;

#[cfg(feature = "alloc")]
extern crate alloc;

use a653rs::prelude::*;
use postcard::de_flavors::Slice as DeSlice;
use postcard::ser_flavors::Slice as SerSlice;
use serde::{Deserialize, Serialize};

use crate::error::*;

pub trait SamplingPortSourceExt {
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T) -> Result<(), SendError>
    where
        T: Serialize;

    fn send_type_buf<T>(&self, p: T, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize;
}

pub trait SamplingPortDestinationExt {
    #[cfg(feature = "alloc")]
    fn recv_type<T>(&self) -> Result<(Validity, T), SamplingRecvError>
    where
        T: for<'a> Deserialize<'a>;

    fn recv_type_buf<'a, T>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<(Validity, T), SamplingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>;
}

impl<Q: ApexSamplingPortP4Ext> SamplingPortSourceExt for SamplingPortSource<Q> {
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let msg = postcard::to_allocvec(&p)?;
        self.send(&msg).map_err(SendError::from)
    }

    fn send_type_buf<T>(&self, p: T, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let buf =
            postcard::serialize_with_flavor::<T, SerSlice, &mut [u8]>(&p, SerSlice::new(buf))?;
        self.send(buf).map_err(SendError::from)
    }
}

impl<Q: ApexSamplingPortP4Ext> SamplingPortDestinationExt for SamplingPortDestination<Q> {
    #[cfg(feature = "alloc")]
    fn recv_type<'a, T>(&self) -> Result<(Validity, T), SamplingRecvError>
    where
        T: for<'b> Deserialize<'b>,
    {
        let mut buf = vec![0; self.size() as usize];
        let (val, msg) = self.receive(&mut buf)?;
        match postcard::from_bytes(msg) {
            Ok(t) => Ok((val, t)),
            Err(e) => {
                let msg_len = msg.len();
                buf.truncate(msg_len);
                Err(SamplingRecvError::Postcard(e, val, buf))
            }
        }
    }

    fn recv_type_buf<'a, T>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<(Validity, T), SamplingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>,
    {
        let (val, msg) = self.receive(buf)?;
        let msg_slice = DeSlice::new(msg);
        let mut deserializer = postcard::Deserializer::from_flavor(msg_slice);
        match T::deserialize(&mut deserializer) {
            Ok(t) => Ok((val, t)),
            Err(e) => Err(SamplingRecvBufError::Postcard(e, val, msg)),
        }
    }
}
