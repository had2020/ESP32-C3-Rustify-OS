#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::u16;

use alloc::fmt::format;
use alloc::string;
use esp_alloc::HeapStats;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::{AccessPointConfig, AuthMethod, ModeConfig, WifiMode};
//use esp_radio::ble::controller::BleConnector;
use crate::alloc::string::ToString;
use log::info;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config =
        esp_hal::Config::default().with_cpu_clock(/*CpuClock::max()*/ CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");

    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    //let _connector = BleConnector::new(&radio_init, peripherals.BT, Default::default());

    let access_point_config = AccessPointConfig::default()
        .with_ssid("SSH_ESP32-3C-RS-OS".to_string())
        .with_channel(10)
        .with_secondary_channel(0)
        .with_auth_method(AuthMethod::Wpa2Wpa3Personal)
        .with_password("RustIsFun".to_string());

    _wifi_controller
        .set_config(&ModeConfig::AccessPoint(access_point_config))
        .unwrap();

    _wifi_controller
        .set_mode(esp_radio::wifi::WifiMode::Ap)
        .unwrap();

    let _ = _wifi_controller.start().unwrap();
    info!(
        "Acess Point Online: {}",
        _wifi_controller.is_started().unwrap()
    );

    loop {
        let stats: HeapStats = esp_alloc::HEAP.stats();
        info!("Memory: {}", stats);
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_secs(10) {}
    }
}
