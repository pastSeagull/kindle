use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

const TARGET_UUID: &str = "66831b50-1daf-180a-729c-4ecebfbd146b";

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct SensorData {
    pub temp: f64,
    pub humi: u8,
    pub batt: u8,
    pub last_update: u64, // Unix timestamp
}

pub async fn fetch_sensor_data_once(
    shared_data: Arc<RwLock<Option<SensorData>>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .nth(0)
        .ok_or("No Bluetooth adapter found")?;

    central.start_scan(ScanFilter::default()).await?;
    println!("=== Scanning for Sensor ({}) ===", TARGET_UUID);

    let mut events = central.events().await?;

    // Wait for the first valid event
    while let Some(event) = events.next().await {
        if let btleplug::api::CentralEvent::ServiceDataAdvertisement { id, service_data } = event {
            if id.to_string() == TARGET_UUID {
                for (_, data) in service_data {
                    if let Some((temp, humi, batt)) = parse_atc(&data) {
                        let now = SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_secs();

                        let new_data = SensorData {
                            temp,
                            humi,
                            batt,
                            last_update: now,
                        };

                        // Update shared state
                        if let Ok(mut lock) = shared_data.write() {
                            *lock = Some(new_data);
                        }

                        println!("Sensor updated: {:.1}°C {}%", temp, humi);
                        // Stop scanning and return
                        let _ = central.stop_scan().await;
                        return Ok(());
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_atc(data: &[u8]) -> Option<(f64, u8, u8)> {
    if data.len() < 10 {
        return None;
    }
    let temp_raw = ((data[6] as i16) << 8) | (data[7] as i16);
    let temp = temp_raw as f64 / 10.0;
    let humi = data[8];
    let batt = data[9];
    Some((temp, humi, batt))
}
