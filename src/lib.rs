#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_code)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]

use embedded_hal::{delay::DelayNs, i2c::I2c};
#[cfg(feature = "async")]
use embedded_hal_async::{delay::DelayNs as DelayNsAsync, i2c::I2c as I2cAsync};

fn crc8(data: &[u8]) -> u8 {
    let polynomial = 0x31u8; // x^8 + x^5 + x^4 + 1
    let mut crc = 0xFFu8;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ polynomial;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Possible errors when interacting with the sensor.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SensorError {
    Io,
    Timeout,
    Checksum,
}

impl core::fmt::Display for SensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            SensorError::Io => "io failed",
            SensorError::Timeout => "operation timeout",
            SensorError::Checksum => "checksum mismatch",
        })
    }
}

impl core::error::Error for SensorError {}

impl<E: embedded_hal::i2c::Error> From<E> for SensorError {
    fn from(_value: E) -> Self {
        SensorError::Io
    }
}

/// AHT10 sensor.
#[maybe_async_cfg::maybe(
    idents(
        Aht10Async(sync = "Aht10", async),
        I2cAsync(sync = "I2c", async),
        DelayNsAsync(sync = "DelayNs", async)
    ),
    sync(all()),
    async(feature = "async")
)]
pub struct Aht10Async<I: I2cAsync, D: DelayNsAsync> {
    addr: u8,
    i2c: I,
    delay: D,
}
#[maybe_async_cfg::maybe(
    idents(
        Aht10Async(sync = "Aht10", async),
        I2cAsync(sync = "I2c", async),
        DelayNsAsync(sync = "DelayNs", async)
    ),
    sync(all()),
    async(feature = "async")
)]
impl<I: I2cAsync, D: DelayNsAsync> Aht10Async<I, D> {
    /// Requires >= 40ms after power-up, this function includes no delay, callers must ensure this timing.
    pub fn new(addr: u8, i2c: I, delay: D) -> Self {
        Self { addr, i2c, delay }
    }

    /// Calibration is only necessary immediately after power-up. Not required during normal data acquisition.
    pub async fn calibrate(&mut self) -> Result<(), SensorError> {
        let mut status: [u8; 1] = [0; 1];
        'done: {
            for _ in 0..5 {
                self.i2c.read(self.addr, &mut status).await?; // read only 8bit status
                if status[0] & 0b1000 == 0b1000 {
                    break 'done;
                }
                self.i2c.write(self.addr, &[0xE1, 0x08, 0x00]).await?;
                self.delay.delay_ms(10).await;
            }
            return Err(SensorError::Timeout);
        }
        Ok(())
    }

    /// Reset sensor, without removing the power supply, takes <= 20ms to be done, this function includes no delay, callers must ensure this timing.
    pub async fn soft_reset(&mut self) -> Result<(), SensorError> {
        self.i2c.write(self.addr, &[0xBA]).await?;
        Ok(())
    }

    /// Measure then read sensor, takes >= 80ms to be done. Unlike AHT20, there's no checksum.
    pub async fn read(&mut self) -> Result<Aht10Measurement, SensorError> {
        self.i2c.write(self.addr, &[0xAC, 0x33, 0x00]).await?;
        self.delay.delay_ms(80).await;
        let mut response: [u8; 6] = [0; 6]; // 48bits = status(8bit) + humidity(20bits) + temperature(20bits)
        'done: {
            for _ in 0..100 {
                self.i2c.read(self.addr, &mut response[..1]).await?;
                if response[0] & 0b1000_0000 == 0 {
                    break 'done;
                }
                self.delay.delay_ms(5).await;
            }
            return Err(SensorError::Timeout);
        }
        self.i2c.read(self.addr, &mut response).await?;
        let humidity_raw =
            (response[1] as u32) << 12 | (response[2] as u32) << 4 | (response[3] as u32) >> 4;
        let temperature_raw =
            ((response[3] & 0x0f) as u32) << 16 | (response[4] as u32) << 8 | response[5] as u32;
        Ok(Aht10Measurement {
            humidity_raw,
            temperature_raw,
        })
    }
}
pub struct Aht10Measurement {
    pub humidity_raw: u32, // keep u32, decode is optional, some MCU without FPU will benefit from this
    pub temperature_raw: u32,
}
impl Aht10Measurement {
    /// Decode to `(humidity, temperature)`. For example, humidity = 67.25 (percent), temperature = 23.75 (celsius).
    pub fn decode(&self) -> (f32, f32) {
        let humidity = self.humidity_raw as f32 / ((1 << 20) as f32) * 100f32;
        let temperature = self.temperature_raw as f32 / ((1 << 20) as f32) * 200f32 - 50f32;
        (humidity, temperature)
    }
}
pub const AHT10_DEFAULT_ADDR: u8 = 0x38;

/// AHT20 / AHT25 / AHT30 sensor.
#[maybe_async_cfg::maybe(
    idents(
        Aht20Async(sync = "Aht20", async),
        I2cAsync(sync = "I2c", async),
        DelayNsAsync(sync = "DelayNs", async)
    ),
    sync(all()),
    async(feature = "async")
)]
pub struct Aht20Async<I: I2cAsync, D: DelayNsAsync> {
    addr: u8,
    i2c: I,
    delay: D,
}
#[maybe_async_cfg::maybe(
    idents(
        Aht20Async(sync = "Aht20", async),
        I2cAsync(sync = "I2c", async),
        DelayNsAsync(sync = "DelayNs", async)
    ),
    sync(all()),
    async(feature = "async")
)]
impl<I: I2cAsync, D: DelayNsAsync> Aht20Async<I, D> {
    /// Requires >= 100ms after power-up, this function includes no delay, callers must ensure this timing.
    pub fn new(addr: u8, i2c: I, delay: D) -> Self {
        Self { addr, i2c, delay }
    }

    /// Calibration is only necessary immediately after power-up. Not required during normal data acquisition. May not be required for models manufactured after 2022, but still recommended.
    pub async fn calibrate(&mut self) -> Result<(), SensorError> {
        let mut status: [u8; 1] = [0; 1];
        'done: {
            for _ in 0..5 {
                self.i2c.read(self.addr, &mut status).await?; // read only 8bit status
                if status[0] & 0x18 == 0x18 {
                    break 'done;
                }
                // self.i2c.write(self.addr, &[0xBE, 0x08, 0x00]).await?; // in newer version of docs, this was replaced by below, https://github.com/adafruit/Adafruit_CircuitPython_AHTx0/issues/17
                self.i2c.write(self.addr, &[0x1B, 0, 0]).await?;
                self.i2c.write(self.addr, &[0x1C, 0, 0]).await?;
                self.i2c.write(self.addr, &[0x1E, 0, 0]).await?;
                self.delay.delay_ms(10).await;
            }
            return Err(SensorError::Timeout);
        }
        Ok(())
    }

    /// Reset sensor, without removing the power supply, takes <= 20ms to be done, this function includes no delay, callers must ensure this timing.
    pub async fn soft_reset(&mut self) -> Result<(), SensorError> {
        self.i2c.write(self.addr, &[0xBA]).await?;
        Ok(())
    }

    /// Measure then read sensor, takes >= 80ms to be done. Enable checksum is recommended.
    pub async fn read(&mut self, checksum: bool) -> Result<Aht20Measurement, SensorError> {
        self.i2c.write(self.addr, &[0xAC, 0x33, 0x00]).await?; // measurement command, some crates like [this](https://docs.rs/crate/embedded-dht-rs/0.5.0/source/src/dht20.rs#17-29) is incorrect, 0x71 is the "i2c read" for address 0x38 and does not need to be sent manually
        self.delay.delay_ms(80).await;
        let mut response: [u8; 7] = [0; 7]; // 56bits = status(8bit) + humidity(20bits) + temperature(20bits) + crc(8bits)
        'done: {
            for _ in 0..100 {
                self.i2c.read(self.addr, &mut response[..1]).await?; // read only 8bit status
                if response[0] & 0b1000_0000 == 0 {
                    break 'done;
                }
                self.delay.delay_ms(5).await; // official example use 1ms, but it cause hang in some old model
            }
            return Err(SensorError::Timeout);
        }
        self.i2c.read(self.addr, &mut response).await?; // read whole response
        if checksum {
            // compare the calculated crc with the received crc
            if response[6] != crc8(&response[..6]) {
                return Err(SensorError::Checksum);
            }
        }
        // humidity 20 bits (8 + 8 + 4), temperature 20 bits (4 + 8 + 8)
        let humidity_raw =
            (response[1] as u32) << 12 | (response[2] as u32) << 4 | (response[3] as u32) >> 4;
        let temperature_raw =
            ((response[3] & 0x0f) as u32) << 16 | (response[4] as u32) << 8 | response[5] as u32;
        Ok(Aht20Measurement {
            humidity_raw,
            temperature_raw,
        })
    }
}
pub use Aht10Measurement as Aht20Measurement; // aht20 and aht10 use same decode formula
pub const AHT20_DEFAULT_ADDR: u8 = 0x38;

/// AHT40 sensor.
#[maybe_async_cfg::maybe(
    idents(
        Aht40Async(sync = "Aht40", async),
        I2cAsync(sync = "I2c", async),
        DelayNsAsync(sync = "DelayNs", async)
    ),
    sync(all()),
    async(feature = "async")
)]
pub struct Aht40Async<I: I2cAsync, D: DelayNsAsync> {
    addr: u8,
    i2c: I,
    delay: D,
}
#[maybe_async_cfg::maybe(
    idents(
        Aht40Async(sync = "Aht40", async),
        I2cAsync(sync = "I2c", async),
        DelayNsAsync(sync = "DelayNs", async)
    ),
    sync(all()),
    async(feature = "async")
)]
impl<I: I2cAsync, D: DelayNsAsync> Aht40Async<I, D> {
    /// Requires >= 5ms (datasheet says this, but I recommend 100ms like AHT30) after power-up, this function includes no delay, callers must ensure this timing.
    pub fn new(addr: u8, i2c: I, delay: D) -> Self {
        Self { addr, i2c, delay }
    }

    /// Measure then read sensor, takes >= 80ms to be done. Enable checksum is recommended.
    pub async fn read(&mut self, checksum: bool) -> Result<Aht40Measurement, SensorError> {
        self.i2c.write(self.addr, &[0xFD]).await?; // measurement command, [this implement](https://github.com/libdriver/aht40/blob/84b7d3a74eff772cf9f61ff8bb189005f97ca6dd/src/driver_aht40.c#L319) and zh-cn datasheet says it should be single 0xFD, but [this datasheet](https://www.scribd.com/document/889704953/AHT40-datasheet-May-2024) says it should be [0xFD, 0xF6, 0xE0, 0x39, 0x32, 0x2F, 0x24, 0x1E, 0x15]
        self.delay.delay_ms(80).await;
        let mut response: [u8; 6] = [0; 6]; // 48bits = temperature(16bits) + crc(8bits) + humidity(16bits) + crc(8bits)
        self.i2c.read(self.addr, &mut response).await?; // read whole response, aht40 has no status to read
        if checksum {
            // compare the calculated crc with the received crc
            if response[2] != crc8(&response[0..2]) || response[5] != crc8(&response[3..5]) {
                return Err(SensorError::Checksum);
            }
        }
        let humidity_raw = (response[3] as u16) << 8 | (response[4] as u16);
        let temperature_raw = (response[0] as u16) << 8 | (response[1] as u16);
        Ok(Aht40Measurement {
            humidity_raw,
            temperature_raw,
        })
    }
}
pub struct Aht40Measurement {
    pub humidity_raw: u16, // keep u16, decode is optional, some MCU without FPU will benefit from this
    pub temperature_raw: u16,
}
impl Aht40Measurement {
    /// Decode to `(humidity, temperature)`. For example, humidity = 67.25 (percent), temperature = 23.75 (celsius).
    pub fn decode(&self) -> (f32, f32) {
        let humidity = self.humidity_raw as f32 / 65535f32 * 125f32 - 6f32;
        let temperature = self.temperature_raw as f32 / 65535f32 * 175f32 - 45f32;
        (humidity, temperature)
    }
}
pub const AHT40_DEFAULT_ADDR: u8 = 0x44;

/// - [AHT10.cpp](https://github.com/Thinary/AHT10/raw/refs/heads/master/src/Thinary_AHT10.cpp)
/// - [AHT20_dvarrel.cpp](https://github.com/dvarrel/AHT20/raw/refs/heads/main/src/AHT20.cpp)
/// - [AHT20_sparkfun.cpp](https://github.com/sparkfun/SparkFun_Qwiic_Humidity_AHT20_Arduino_Library/raw/refs/heads/main/src/SparkFun_Qwiic_Humidity_AHT20.cpp)
/// - 官网提供的“说明书”是残缺的，缺少很多东西。“规格书(产品手册)”才是完整版。
/// - [AHT10产品手册_A3_20201210.pdf](https://www.aosong.com/userfiles/files/media/AHT10%E4%BA%A7%E5%93%81%E6%89%8B%E5%86%8C%20A3%2020201210.pdf)
/// - [AHT20_Data_Sheel_english_2021.pdf](https://www.aosong.com/userfiles/files/media/Data%20Sheet%20AHT20.pdf)
/// - [AHT20产品规格书(中文版)A5.pdf](https://www.aosong.com/userfiles/files/media/AHT20%E4%BA%A7%E5%93%81%E8%A7%84%E6%A0%BC%E4%B9%A6(%E4%B8%AD%E6%96%87%E7%89%88)%20A5.pdf)
/// - [DHT20产品规格书(中文版)A3-202409.pdf](https://www.aosong.com/uploadfiles/2025/02/20250228161634702.pdf)
/// - [AHT25产品规格书中文版A3.pdf](https://www.aosong.com/uploadfiles/2025/07/20250724093156529.pdf)
/// - [AHT20温湿度传感器说明书中文版C0-202407.pdf](https://www.aosong.com/uploadfiles/2025/05/20250526104654866.pdf)
/// - [AHT20温湿度传感器说明书中文版C0-202407.pdf](https://www.aosong.com/uploadfiles/2025/05/20250526104654866.pdf)
/// - [AHT30温湿度传感器说明书(中)A4-202505.pdf](https://www.aosong.com/uploadfiles/2025/05/20250528154400146.pdf)
/// - [AHT40温湿度传感器说明书A2-202506.pdf](https://www.aosong.com/uploadfiles/2025/09/20250901145507216.pdf)
/// - [AHT20-F温湿度传感器说明书中文版A3-202407.pdf](https://www.aosong.com/uploadfiles/2025/04/20250425154457582.pdf)
/// - [DHT11温湿度传感器说明书(中)A0-1208.pdf](https://www.aosong.com/uploadfiles/2025/03/20250305165717988.pdf)
/// - [AHT2系列温湿度传感器IIC例程.zip](https://www.aosong.com/uploadfiles/2025/02/20250211143151669.zip)
/// - [PCB-AHT20](https://oshwhub.com/xh_timmo/aht20-wen-shi-du-chuan-gan-qi)
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let result = crc8(&[0x76, 0x54, 0x32, 0x10]); // we test on real hardware, not using mock
        assert_eq!(result, 0x21);
    }
}
