mod models;
mod xirr;

pub use models::{DateLike, InvalidPaymentsError, Payment};
pub use xirr::{xirr, xnpv};
