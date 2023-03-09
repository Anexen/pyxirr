// TODO: move core module into a separate crate

mod models;
mod optimize;
pub mod periodic;
mod scheduled;

pub use models::{DateLike, InvalidPaymentsError};
pub use periodic::*;
pub use scheduled::{days_between, xfv, xirr, xnfv, xnpv, year_fraction, DayCount};
