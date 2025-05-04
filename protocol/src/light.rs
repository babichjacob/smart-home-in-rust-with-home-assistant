use std::{error::Error, future::Future};

use deranged::RangedU16;
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

#[ext_trait::extension(pub trait IsOff)]
impl<T: GetState> T {
    async fn is_off(&self) -> Result<bool, T::Error> {
        Ok(self.get_state().await?.is_off())
    }
}

#[ext_trait::extension(pub trait IsOn)]
impl<T: GetState> T {
    async fn is_on(&self) -> Result<bool, T::Error> {
        Ok(self.get_state().await?.is_on())
    }
}

pub trait SetState {
    type Error: Error;
    fn set_state(&mut self, state: State) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[ext_trait::extension(pub trait TurnOff)]
impl<T: SetState> T {
    async fn turn_off(&mut self) -> Result<(), T::Error> {
        self.set_state(State::Off).await
    }
}

#[ext_trait::extension(pub trait TurnOn)]
impl<T: SetState> T {
    async fn turn_on(&mut self) -> Result<(), T::Error> {
        self.set_state(State::On).await
    }
}

pub trait Toggle {
    type Error: Error;
    fn toggle(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[derive(Debug, Clone, Snafu)]
pub enum InvertToToggleError<GetStateError: Error + 'static, SetStateError: Error + 'static> {
    GetStateError { source: GetStateError },
    SetStateError { source: SetStateError },
}

impl<T: GetState + SetState + Send> Toggle for T
where
    <T as GetState>::Error: 'static,
    <T as SetState>::Error: 'static,
{
    type Error = InvertToToggleError<<T as GetState>::Error, <T as SetState>::Error>;
    /// Toggle the light by setting it to the inverse of its current state
    async fn toggle(&mut self) -> Result<(), Self::Error> {
        let state = self.get_state().await.context(GetStateSnafu)?;
        self.set_state(state.invert())
            .await
            .context(SetStateSnafu)?;

        Ok(())
    }
}

pub type Kelvin = RangedU16<2000, 10000>;

pub trait TurnToTemperature {
    type Error: Error;
    fn turn_to_temperature(
        &mut self,
        temperature: Kelvin,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub type Oklch = palette::Oklch<f64>;

pub trait TurnToColor {
    type Error: Error;
    fn turn_to_color(
        &mut self,
        color: Oklch,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
