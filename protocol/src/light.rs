use std::error::Error;

use deranged::RangedU16;

pub trait Light {
    type IsOnError: Error;
    async fn is_on(&self) -> Result<bool, Self::IsOnError>;

    type IsOffError: Error;
    async fn is_off(&self) -> Result<bool, Self::IsOffError>;

    type TurnOnError: Error;
    async fn turn_on(&mut self) -> Result<(), Self::TurnOnError>;

    type TurnOffError: Error;
    async fn turn_off(&mut self) -> Result<(), Self::TurnOffError>;

    type ToggleError: Error;
    async fn toggle(&mut self) -> Result<(), Self::ToggleError>;
}

#[derive(Debug, Clone, Copy, derive_more::From, derive_more::Into)]
pub struct Kelvin(pub RangedU16<2000, 10000>);

pub trait KelvinLight: Light {
    type TurnToKelvinError: Error;
    async fn turn_to_kelvin(&mut self, temperature: Kelvin) -> Result<(), Self::TurnToKelvinError>;
}

// TODO: replace with a type from a respected and useful library
#[derive(Debug, Clone, Copy, derive_more::From, derive_more::Into)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub trait RgbLight: Light {
    type TurnToRgbError: Error;
    async fn turn_to_rgb(&mut self, color: Rgb) -> Result<(), Self::TurnToRgbError>;
}
