mod cursor;
mod error;
mod read;
mod write;

use alloc::vec::Vec;
use core::{cmp, convert::TryInto, mem};

pub use cursor::*;
pub use error::*;
pub use read::*;
pub use write::*;

pub mod prelude {
    pub use super::{Read, Result, Write};
}

#[inline]
fn slice_write(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> Result<usize> {
    let pos = cmp::min(*pos_mut, slice.len() as u64);
    let amt = (&mut slice[(pos as usize)..]).write(buf)?;
    *pos_mut += amt as u64;
    Ok(amt)
}

fn vec_write(pos_mut: &mut u64, vec: &mut Vec<u8>, buf: &[u8]) -> Result<usize> {
    let pos: usize = (*pos_mut).try_into().map_err(|_| {
        Error::new(
            ErrorKind::InvalidInput,
            "cursor position exceeds maximum possible vector length",
        )
    })?;
    let len = vec.len();
    if len < pos {
        vec.resize(pos, 0);
    }
    {
        let space = vec.len() - pos;
        let (left, right) = buf.split_at(cmp::min(space, buf.len()));
        vec[pos..pos + left.len()].copy_from_slice(left);
        vec.extend_from_slice(right);
    }
    *pos_mut = (pos + buf.len()) as u64;
    Ok(buf.len())
}
