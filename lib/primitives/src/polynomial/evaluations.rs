mod multivariate;
mod univariate;

use super::{domain::*, traits::*, univariate::*};
use crate::{cfg_iter_mut, curve::batch_inversion, field::traits::FftField};
pub use univariate::Evaluations;
