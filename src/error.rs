use a653rs::prelude::*;

#[derive(Debug, Clone)]
pub enum QueuingRecvBufError<'a> {
    Apex(a653rs::prelude::Error),
    Postcard(postcard::Error, &'a [u8]),
}

impl From<a653rs::prelude::Error> for QueuingRecvBufError<'_> {
    fn from(e: a653rs::prelude::Error) -> Self {
        QueuingRecvBufError::Apex(e)
    }
}

#[derive(Debug, Clone)]
pub enum SamplingRecvBufError<'a> {
    Apex(a653rs::prelude::Error),
    Postcard(postcard::Error, Validity, &'a [u8]),
}

impl From<a653rs::prelude::Error> for SamplingRecvBufError<'_> {
    fn from(e: a653rs::prelude::Error) -> Self {
        SamplingRecvBufError::Apex(e)
    }
}

#[derive(Debug)]
pub enum SendError {
    Apex(a653rs::prelude::Error),
    Postcard(postcard::Error),
}

impl From<a653rs::prelude::Error> for SendError {
    fn from(e: a653rs::prelude::Error) -> Self {
        SendError::Apex(e)
    }
}

impl From<postcard::Error> for SendError {
    fn from(e: postcard::Error) -> Self {
        SendError::Postcard(e)
    }
}
