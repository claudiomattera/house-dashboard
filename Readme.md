House Dashboard
====

An application to display various kinds of charts on a house dashboard.

<https://gitlab.com/claudiomattera/house-dashboard/>

![Infrastructure Chart](./docs/infrastructure.png)
![Proxmox Chart](./docs/proxmox.png)
![Trend Chart](./docs/trend.png)
![Geographical Heat-Map Chart](./docs/geographicalheatmap.png)
![Temporal Heat-Map Chart](./docs/temporalheatmap.png)

This application can be used to display room temperature, humidity and air quality; water and heating meters readings; weather forecast; and any other kind of data available.

It is designed to run on a [Raspberry Pi 0 W] equipped with a TFT 320×240 display and running [Raspbian Buster], but it should be possible to run it on most platforms.
The dashboard is implemented in [Rust] and it fetches data from an [InfluxDB]
database, which are displayed using [Plotters], a Rust crate for creating charts.

[Raspberry Pi 0 W]: https://www.raspberrypi.org/products/raspberry-pi-zero-w/
[Raspbian Buster]: https://www.raspberrypi.org/downloads/raspbian/
[Rust]: https://www.rust-lang.org/
[InfluxDB]: https://www.influxdata.com/products/influxdb-overview/
[Plotters]: https://crates.io/crates/plotters


Installation
----

Executables for Linux can be found in the [releases page](https://gitlab.com/claudiomattera/house-dashboard/-/releases).


### From source

This application can be compiled using the Rust toolchain.

~~~~shell
cargo build --release --all-features
~~~~

The resulting executable will be created in `target/release/house-dashboard`.


Usage
----

This is a command-line application that saves charts as BMP images.

The charts are defined in a configuration file which is passed to the application through a command-line argument.

~~~~plain
house-dashboard -v \
    --configuration-directory /path/to/configuration/ \
    --output-directory /path/to/generated/charts
~~~~

The `--help` argument describes the command-line interface in details.

~~~~plain
> house-dashboard --help
Create dashboard images

Usage: -v... -c PATH [-o PATH]

Available options:
    -v, --verbose                         Verbosity level
    -c, --configuration-directory <PATH>  Path to configuration directory
    -o, --output-directory <PATH>         Path to output directory (default: .)
    -h, --help                            Prints help information
~~~~


### Configuration

The configuration directory must contain the following [TOML] files:

- `influxdb.toml`: configures the address and credentials of the InfluxDB server.
- `style.toml`: configures the appearance of generated charts.
- `*.toml`: each of those files configures an individual chart.

[TOML]: https://github.com/toml-lang/toml


#### InfluxDB Configuration

File `influxdb.toml` must contain the following information:

~~~~toml
url = "https://influxdb.example.com:8086"
# cacert = "/path/to/custom/certification/authority/root.crt"
# dangerously_accept_invalid_certs = false
username = "some-user-name"
password = "some-password"
~~~~


#### Style Configuration

File `style.toml` must contain the following information:

~~~~toml
font_name = "FontName"
font_path = "FontName.ttf" # Relative to the configuration directory
font_scale = 1
system_palette = "Light"
series_palette = "ColorbrewerSet1"
resolution = [320, 240]
~~~~

Recommendation: font [Print Char 21] works quite well for small displays.

[Print Char 21]: https://www.kreativekorp.com/software/fonts/apple2/


#### Chart Configuration

Each of the other `.toml` files defines a specific chart.

Charts can be of several types:

* Infrastructure: Show the status and load of physical servers (measurement `system` in database `telegraf`, limited to hosts with tag `always-on` equal to `true`).
* Proxmox: Show the status and load of virtual machines and containers (measurement `proxmox` in database `telegraf`, limited to hosts with tag `node-fqdn` equal to the value specified).
* Trend: Show a line chart.
* Geographical heat-map: Show a geographical heat-map on regions defined in the configuration file.
* Temporal heat-map: Show a temporal heat-map of a measurement over time.


##### Infrastructure Chart

![Infrastructure Chart](./docs/infrastructure.png)

The configuration file must contain the following information:

~~~~toml
kind = "InfrastructureSummary"
title = "INFRASTRUCTURE"
how_long_ago = "PT30M"
suffix = ".example.com" # This is stripped from hostnames
last_update_format = "Updated %A at %H:%M"
vertical_step = 18
~~~~


##### Proxmox Chart

![Proxmox Chart](./docs/proxmox.png)

The configuration file must contain the following information:

~~~~toml
kind = "ProxmoxSummary"
title = "PROXMOX"
how_long_ago = "PT30M"
suffix = ".example.com" # This is stripped from hostnames
vertical_step = 8
node_fqdn = "proxmox.example.com"
~~~~


##### Trend Chart

![Trend Chart](./docs/trend.png)

The configuration file must contain the following information:

~~~~toml
kind = "Trend"
title = "Temperature"
ylabel = "Temperature"
yunit = "C"
database = "house"
measurement = "indoor_environment"
field = "temperature"
tag = "room" # Show one line per tag value
scale = 1
how_long_ago = "P1D"
how_often = "PT1H" # Resample to this period
xlabel_format = "%H:%M"
draw_last_value = true
hide_legend = false
max_x_ticks = 6
draw_horizontal_grid = true
~~~~


##### Geographical Heat-map Chart

![Geographical Heat-Map Chart](./docs/geographicalheatmap.png)

The configuration file must contain the following information:

~~~~toml
kind = "GeographicalHeatMap"
title = "TEMPERATURE"
unit = "C"
database = "house"
measurement = "indoor_environment"
field = "temperature"
tag = "room"
how_long_ago = "P1D"
bounds = [15, 35]
colormap = "CoolWarm"


[[regions]]
name = "living room"
coordinates = [
  [0.0, 0.0],
  [0.0, 1.0],
  [1.0, 1.0],
  [1.0, 0.0],
]

[[regions]]
name = "bedroom"
coordinates = [
  [1.0, 0.0],
  [1.0, 1.0],
  [2.0, 1.0],
  [2.0, 0.0],
]

[[regions]]
name = "kitchen"
coordinates = [
  [0.0, 1.0],
  [0.0, 2.0],
  [1.0, 2.0],
  [1.0, 1.0],
]

[[regions]]
name = "bathroom"
coordinates = [
  [1.0, 1.0],
  [1.0, 2.0],
  [2.0, 2.0],
  [2.0, 1.0],
]
~~~~


##### Temporal Heat-map Chart

![Temporal Heat-Map Chart](./docs/temporalheatmap.png)

The configuration file must contain the following information:

~~~~toml
kind = "TemporalHeatMap"
title = "OUTDOOR TEMPERATURE"
unit = "C"
database = "house"
measurement = "outdoor_environment"
field = "temperature"
aggregator = "mean"
tag = "address"
tag_value = "some address"
period = "HourOverDay"
# bounds = [-20, 0]
colormap = "CoolWarm"
~~~~


License
----

Copyright Claudio Mattera 2023

You are free to copy, modify, and distribute this application with attribution under the terms of the [MPL 2.0 license]. See the [`License.txt`](./License.txt) file for details.


[MPL 2.0 license]: https://opensource.org/licenses/MPL-2.0
