// TODO: refactor
// TODO: move core module into a separate crate

mod fv;
mod irr;
mod mirr;
mod models;
mod optimize;
mod pmt;
mod pv;
mod xirr;

pub use fv::{fv, xfv};
pub use irr::{irr, npv};
pub use mirr::mirr;
pub use models::{DateLike, InvalidPaymentsError};
pub use pmt::{nper, pmt};
pub use pv::pv;
pub use xirr::{xirr, xnpv};
