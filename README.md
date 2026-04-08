# aht30

Better AHT10 / AHT20 / AHT30 / AHT40 humidity temperature sensor driver, for rust embedded-hal, optional async.

- Correct and sound: follow official datasheet and routine, no lies, already [tested on real hardware](#details).
- Flexable design: configurable I2C address, enable / disable checksum.
- Optional async: enable the `async` feature, can be used with [embassy](https://github.com/embassy-rs/embassy).
- Friendly for no-FPU platform: allow read raw `u32` or `u16` value.

## Example

### AHT10 async

```rust
use aht30::{AHT10_DEFAULT_ADDR, Aht10Async};
use embassy_time::{Delay, Timer};
use esp_hal::i2c::master::I2c; // or whatever any embedded-hal i2c, like ch32-hal, stm32f4xx-hal, and linux-embedded-hal
let i2c = I2c::new(peripherals.I2C0, Default::default())?
    .with_sda(peripherals.GPIO4)
    .with_scl(peripherals.GPIO5)
    .into_async();

let mut aht10 = Aht10Async::new(AHT10_DEFAULT_ADDR, i2c, Delay); 
// do calibrate after power-up is recommended but not forced
aht10.calibrate().await?;
// call .read() then .decode()
let (humidity, temperature) = aht10.read().await?.decode();
// example output: humidity = 67.25 %, temperature = 23.75 °C
info!("humidity = {} %, temperature = {} °C", humidity, temperature);
// can read raw value without decode formula calc
let humidity_raw = aht10.read().await?.humidity_raw; 
```

### AHT10 sync

```rust
use aht30::{AHT10_DEFAULT_ADDR, Aht10};
use esp_hal::delay::Delay;
use esp_hal::i2c::master::I2c;
let i2c = I2c::new(peripherals.I2C0, Default::default())?
    .with_sda(peripherals.GPIO4)
    .with_scl(peripherals.GPIO5);
let mut aht10 = Aht10::new(AHT10_DEFAULT_ADDR, i2c, Delay);
aht10.calibrate()?;
let (humidity, temperature) = aht10.read()?.decode();
```

### AHT20 / AHT25 / AHT30, async

For sync usage, see aht10 example. Just remove `Async` suffix in name.

```rust
use aht30::{AHT20_DEFAULT_ADDR, Aht20Async};
let mut aht20 = Aht20Async::new(AHT20_DEFAULT_ADDR, i2c, Delay);
// do calibrate may not be required for models manufactured after 2022, but still recommended
aht20.calibrate().await?;
// enable checksum is recommended
let (humidity, temperature) = aht20.read(true).await?.decode();
```

### AHT40, async

```rust
use aht30::{AHT40_DEFAULT_ADDR, Aht40Async}; // it has different i2c address
let mut aht40 = Aht40Async::new(AHT40_DEFAULT_ADDR, i2c, Delay);
// enable checksum is recommended, no calibrate function in aht40
let (humidity, temperature) = aht40.read(true).await?.decode();
```

## Details

<details>
<summary>Click to show</summary>

<img alt="screenshot-solution" loading="lazy" width="692" height="774" src="https://github.com/kkocdko/kblog/releases/download/simple_storage/hal-drv_aht30_1.webp">

</details>

## Contributors

[![Contributors](https://hub-io-mcells-projects.vercel.app/r/hal-drv/aht30)](https://github.com/hal-drv/aht30/graphs/contributors)

## License

[MIT License](./LICENSE).
