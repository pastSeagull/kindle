use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::Manager;
use std::error::Error;
use std::time::Duration;
use tokio::time;

// Protocol Helpers
fn make_packet(byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte4: u8) -> Vec<u8> {
    let checksum = byte0 ^ byte1 ^ byte2 ^ byte3 ^ byte4;
    vec![byte0, byte1, byte2, byte3, byte4, checksum]
}

fn packet_mode_switch_cct() -> Vec<u8> {
    make_packet(0x01, 0x00, 0x00, 0x00, 0x00)
}
fn packet_mode_switch_hsi() -> Vec<u8> {
    make_packet(0x01, 0x01, 0x00, 0x00, 0x00)
}
fn header_ch_a() -> Vec<u8> {
    make_packet(0x02, 0x01, 0x00, 0x00, 0x00)
}
fn packet_val_word(val: u16) -> Vec<u8> {
    make_packet(
        0x03,
        (val & 0xFF) as u8,
        ((val >> 8) & 0xFF) as u8,
        0x00,
        0x00,
    )
}
fn packet_val_byte_05(val: u8) -> Vec<u8> {
    make_packet(0x05, val.min(100), 0x00, 0x00, 0x00)
}

pub async fn turn_light_on_logic() -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).ok_or("No Bluetooth adapter")?;

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    let peripherals = central.peripherals().await?;
    let mut target_light = None;
    for p in peripherals {
        let name = p
            .properties()
            .await?
            .unwrap_or_default()
            .local_name
            .unwrap_or_default();
        if crate::config::LIGHT_DEVICE_KEYWORDS
            .iter()
            .any(|k| name.contains(k))
        {
            target_light = Some(p);
            break;
        }
    }
    let light = target_light.ok_or("Light not found")?;

    light.connect().await?;
    light.discover_services().await?;

    let chars = light.characteristics();
    let cmd_char = chars
        .iter()
        .find(|c| c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE))
        .ok_or("No write characteristic")?;

    // ON Sequence
    light
        .write(
            cmd_char,
            &packet_val_byte_05(10),
            WriteType::WithoutResponse,
        )
        .await?;
    time::sleep(Duration::from_millis(20)).await;
    light
        .write(
            cmd_char,
            &packet_mode_switch_cct(),
            WriteType::WithoutResponse,
        )
        .await?;
    time::sleep(Duration::from_millis(20)).await;
    light
        .write(cmd_char, &header_ch_a(), WriteType::WithoutResponse)
        .await?;
    time::sleep(Duration::from_millis(20)).await;
    light
        .write(cmd_char, &packet_val_word(5600), WriteType::WithoutResponse)
        .await?;

    // Disconnect after action to save resources/allow others to connect?
    // Usually better to keep connection if frequent, but for this simple app, disconnect is safer.
    // However, the original code had "sleep 3s" then exit.
    // We will just disconnect.
    time::sleep(Duration::from_millis(500)).await;
    let _ = light.disconnect().await;
    Ok(())
}

pub async fn turn_light_off_logic() -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).ok_or("No Bluetooth adapter")?;

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    let peripherals = central.peripherals().await?;
    let mut target_light = None;
    for p in peripherals {
        let name = p
            .properties()
            .await?
            .unwrap_or_default()
            .local_name
            .unwrap_or_default();
        if crate::config::LIGHT_DEVICE_KEYWORDS
            .iter()
            .any(|k| name.contains(k))
        {
            target_light = Some(p);
            break;
        }
    }
    let light = target_light.ok_or("Light not found")?;

    light.connect().await?;
    light.discover_services().await?;

    let chars = light.characteristics();
    let cmd_char = chars
        .iter()
        .find(|c| c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE))
        .ok_or("No write characteristic")?;

    // OFF Sequence
    light
        .write(cmd_char, &packet_val_byte_05(0), WriteType::WithoutResponse)
        .await?;
    time::sleep(Duration::from_millis(10)).await;
    light
        .write(
            cmd_char,
            &packet_mode_switch_hsi(),
            WriteType::WithoutResponse,
        )
        .await?;

    time::sleep(Duration::from_millis(500)).await;
    let _ = light.disconnect().await;
    Ok(())
}
