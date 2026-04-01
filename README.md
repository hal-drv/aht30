# aht30

Correct AHT10 / AHT20 / AHT30 / AHT40 humidity temperature sensor driver, on embedded-hal, optional async.

- Flexable design: configurable I2C address, enable / disable checksum.
- Optional async: enable the `async` feature, can be used with embassy.
- Allow read `u32` value for platforms without FPU.
- Example: ch32v003, esp32c3.
