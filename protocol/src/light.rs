use std::{error::Error, future::Future};

use deranged::RangedU16;
use palette::Oklch;
use snafu::{ResultExt, Snafu};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::Display, strum::EnumIs,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum State {
    Off,
    On,
}

impl State {
    pub const fn invert(self) -> Self {
        match self {
            State::Off => State::On,
            State::On => State::Off,
        }
    }
}

impl From<bool> for State {
    fn from(bool: bool) -> Self {
        if bool {
            State::On
        } else {
            State::Off
        }
    }
}

impl From<State> for bool {
    fn from(state: State) -> Self {
        state.is_on()
    }
}

pub trait GetState {
    type Error: Error;
    fn get_state(&self) -> impl Future<Output = Result<State, Self::Error>> + Send;
}

#[ext_trait::extension(trait IsOff)]
impl<T: GetState> T {
    async fn is_off(&self) -> Result<bool, T::Error> {
        Ok(self.get_state().await?.is_off())
    }
}

#[ext_trait::extension(trait IsOn)]
impl<T: GetState> T {
    async fn is_on(&self) -> Result<bool, T::Error> {
        Ok(self.get_state().await?.is_on())
    }
}

pub trait SetState {
    type Error: Error;
    fn set_state(&mut self, state: State) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[ext_trait::extension(trait TurnOff)]
impl<T: SetState> T {
    async fn turn_off(&mut self) -> Result<(), T::Error> {
        self.set_state(State::Off).await
    }
}

#[ext_trait::extension(trait TurnOn)]
impl<T: SetState> T {
    async fn turn_on(&mut self) -> Result<(), T::Error> {
        self.set_state(State::On).await
    }
}

#[derive(Debug, Clone, Snafu)]
enum InvertToToggleError<GetStateError: Error + 'static, SetStateError: Error + 'static> {
    GetStateError { source: GetStateError },
    SetStateError { source: SetStateError },
}

#[ext_trait::extension(trait InvertToToggle)]
impl<T: GetState + SetState> T
where
    <T as GetState>::Error: 'static,
    <T as SetState>::Error: 'static,
{
    /// Toggle the light by setting it to the inverse of its current state
    async fn toggle(
        &mut self,
    ) -> Result<(), InvertToToggleError<<T as GetState>::Error, <T as SetState>::Error>> {
        let state = self.get_state().await.context(GetStateSnafu)?;
        self.set_state(state.invert())
            .await
            .context(SetStateSnafu)?;

        Ok(())
    }
}

pub trait Toggle {
    type Error: Error;
    fn toggle(&mut self, state: State) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[derive(Debug, Clone, Copy, derive_more::From, derive_more::Into)]
pub struct Kelvin(pub RangedU16<2000, 10000>);

pub trait TurnToTemperature {
    type TurnToTemperatureError: Error;
    fn turn_to_temperature(
        &mut self,
        temperature: Kelvin,
    ) -> impl Future<Output = Result<(), Self::TurnToTemperatureError>> + Send;
}

pub trait TurnToColor {
    type TurnToColorError: Error;
    fn turn_to_color(
        &mut self,
        color: Oklch<f64>,
    ) -> impl Future<Output = Result<(), Self::TurnToColorError>> + Send;
}
