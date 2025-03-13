use strum::EnumString;

#[derive(Debug, Clone, EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Domain {
    Automation,
    BinarySensor,
    Button,
    Camera,
    Climate,
    Conversation,
    Cover,
    DeviceTracker,
    Group,
    InputDatetime,
    InputNumber,
    InputSelect,
    InputText,
    Light,
    Lock,
    MediaPlayer,
    Notify,
    Person,
    Remote,
    Scene,
    Select,
    Sensor,
    Sun,
    Switch,
    Tag,
    Update,
    Weather,
    Zone,
}
