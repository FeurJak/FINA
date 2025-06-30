pub mod deserialize;
pub mod flags;
pub mod serialize;

use crate::{Compress, Validate, error::SerializationError, marshall::HashMarshaller};
pub use deserialize::{ArkDeserialize, ArkDeserializeWithFlags, ArkSerializeHashExt};
pub use flags::*;
pub use serialize::ArkSerialize;

pub trait Valid: Sync {
    fn check(&self) -> Result<(), SerializationError>;

    fn batch_check<'a>(
        batch: impl Iterator<Item = &'a Self> + Send,
    ) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        for item in batch {
            item.check()?;
        }

        Ok(())
    }
}
