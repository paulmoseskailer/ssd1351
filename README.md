# SSD1351 driver

Branch `sync_draw_async_flush` - blocking draw operations but async `DrawTarget` dependency to use async bus and async flush.
Useful for comparing to the same thing but with sync flush (branch `sync_draw_async_drawtarget`).

## Features
- [`embedded-graphics`](https://github.com/jamwaffles/embedded-graphics) support
- Full 16bit colour support for primitives and fonts and images
- Bufferless
- Rotation Support

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
