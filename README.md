# Scout

Scout is a realtime data api for distributed hardware.

We use scout at [Adventure Labs](https://adventurelabs.earth) for collecting and analyzing data from remote trail cameras. However, the tools are designed with first class support for a veriety of devices including smart buoys, gps trackers, drones, and more.

## Examples

- [Upload Image Directory](scout_rs/src/bin/upload_directory.rs)
- [Scout Provider](core/README.md#usage)

Note: Deep dive client integration guides coming soon.

## Tools

All tools are licensed under the GPL-3.0 license.

[core](core/README.md) provides convenient methods for interacting with scout from a NextJs application.

[scout_rs](scout_rs/README.md) provides a rust library for interacting with scout.
