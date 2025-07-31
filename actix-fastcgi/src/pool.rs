use deadpool::managed;

use crate::{Error, SockStream, stream::StreamAddr};

pub type SockPool = managed::Pool<Manager>;

pub struct Manager(pub(crate) StreamAddr);

impl managed::Manager for Manager {
    type Type = SockStream;
    type Error = Error;

    #[inline]
    async fn create(&self) -> Result<Self::Type, Error> {
        SockStream::connect(&self.0).await
    }

    async fn recycle(
        &self,
        _: &mut SockStream,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Error> {
        Ok(())
    }
}
