Kindle

A simple web server to control lights and display weather and sensor data and control light.

## Features

- Control Xiaomi lights
- Display weather and sensor data
- Schedule sensor scans
- Display weather and sensor data in a web interface

## Installation

```bash
cargo install kindle
```

## Usage

```bash
kindle
```

## Configuration

Configuration is done through the config.rs file.

```rust
// QWeather API Configuration
pub const WEATHER_API_HOST: &str = "";
pub const WEATHER_API_KEY: &str = "";
pub const WEATHER_LOCATION: &str = "";

// Xiaomi Sensor Configuration
pub const SENSOR_TARGET_UUID: &str = "";

// Light Control Configuration
// Keywords to identify the light device by name
pub const LIGHT_DEVICE_KEYWORDS: &[&str] = &["", "", ""];
```
