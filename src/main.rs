use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::task;
use tokio::time::{Duration, sleep};

pub mod config;
mod light;
mod sensor;
mod weather;

use sensor::SensorData;
use weather::WeatherCombined;

#[derive(Debug, Clone, Serialize)]
struct AggregatedData {
    time: String,
    sensor: Option<SensorData>,
    weather: Option<WeatherCombined>,
}

struct AppState {
    sensor: Arc<RwLock<Option<SensorData>>>,
    weather: Arc<RwLock<Option<WeatherCombined>>>,
}

#[derive(Deserialize)]
struct LightRequest {
    action: String,
}

#[get("/api/data")]
async fn get_data(data: web::Data<AppState>) -> impl Responder {
    let sensor = data.sensor.read().unwrap().clone();
    let now = chrono::Local::now().format("%H:%M").to_string();

    let cloned_weather = data.weather.read().unwrap().clone();
    let response = AggregatedData {
        time: now,
        sensor,
        weather: cloned_weather,
    };
    HttpResponse::Ok().json(response)
}

#[post("/api/light")]
async fn control_light(req: web::Json<LightRequest>) -> impl Responder {
    let result = match req.action.as_str() {
        "on" => light::turn_light_on_logic().await,
        "off" => light::turn_light_off_logic().await,
        _ => return HttpResponse::BadRequest().body("Invalid action"),
    };

    match result {
        Ok(_) => HttpResponse::Ok().body("Success"),
        Err(e) => {
            eprintln!("Light control error: {}", e);
            HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
    }
}

async fn update_weather_task(weather_storage: Arc<RwLock<Option<WeatherCombined>>>) {
    match weather::fetch_weather().await {
        Ok(data) => {
            println!(
                "Weather updated: Now {:?}, Hourly {}",
                data.now.is_some(),
                data.hourly.len()
            );
            let mut lock = weather_storage.write().unwrap();
            *lock = Some(data);
        }
        Err(e) => eprintln!("Failed to fetch weather: {}", e),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Shared state
    let sensor_data = Arc::new(RwLock::new(None));
    let weather_data = Arc::new(RwLock::new(None));

    // Spawn Sensor Monitor
    let sensor_clone = sensor_data.clone();
    task::spawn(async move {
        loop {
            // Try to fetch sensor data with a timeout (e.g., 60 seconds)
            println!("Starting scheduled sensor scan...");
            match tokio::time::timeout(
                Duration::from_secs(60),
                sensor::fetch_sensor_data_once(sensor_clone.clone()),
            )
            .await
            {
                Ok(Ok(_)) => println!("Sensor scan completed successfully."),
                Ok(Err(e)) => eprintln!("Sensor scan failed: {}", e),
                Err(_) => eprintln!("Sensor scan timed out."),
            }

            // Sleep for 15 minutes before next scan
            // (15 * 60 = 900 seconds)
            sleep(Duration::from_secs(900)).await;
        }
    });

    // Spawn Weather Updater
    let weather_clone = weather_data.clone();
    task::spawn(async move {
        // Run once as requested
        update_weather_task(weather_clone).await;
    });

    let app_state = web::Data::new(AppState {
        sensor: sensor_data,
        weather: weather_data,
    });

    println!("Starting server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(get_data)
            .service(control_light)
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
