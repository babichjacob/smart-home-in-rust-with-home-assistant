use std::{error::Error, future::Future};

use deranged::RangedU16;

pub trait Light {
    type IsOnError: Error;
    fn is_on(&self) -> impl Future<Output = Result<bool, Self::IsOnError>> + Send;

    type IsOffError: Error;
    fn is_off(&self) -> impl Future<Output = Result<bool, Self::IsOffError>> + Send;

    type TurnOnError: Error;
    fn turn_on(&mut self) -> impl Future<Output = Result<(), Self::TurnOnError>> + Send;

    type TurnOffError: Error;
    fn turn_off(&mut self) -> impl Future<Output = Result<(), Self::TurnOffError>> + Send;

    type ToggleError: Error;
    fn toggle(&mut self) -> impl Future<Output = Result<(), Self::ToggleError>> + Send;
}

#[derive(Debug, Clone, Copy, derive_more::From, derive_more::Into)]
pub struct Kelvin(pub RangedU16<2000, 10000>);

pub trait KelvinLight: Light {
    type TurnToKelvinError: Error;
    fn turn_to_kelvin(
        &mut self,
        temperature: Kelvin,
    ) -> impl Future<Output = Result<(), Self::TurnToKelvinError>> + Send;
}

// TODO: replace with a type from a respected and useful library
#[derive(Debug, Clone, Copy, derive_more::From, derive_more::Into)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub trait RgbLight: Light {
    type TurnToRgbError: Error;
    fn turn_to_rgb(
        &mut self,
        color: Rgb,
    ) -> impl Future<Output = Result<(), Self::TurnToRgbError>> + Send;
}
