use std::convert::Infallible;

use palette::{encoding::Srgb, Hsv, IntoColor};
use protocol::light::{GetState, Kelvin, SetState, TurnToColor, TurnToTemperature};
use snafu::{ResultExt, Snafu};

use crate::{
    connection::{HandleError, LB130USHandle},
    messages::{
        Angle, Hsb, LightState, Off, On, Percentage, SetLightHsv, SetLightLastOn, SetLightOff,
        SetLightStateArgs, SetLightTo,
    },
};

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum GetStateError {
    HandleError { source: HandleError },
}

impl GetState for LB130USHandle {
    type Error = GetStateError;

    async fn get_state(&self) -> Result<protocol::light::State, Self::Error> {
        let sys = self
            .get_sysinfo()
            .await
            .context(get_state_error::HandleSnafu)?;
        let light_state = sys.sys_info.light_state;
        let state = match light_state {
            LightState::On { .. } => protocol::light::State::On,
            LightState::Off { .. } => protocol::light::State::Off,
        };

        Ok(state)
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum SetStateError {
    HandleError { source: HandleError },
}

impl SetState for LB130USHandle {
    type Error = SetStateError;

    async fn set_state(&mut self, state: protocol::light::State) -> Result<(), Self::Error> {
        let to = match state {
            protocol::light::State::Off => SetLightTo::Off(SetLightOff { on_off: Off }),
            protocol::light::State::On => SetLightTo::LastOn(SetLightLastOn { on_off: On }),
        };

        let args = SetLightStateArgs {
            to,
            transition: None,
        };

        self.set_light_state(args)
            .await
            .context(set_state_error::HandleSnafu)?;

        Ok(())
    }
}

impl TurnToTemperature for LB130USHandle {
    type Error = Infallible; // TODO

    async fn turn_to_temperature(&mut self, temperature: Kelvin) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum TurnToColorError {
    HandleError { source: HandleError },
}

impl TurnToColor for LB130USHandle {
    type Error = TurnToColorError;

    async fn turn_to_color(&mut self, color: protocol::light::Oklch) -> Result<(), Self::Error> {
        let hsv: Hsv<Srgb, f64> = color.into_color();
        let hsb = hsv.into_color();

        self.set_light_state(SetLightStateArgs {
            to: SetLightTo::Hsv(SetLightHsv { on_off: On, hsb }),
            transition: None,
        })
        .await
        .context(turn_to_color_error::HandleSnafu)?;

        Ok(())
    }
}
