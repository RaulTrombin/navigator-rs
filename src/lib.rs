#![deny(unsafe_code)]
#![doc(html_logo_url = "https://upload.wikimedia.org/wikipedia/commons/1/12/Bluerobotics-logo.svg")]
#![doc = include_str!("../README.md")]

use std::fmt;

mod ads1115;
mod ak09915;
mod bmp280;
mod bmp390;
mod icm20689;
mod leak;
mod led;
mod pca9685;
mod peripherals;
mod rgb;

use peripherals::*;

use crate::bmp280::Bmp280Device;
use crate::icm20689::Icm20689Device;
use ads1115::Ads1115Device;
use ak09915::Ak09915Device;
use bmp390::Bmp390Device;
use leak::LeakDetector;
use led::LedController;
use pca9685::Pca9685Device;
use rgb::RgbController;

// add docs ( difference btwen boards)
#[derive(Debug, Clone, Copy)]
pub enum BoardVersion {
    V1,
    V2,
}

// add docs ( default for now or V2 directly for 2025?)
impl Default for BoardVersion {
    fn default() -> Self {
        BoardVersion::V1
    }
}

// add docs ( explicit difference btwen overlays)
#[derive(Debug, Clone, Copy)]
pub enum PiVersion {
    Pi4,
    Pi5,
}

// add docs ( default for now or pi5 for 2025?)
impl Default for PiVersion {
    fn default() -> Self {
        PiVersion::Pi4
    }
}

pub struct NavigatorBuilder {
    rgb_led_strip_size: usize,
    board_version: BoardVersion,
    pi_version: PiVersion,
}

impl Default for NavigatorBuilder {
    fn default() -> Self {
        Self {
            rgb_led_strip_size: 1,
            board_version: BoardVersion::default(),
            pi_version: PiVersion::default(),
        }
    }
}

/// Set of available options to select ADC's channel.
#[derive(Debug, Clone, Copy)]
pub enum AdcChannel {
    Ch0,
    Ch1,
    Ch2,
    Ch3,
}

/// Set of options to control navigator's LEDs.
#[derive(Debug, Clone, Copy)]
pub enum UserLed {
    /// Attached to green LED through GPIO 24, labelled LED_1.
    Led1,
    /// Attached to blue LED through GPIO 25, labelled LED_2.
    Led2,
    /// Attached to red LED through GPIO 11, labelled LED_3.
    Led3,
}

impl fmt::Display for UserLed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserLed::Led1 => write!(f, "LED_1"),
            UserLed::Led2 => write!(f, "LED_2"),
            UserLed::Led3 => write!(f, "LED_3"),
        }
    }
}

/// The `AxisData` struct encapsulates values for the x, y, and z axes.
#[derive(Debug, Default, Clone)]
pub struct AxisData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Reads all sensors and stores on a single structure.
#[derive(Debug, Default, Clone)]
pub struct SensorData {
    pub adc: Vec<f32>,
    pub temperature: f32,
    pub pressure: f32,
    pub accelerometer: AxisData,
    pub magnetometer: AxisData,
    pub gyro: AxisData,
    pub leak: bool,
}

/// The `Navigator` struct contains various components used for navigator. It includes PWM control,
/// pressure and temperature sensing, analog-to-digital conversion, inertial measurement unit,
/// magnetometer, and LEDs control.
///
/// All these components are integrated and abstracted to be used directly from the navigator module.
///
/// Please check [`Implementations`](struct.Navigator.html#implementations) and its examples, then start coding your applications!
pub struct Navigator {
    devices: Vec<Box<dyn AnyHardware>>,
}

impl Navigator {
    pub fn new() -> Navigator {
        Self::default()
    }

    pub fn create() -> NavigatorBuilder {
        NavigatorBuilder::new()
    }

    // Helper methods to get devices
    fn get_adc_sensor(&mut self) -> Option<&mut dyn AdcSensor> {
        for device in &mut self.devices {
            if let Some(adc_sensor) = device.as_adc_sensor() {
                return Some(adc_sensor);
            }
        }
        None
    }

    fn get_temperature_sensor(&mut self) -> Option<&mut dyn TemperatureSensor> {
        for device in &mut self.devices {
            if let Some(temp_sensor) = device.as_temperature_sensor() {
                return Some(temp_sensor);
            }
        }
        None
    }

    fn get_barometer_sensor(&mut self) -> Option<&mut dyn BarometerSensor> {
        for device in &mut self.devices {
            if let Some(baro_sensor) = device.as_barometer_sensor() {
                return Some(baro_sensor);
            }
        }
        None
    }

    fn get_magnetometer_sensor(&mut self) -> Option<&mut dyn MagnetometerSensor> {
        for device in &mut self.devices {
            if let Some(mag_sensor) = device.as_magnetometer_sensor() {
                return Some(mag_sensor);
            }
        }
        None
    }

    fn get_accelerometer_sensor(&mut self) -> Option<&mut dyn AccelerometerSensor> {
        for device in &mut self.devices {
            if let Some(accel_sensor) = device.as_accelerometer_sensor() {
                return Some(accel_sensor);
            }
        }
        None
    }

    fn get_gyroscope_sensor(&mut self) -> Option<&mut dyn GyroscopeSensor> {
        for device in &mut self.devices {
            if let Some(gyro_sensor) = device.as_gyroscope_sensor() {
                return Some(gyro_sensor);
            }
        }
        None
    }

    fn get_leak_sensor(&mut self) -> Option<&mut dyn LeakSensor> {
        for device in &mut self.devices {
            if let Some(leak_sensor) = device.as_leak_sensor() {
                return Some(leak_sensor);
            }
        }
        None
    }

    fn get_led_behaviour(&mut self) -> Option<&mut dyn LedBehaviour> {
        for device in &mut self.devices {
            if let Some(led_behaviour) = device.as_led_behaviour() {
                return Some(led_behaviour);
            }
        }
        None
    }

    fn get_pwm_behaviour(&mut self) -> Option<&mut dyn PwmBehaviour> {
        for device in &mut self.devices {
            if let Some(pwm_behaviour) = device.as_pwm_behaviour() {
                return Some(pwm_behaviour);
            }
        }
        None
    }

    fn get_rgb_behaviour(&mut self) -> Option<&mut dyn RgbBehaviour> {
        for device in &mut self.devices {
            if let Some(rgb_behaviour) = device.as_rgb_behaviour() {
                return Some(rgb_behaviour);
            }
        }
        None
    }

    // Implement methods to interact with devices
    pub fn read_temperature(&mut self) -> f32 {
        if let Some(temp_sensor) = self.get_temperature_sensor() {
            temp_sensor.read_temperature().unwrap()
        } else {
            panic!("No temperature sensor available");
        }
    }

    pub fn read_pressure(&mut self) -> f32 {
        if let Some(baro_sensor) = self.get_barometer_sensor() {
            baro_sensor.read_pressure().unwrap()
        } else {
            panic!("No barometer sensor available");
        }
    }

    pub fn read_mag(&mut self) -> AxisData {
        if let Some(mag_sensor) = self.get_magnetometer_sensor() {
            let (x, y, z) = mag_sensor.read_magnetic_field().unwrap();
            // Adjust axes if necessary
            AxisData { x, y, z }
        } else {
            panic!("No magnetometer sensor available");
        }
    }

    pub fn read_accel(&mut self) -> AxisData {
        if let Some(accel_sensor) = self.get_accelerometer_sensor() {
            let (x, y, z) = accel_sensor.read_acceleration().unwrap();
            AxisData { x, y, z }
        } else {
            panic!("No accelerometer sensor available");
        }
    }

    pub fn read_gyro(&mut self) -> AxisData {
        if let Some(gyro_sensor) = self.get_gyroscope_sensor() {
            let (x, y, z) = gyro_sensor.read_angular_velocity().unwrap();
            AxisData { x, y, z }
        } else {
            panic!("No gyroscope sensor available");
        }
    }

    pub fn read_leak(&mut self) -> bool {
        if let Some(leak_sensor) = self.get_leak_sensor() {
            leak_sensor.is_leak_detected().unwrap()
        } else {
            panic!("No leak sensor available");
        }
    }

    pub fn set_led(&mut self, select: UserLed, state: bool) {
        let index = match select {
            UserLed::Led1 => 1,
            UserLed::Led2 => 2,
            UserLed::Led3 => 0,
        };
        if let Some(led_behaviour) = self.get_led_behaviour() {
            led_behaviour.set_led(index, state);
        } else {
            panic!("No LED controller available");
        }
    }

    pub fn set_led_toggle(&mut self, select: UserLed) {
        let index = match select {
            UserLed::Led1 => 1,
            UserLed::Led2 => 2,
            UserLed::Led3 => 0,
        };
        let state = self.get_led(select);
        if let Some(led_behaviour) = self.get_led_behaviour() {
            led_behaviour.set_led(index, !state);
        } else {
            panic!("No LED controller available");
        }
    }

    pub fn get_led(&mut self, select: UserLed) -> bool {
        let index = match select {
            UserLed::Led1 => 1,
            UserLed::Led2 => 2,
            UserLed::Led3 => 0,
        };
        if let Some(led_behaviour) = self.get_led_behaviour() {
            led_behaviour.get_led(index)
        } else {
            panic!("No LED controller available");
        }
    }

    pub fn set_pwm_enable(&mut self, enable: bool) {
        if let Some(pwm_behaviour) = self.get_pwm_behaviour() {
            pwm_behaviour.enable_output(enable).unwrap();
        } else {
            panic!("No PWM controller available");
        }
    }

    pub fn set_pwm_frequency(&mut self, freq_hz: f32) {
        if let Some(pwm_behaviour) = self.get_pwm_behaviour() {
            pwm_behaviour.set_frequency(freq_hz).unwrap();
        } else {
            panic!("No PWM controller available");
        }
    }

    pub fn set_pwm_duty_cycle(&mut self, channel: usize, duty_cycle: f32) {
        if let Some(pwm_behaviour) = self.get_pwm_behaviour() {
            pwm_behaviour.set_duty_cycle(channel, duty_cycle).unwrap();
        } else {
            panic!("No PWM controller available");
        }
    }

    pub fn set_duty_cycle_all(&mut self, duty_cycle: f32) {
        if let Some(pwm_behaviour) = self.get_pwm_behaviour() {
            pwm_behaviour.set_duty_cycle_all(duty_cycle).unwrap();
        } else {
            panic!("No PWM controller available");
        }
    }

    pub fn read_adc(&mut self, channel: AdcChannel) -> f32 {
        let index = match channel {
            AdcChannel::Ch0 => 0,
            AdcChannel::Ch1 => 1,
            AdcChannel::Ch2 => 2,
            AdcChannel::Ch3 => 3,
        };
        if let Some(adc_sensor) = self.get_adc_sensor() {
            adc_sensor.read_channel(index).unwrap()
        } else {
            panic!("No ADC sensor available");
        }
    }

    pub fn read_adc_all(&mut self) -> Vec<f32> {
        if let Some(adc_sensor) = self.get_adc_sensor() {
            adc_sensor.read_all_channels().unwrap()
        } else {
            panic!("No ADC sensor available");
        }
    }

    pub fn set_neopixel(&mut self, colors: &[[u8; 3]]) {
        if let Some(rgb_behaviour) = self.get_rgb_behaviour() {
            rgb_behaviour.set_colors(colors).unwrap();
        } else {
            panic!("No RGB controller available");
        }
    }

    pub fn set_neopixel_rgbw(&mut self, colors: &[[u8; 4]]) {
        self.get_rgb_behaviour()
            .expect("No RGB controller available")
            .set_colors_rgbw(colors)
            .expect("Failed to set RGB colors");
    }

    pub fn read_all(&mut self) -> SensorData {
        SensorData {
            adc: self.read_adc_all(),
            temperature: self.read_temperature(),
            pressure: self.read_pressure(),
            accelerometer: self.read_accel(),
            magnetometer: self.read_mag(),
            gyro: self.read_gyro(),
            leak: self.read_leak(),
        }
    }
}

impl Default for Navigator {
    fn default() -> Self {
        Self::create().build()
    }
}

impl NavigatorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rgb_led_strip_size(mut self, size: usize) -> Self {
        self.rgb_led_strip_size = size;
        self
    }

    pub fn with_board_version(mut self, version: BoardVersion) -> Self {
        self.board_version = version;
        self
    }

    pub fn with_pi_version(mut self, version: PiVersion) -> Self {
        self.pi_version = version;
        self
    }

    fn build_common_devices(&self, i2c_bus: Option<&str>, gpiochip: Option<&str>) -> Vec<Box<dyn AnyHardware>> {
        let mut devices: Vec<Box<dyn AnyHardware>> = vec![
            Box::new(
                Ads1115Device::builder()
                    .build()
                    .expect("Failed to create Ads1115"),
            ),
            Box::new(
                Ak09915Device::builder()
                    .build()
                    .expect("Failed to create Ak09915"),
            ),
            Box::new(
                Icm20689Device::builder()
                    .build()
                    .expect("Failed to create Icm20689"),
            ),
        ];

        // Navigator-specific setup ( BoardVersion enum <- link to ddocs)
        let pressure_sensor: Box<dyn AnyHardware> = match self.board_version {
            BoardVersion::V1 => Box::new(
                Bmp280Device::builder()
                    .build()
                    .expect("Failed to create Bmp280"),
            ),
            BoardVersion::V2 => Box::new(
                Bmp390Device::builder()
                    .build()
                    .expect("Failed to create Bmp390"),
            ),
        };
        devices.push(pressure_sensor);

        // Platform-specific setup ( PiVersion enum <- link to ddocs)
        let mut leak_detector = LeakDetector::builder();
        let mut led_controller = LedController::builder();
        let mut pca9685 = Pca9685Device::builder();

        if let Some(chip) = gpiochip {
            leak_detector = leak_detector.with_gpiochip(chip);
            led_controller = led_controller.with_gpiochip(chip);
            pca9685 = pca9685.with_gpiochip(chip);
        }

        if let Some(bus) = i2c_bus {
            pca9685 = pca9685.with_i2c_bus(bus);
        }

        devices.push(Box::new(leak_detector.build().expect("Failed to create LeakDetector")));
        devices.push(Box::new(led_controller.build()));
        devices.push(Box::new(pca9685.build().expect("Failed to create Pca9685")));

        let rgb_device = RgbController::builder()
            .with_led_count(self.rgb_led_strip_size)
            .build()
            .expect("Failed to create RgbController");
        devices.push(Box::new(rgb_device));

        devices
    }

    pub fn build(self) -> Navigator {
        match self.pi_version {
            PiVersion::Pi4 => {
                let devices = self.build_common_devices(None, None);
                Navigator { devices }
            }
            PiVersion::Pi5 => {
                let devices = self.build_common_devices(Some("/dev/i2c-3"), Some("/dev/gpiochip4"));
                Navigator { devices }
            }
        }
    }
}