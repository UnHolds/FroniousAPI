# Fronius API

This project offers the functionality to periodically poll the fornius API.
The polled datasets will then be sent to a InfluxDB database.
This project also offers the fornius API functions in a separate files called
`fronius.rs`. If you only want to use the API calls and implement the reporting
functions yourself, then feel free to copy this file (License applies).

This project has only been tested on a GEN24 with an Batterypack and an OhmPilot.

## Usage

### Docker-Hub

There also exists an offical Docker image on docker hub:
https://hub.docker.com/r/unhold/fronius-rust

If you want to use this image you just need to create the following
`docker-compose.yml` file and change the enviroment variables.

```yml
services:
  fronius:
    image: unhold/fronius-rust:latest
    environment:
      - FRONIUS_IP=10.0.0.1
      - INFLUX_DB_URL=http://10.0.0.2:8086
      - INFLUX_DB_ORG=<organisation>
      - INFLUX_DB_TOKEN=<token>
      - INFLUX_DB_BUCKET=<bucket>
```

After that you can run:

```
docker compose up
```

### docker-compose:

To use this project you need to create a enviromennt file called '.env'. This
file needs to contain the following values.

```
FRONIUS_IP=10.0.0.1
INFLUX_DB_URL=http://10.0.0.2:8086
INFLUX_DB_ORG=<organisation>
INFLUX_DB_TOKEN=<secret>
INFLUX_DB_BUCKET=<bucket>
```

After that you can run:

```
docker compose up
```

When the project is started you will perdiodically (every 15sec) see log messages
that indicate that data was reported.

### Build & Run

If you just want to build and run it, then make sure that the following
enviroment variables are set:

```
FRONIUS_IP=10.0.0.1
INFLUX_DB_URL=http://10.0.0.2:8086
INFLUX_DB_ORG=<organisation>
INFLUX_DB_TOKEN=<secret>
INFLUX_DB_BUCKET=<bucket>
```

## fronius.rs

### Supported API calls

| API path                                    | library function                                                              |
| ------------------------------------------- | ----------------------------------------------------------------------------- |
| /solar_api/v1/GetInverterRealtimeData.cgi   | `get_inverter_realtime_data_system()` `get_inverter_realtime_data_device()`   |
| /solar_api/v1/GetInverterInfo.cgi           | `get_inverter_info()`                                                         |
| /solar_api/v1/GetActiveDeviceInfo.cgi       | `get_active_device_info()`                                                    |
| /solar_api/v1/GetMeterRealtimeData.cgi      | `get_meter_realtime_data_system()` `get_meter_realtime_data_device()`         |
| /solar_api/v1/GetStorageRealtimeData.cgi    | `get_storage_realtime_data_system()` `get_storage_realtime_data_device()`     |
| /solar_api/v1/GetOhmPilotRealtimeData.cgi   | `get_ohm_pilot_realtime_data_system()` `get_ohm_pilot_realtime_data_device()` |
| /solar_api/v1/GetPowerFlowRealtimeData.fcgi | `get_power_flow_realtime_data()`                                              |

### Example usage

```rs
    let ip = IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1));
    let fronius = Fronius::connect(ip)?;
    println!(
        "{:#?}",
        fronius.get_inverter_realtime_data_system()?
    );
```
