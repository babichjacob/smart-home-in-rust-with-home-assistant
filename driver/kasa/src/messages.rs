use std::{collections::BTreeMap, fmt::Display, str::FromStr, time::Duration};

use deranged::{RangedU16, RangedU8};
use mac_address::{MacAddress, MacParseError};
use palette::{FromColor, Hsv};
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize};
use serde_repr::Deserialize_repr;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug)]
pub struct GetSysInfo;

impl Serialize for GetSysInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let target = "system";
        let cmd = "get_sysinfo";
        let arg: Option<()> = None;

        let mut top_level_map = serializer.serialize_map(Some(1))?;
        top_level_map.serialize_entry(target, &BTreeMap::from([(cmd, arg)]))?;
        top_level_map.end()
    }
}

#[derive(Debug, Deserialize)]
pub struct GetSysInfoResponse {
    pub system: GetSysInfoResponseSystem,
}

#[derive(Debug, Deserialize)]
pub struct GetSysInfoResponseSystem {
    pub get_sysinfo: SysInfo,
}

#[derive(Debug, Deserialize)]
pub struct CommonSysInfo {
    pub active_mode: ActiveMode,
    pub alias: String,
    pub ctrl_protocols: CtrlProtocols,
    pub description: String,
    pub dev_state: DevState,
    #[serde(rename = "deviceId")]
    pub device_id: DeviceId,
    pub disco_ver: String,
    pub err_code: i32, // No idea
    pub heapsize: u64, // No idea
    #[serde(rename = "hwId")]
    pub hw_id: HardwareId,
    pub hw_ver: String,
    pub is_color: IsColor,
    pub is_dimmable: IsDimmable,
    pub is_factory: bool,
    pub is_variable_color_temp: IsVariableColorTemp,
    pub light_state: LightState,
    pub mic_mac: MacAddressWithoutSeparators,
    pub mic_type: MicType,
    // model: Model,
    #[serde(rename = "oemId")]
    pub oem_id: OemId,
    pub preferred_state: Vec<PreferredStateChoice>,
    pub rssi: i32,
    pub sw_ver: String,
}

#[derive(Debug, Deserialize)]
pub struct LB130USSys {
    #[serde(flatten)]
    pub sys_info: CommonSysInfo,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "model")]
pub enum SysInfo {
    #[serde(rename = "LB130(US)")]
    LB130US(LB130USSys),
}

#[derive(Debug, Deserialize)]
pub struct PreferredStateChoice {
    #[serde(flatten)]
    pub color: Color,
}

#[derive(Debug, SerializeDisplay, DeserializeFromStr)]
struct MacAddressWithoutSeparators(MacAddress);

impl FromStr for MacAddressWithoutSeparators {
    type Err = MacParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [a, b, c, d, e, f, g, h, i, j, k, l] = s
            .as_bytes()
            .try_into()
            .map_err(|_| MacParseError::InvalidLength)?;

        let bytes = [(a, b), (c, d), (e, f), (g, h), (i, j), (k, l)];

        let mut digits = [0; 6];

        for (i, (one, two)) in bytes.into_iter().enumerate() {
            let slice = [one, two];
            let as_string = std::str::from_utf8(&slice).map_err(|_| MacParseError::InvalidDigit)?;
            let number =
                u8::from_str_radix(as_string, 16).map_err(|_| MacParseError::InvalidDigit)?;
            digits[i] = number;
        }

        Ok(MacAddressWithoutSeparators(MacAddress::new(digits)))
    }
}

impl Display for MacAddressWithoutSeparators {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Deserialize)]
enum ActiveMode {
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Deserialize)]
struct CtrlProtocols {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct DeviceId(pub String);

#[derive(Debug, Deserialize)]
enum DevState {
    #[serde(rename = "normal")]
    Normal,
}

#[derive(Debug, Deserialize)]
struct HardwareId(pub String);

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum IsColor {
    NoColor = 0,
    Color = 1,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum IsDimmable {
    NotDimmable = 0,
    Dimmable = 1,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum IsVariableColorTemp {
    NoVariableColorTemp = 0,
    VariableColorTemp = 1,
}

pub type Percentage = RangedU8<0, 100>;
pub type Angle = RangedU16<0, 360>;
pub type Kelvin = RangedU16<2500, 9000>;

#[derive(Debug, Clone)]
struct MaybeKelvin(Option<Kelvin>);

impl<'de> Deserialize<'de> for MaybeKelvin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match u16::deserialize(deserializer)? {
            0 => Ok(MaybeKelvin(None)),
            value => {
                let kelvin = Kelvin::try_from(value).map_err(|e| {
                    serde::de::Error::custom(format!(
                        "{value} is not in the range {}..{}",
                        Kelvin::MIN,
                        Kelvin::MAX
                    ))
                })?;
                Ok(MaybeKelvin(Some(kelvin)))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawColor {
    brightness: Percentage,
    color_temp: MaybeKelvin,
    hue: Angle,
    saturation: Percentage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hsb {
    hue: Angle,
    saturation: Percentage,
    brightness: Percentage,
}

impl<S> FromColor<Hsv<S, f64>> for Hsb {
    fn from_color(hsv: Hsv<S, f64>) -> Self {
        let (hue, saturation, value) = hsv.into_components();

        let hue = hue.into_positive_degrees();
        let hue = Angle::new_saturating(hue as u16);

        let saturation = saturation * (Percentage::MAX.get() as f64);
        let saturation = Percentage::new_saturating(saturation as u8);

        let brightness = value * (Percentage::MAX.get() as f64);
        let brightness = Percentage::new_saturating(brightness as u8);

        Hsb {
            hue,
            saturation,
            brightness,
        }
    }
}

#[derive(Debug, Clone)]
struct KelvinWithBrightness {
    kelvin: Kelvin,
    brightness: Percentage,
}

#[derive(Debug, Clone)]
enum Color {
    HSB(Hsb),
    KelvinWithBrightness(KelvinWithBrightness),
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_color = RawColor::deserialize(deserializer)?;

        let RawColor {
            brightness,
            color_temp,
            hue,
            saturation,
        } = raw_color;

        match color_temp.0 {
            Some(kelvin) => Ok(Color::KelvinWithBrightness(KelvinWithBrightness {
                kelvin,
                brightness,
            })),
            None => Ok(Color::HSB(Hsb {
                hue,
                saturation,
                brightness,
            })),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Off;

impl<'de> Deserialize<'de> for Off {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;

        if value == 0 {
            Ok(Off)
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(value.into()),
                &"0",
            ))
        }
    }
}

impl Serialize for Off {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct On;

impl<'de> Deserialize<'de> for On {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;

        if value == 1 {
            Ok(On)
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(value.into()),
                &"1",
            ))
        }
    }
}

impl Serialize for On {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(1)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LightState {
    On {
        on_off: On,
        #[serde(flatten)]
        color: Color,
        mode: LightStateMode,
    },
    Off {
        on_off: Off,
        dft_on_state: DftOnState,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct DftOnState {
    #[serde(flatten)]
    color: Color,
    mode: LightStateMode,
}

#[derive(Debug, Clone, Deserialize)]
enum LightStateMode {
    #[serde(rename = "normal")]
    Normal,
}

#[derive(Debug, Clone, Deserialize)]
enum MicType {
    #[serde(rename = "IOT.SMARTBULB")]
    IotSmartbulb,
}

#[derive(Debug, Clone, Deserialize)]
struct OemId(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct SetLightStateArgs {
    #[serde(flatten)]
    pub to: SetLightTo,
    pub transition: Option<Duration>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetLightOff {
    pub on_off: Off,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetLightLastOn {
    pub on_off: On,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetLightHsv {
    pub on_off: On,
    #[serde(flatten)]
    pub hsb: Hsb,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SetLightTo {
    Off(SetLightOff),
    LastOn(SetLightLastOn),
    Hsv(SetLightHsv),
    // TODO: kelvin
}

#[derive(Debug, Clone, derive_more::From)]
pub struct SetLightState(pub SetLightStateArgs);

impl Serialize for SetLightState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let target = "smartlife.iot.smartbulb.lightingservice";
        let cmd = "transition_light_state";
        let arg = &self.0;

        let mut top_level_map = serializer.serialize_map(Some(1))?;
        top_level_map.serialize_entry(target, &BTreeMap::from([(cmd, arg)]))?;
        top_level_map.end()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetLightStateResponse {
    // TODO
}
