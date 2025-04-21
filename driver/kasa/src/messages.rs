use std::{collections::BTreeMap, fmt::Display, str::FromStr};

use deranged::{RangedU16, RangedU8};
use mac_address::{MacAddress, MacParseError};
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
    active_mode: ActiveMode,
    alias: String,
    ctrl_protocols: CtrlProtocols,
    description: String,
    dev_state: DevState,
    #[serde(rename = "deviceId")]
    device_id: DeviceId,
    disco_ver: String,
    err_code: i32, // No idea
    heapsize: u64, // No idea
    #[serde(rename = "hwId")]
    hw_id: HardwareId,
    hw_ver: String,
    is_color: IsColor,
    is_dimmable: IsDimmable,
    is_factory: bool,
    is_variable_color_temp: IsVariableColorTemp,
    light_state: LightState,
    mic_mac: MacAddressWithoutSeparators,
    mic_type: MicType,
    // model: Model,
    #[serde(rename = "oemId")]
    oem_id: OemId,
    preferred_state: Vec<PreferredStateChoice>,
    rssi: i32,
    sw_ver: String,
}

#[derive(Debug, Deserialize)]
pub struct LB130USSys {
    #[serde(flatten)]
    sys_info: CommonSysInfo,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "model")]
pub enum SysInfo {
    #[serde(rename = "LB130(US)")]
    LB130US(LB130USSys),
}

#[derive(Debug, Deserialize)]
struct PreferredStateChoice {
    #[serde(flatten)]
    color: Color,
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

type Percentage = RangedU8<0, 100>;
type Angle = RangedU16<0, 360>;
type Kelvin = RangedU16<2500, 9000>;

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

#[derive(Debug, Clone)]
struct Hsb {
    hue: Angle,
    saturation: Percentage,
    brightness: Percentage,
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

#[derive(Debug, Clone, Deserialize)]
struct LightState {
    #[serde(flatten)]
    color: Color,
    mode: LightStateMode,
    on_off: OnOrOff,
}

#[derive(Debug, Clone, Deserialize)]
enum LightStateMode {
    #[serde(rename = "normal")]
    Normal,
}

#[derive(Debug, Clone, Deserialize_repr)]
#[repr(u8)]
#[non_exhaustive]
enum OnOrOff {
    Off = 0,
    On = 1,
}

#[derive(Debug, Clone, Deserialize)]
enum MicType {
    #[serde(rename = "IOT.SMARTBULB")]
    IotSmartbulb,
}

#[derive(Debug, Clone, Deserialize)]
struct OemId(pub String);
