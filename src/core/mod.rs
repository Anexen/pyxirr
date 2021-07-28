// TODO: move core module into a separate crate

mod models;
mod optimize;
mod periodic;
mod scheduled;

pub use models::{DateLike, InvalidPaymentsError};
pub use periodic::{fv, irr, mirr, nfv, nper, npv, pmt, pv};
pub use scheduled::{xfv, xirr, xnfv, xnpv};
