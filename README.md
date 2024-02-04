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
(In this case you **don't** need to download or clone this repository)

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

### Local Build & Run

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

## InfluxDB data

The following datasets are transmitted every 15sec:

### InverterData

Endpoint: `/solar_api/v1/GetInverterRealtimeData.cgi`<br/>
DataCollection: `CommonInverterData` <br/>
InfluxDB Measurement: `inverter`

| Name         | Value (Fronius) | Type      |
| ------------ | --------------- | --------- |
| device       | "Inverter"      | Tag       |
| ac_power     | PAC             | Value     |
| ac_power_abs | SAC             | Value     |
| ac_current   | IAC             | Value     |
| ac_voltage   | UAC             | Value     |
| ac_frequency | FAC             | Value     |
| dc_current   | IDC             | Value     |
| dc_voltage   | UDC             | Value     |
| total_energy | TOTAL_ENERGY    | Value     |
| time         | "current_time"  | Timestamp |

### InverterPhaseData

Endpoint: `/solar_api/v1/GetInverterRealtimeData.cgi`<br/>
DataCollection: `3PInverterData` <br/>
InfluxDB Measurement: `inverter_phase`

| Name          | Value (Fronius) | Type      |
| ------------- | --------------- | --------- |
| device        | "Inverter"      | Tag       |
| ac_l1_current | IAC_L1          | Value     |
| ac_l2_current | IAC_L2          | Value     |
| ac_l3_current | IAC_L3          | Value     |
| dc_l1_voltage | UAC_L1          | Value     |
| dc_l2_voltage | UAC_L2          | Value     |
| dc_l3_voltage | UAC_L3          | Value     |
| time          | "current_time"  | Timestamp |

### InverterInfo

Endpoint: `/solar_api/v1/GetInverterInfo.cgi` <br/>
InfluxDB Measurement: `inverter_info`

| Name          | Value (Fronius) | Type      |
| ------------- | --------------- | --------- |
| device        | "Inverter"      | Tag       |
| device_type   | DT              | Value     |
| pv_power      | PVPower         | Value     |
| name          | CustomName      | Value     |
| is_visualized | Show            | Value     |
| id            | UniqueID        | Value     |
| error_code    | error_code      | Value     |
| status_code   | status_code     | Value     |
| state         | inverter_state  | Value     |
| time          | "current_time"  | Timestamp |

### MeterData

Endpoint: `/solar_api/v1/GetMeterRealtimeData.cgi` <br/>
InfluxDB Measurement: `meter`

| Name              | Value (Fronius)            | Type      |
| ----------------- | -------------------------- | --------- |
| device            | "Meter"                    | Tag       |
| l1_current        | Current_AC_Phase_1         | Value     |
| l2_current        | Current_AC_Phase_2         | Value     |
| l3_current        | Current_AC_Phase_3         | Value     |
| current           | Current_AC_Sum             | Value     |
| l1_voltage        | Voltage_AC_Phase_1         | Value     |
| l2_voltage        | Voltage_AC_Phase_2         | Value     |
| l3_voltage        | Voltage_AC_Phase_2         | Value     |
| l12_voltage       | Voltage_AC_PhaseToPhase_12 | Value     |
| l23_voltage       | Voltage_AC_PhaseToPhase_23 | Value     |
| l31_voltage       | Voltage_AC_PhaseToPhase_31 | Value     |
| l1_power          | PowerReal_P_Phase_1        | Value     |
| l2_power          | PowerReal_P_Phase_2        | Value     |
| l3_power          | PowerReal_P_Phase_3        | Value     |
| power             | PowerReal_P_Sum            | Value     |
| frequency_average | Frequency_Phase_Average    | Value     |
| time              | "current_time"             | Timestamp |

### StorageData

Endpoint: `/solar_api/v1/GetStorageRealtimeData.cgi` <br/>
InfluxDB Measurement: `storage`

| Name              | Value (Fronius)        | Type      |
| ----------------- | ---------------------- | --------- |
| device            | "Storage"              | Tag       |
| enabled:          | Enable                 | Value     |
| charge_percentage | StateOfCharge_Relative | Value     |
| capacity          | Capacity_Maximum       | Value     |
| dc_current        | Current_DC             | Value     |
| dc_voltage        | Voltage_DC             | Value     |
| temperature_cell  | Temperature_Cell       | Value     |
| time              | "current_time"         | Timestamp |

### OhmPilotData

Endpoint: `/solar_api/v1/GetOhmPilotRealtimeData.cgi`<br/>
InfluxDB Measurement: `ohm_pilot`

| Name        | Value (Fronius)       | Type      |
| ----------- | --------------------- | --------- |
| device      | "OhmPilot"            | Tag       |
| state       | CodeOfState           | Value     |
| error_code  | CodeOfError           | Value     |
| power       | PowerReal_PAC_Sum     | Value     |
| temperature | Temperature_Channel_1 | Value     |
| time        | "current_time"        | Timestamp |

### PowerFlowData

Endpoint: `/solar_api/v1/GetPowerFlowRealtimeData.fcgi` <br/>
InfluxDB Measurement: `power_flow`

| Name                      | Value (Fronius)     | Type      |
| ------------------------- | ------------------- | --------- |
| device                    | "Unknown"           | Tag       |
| akku                      | P_Akku              | Value     |
| grid                      | P_Grid              | Value     |
| load                      | P_Load              | Value     |
| photovoltaik              | P_PV                | Value     |
| relative_autonomy         | rel_Autonomy        | Value     |
| relative_self_consumption | rel_SelfConsumption | Value     |
| time                      | "current_time"      | Timestamp |

## Contributing

If you want to contribute you can do so in the following ways:

- Open issues for improvement ideas / bug reports
- Create pull request to fix open issues.

## Authors

- UnHold
