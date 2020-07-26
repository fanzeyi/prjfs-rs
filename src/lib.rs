#![feature(try_trait)]

pub mod conv;
pub mod guid;
pub mod option;
pub mod provider;

pub use crate::{
    option::{NotificationType, OptionBuilder},
    provider::{Provider, ProviderT},
};
pub use winapi::um::projectedfslib as sys;
