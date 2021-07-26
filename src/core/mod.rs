// TODO: move core module into a separate crate

mod models;
mod optimize;
mod periodic;
mod scheduled;

pub use models::{DateLike, InvalidPaymentsError};
pub use periodic::{fv, irr, mirr, nper, npv, pmt, pv};
pub use scheduled::{xirr, xnpv, xfv};
