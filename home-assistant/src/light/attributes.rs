use pyo3::prelude::*;

#[derive(Debug, FromPyObject)]
#[pyo3(from_item_all)]
pub struct LightAttributes {
    min_color_temp_kelvin: Option<u16>, // TODO: only here to allow compilation!
    max_color_temp_kelvin: Option<u16>, // TODO: only here to allow compilation!
}
