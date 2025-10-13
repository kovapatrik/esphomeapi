use esphomeapi_manager::entity::ColorMode as RustColorMode;
use napi_derive::napi;

#[napi]
pub enum ColorMode {
  Unknown = 0,
  OnOff = 1,
  Brightness = 2,
  White = 7,
  ColorTemperature = 11,
  ColdWarmWhite = 19,
  RGB = 35,
  RGBWhite = 39,
  RGBColorTemperature = 47,
  RGBColdWarmWhite = 51,
}

impl From<RustColorMode> for ColorMode {
  fn from(color_mode: RustColorMode) -> Self {
    match color_mode {
      RustColorMode::Unknown => ColorMode::Unknown,
      RustColorMode::OnOff => ColorMode::OnOff,
      RustColorMode::Brightness => ColorMode::Brightness,
      RustColorMode::White => ColorMode::White,
      RustColorMode::ColorTemperature => ColorMode::ColorTemperature,
      RustColorMode::ColdWarmWhite => ColorMode::ColdWarmWhite,
      RustColorMode::RGB => ColorMode::RGB,
      RustColorMode::RGBWhite => ColorMode::RGBWhite,
      RustColorMode::RGBColorTemperature => ColorMode::RGBColorTemperature,
      RustColorMode::RGBColdWarmWhite => ColorMode::RGBColdWarmWhite,
    }
  }
}

impl From<ColorMode> for RustColorMode {
  fn from(color_mode: ColorMode) -> Self {
    match color_mode {
      ColorMode::Unknown => RustColorMode::Unknown,
      ColorMode::OnOff => RustColorMode::OnOff,
      ColorMode::Brightness => RustColorMode::Brightness,
      ColorMode::White => RustColorMode::White,
      ColorMode::ColorTemperature => RustColorMode::ColorTemperature,
      ColorMode::ColdWarmWhite => RustColorMode::ColdWarmWhite,
      ColorMode::RGB => RustColorMode::RGB,
      ColorMode::RGBWhite => RustColorMode::RGBWhite,
      ColorMode::RGBColorTemperature => RustColorMode::RGBColorTemperature,
      ColorMode::RGBColdWarmWhite => RustColorMode::RGBColdWarmWhite,
    }
  }
}
