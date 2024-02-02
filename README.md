# Fronius API

This project provides functions to interface with the API of Fronius devices.
This project has only been tested on a GEN24 with an Batterypack and an OhmPilot.

## Supported API calls

| API path                                    | library function                                                              |
| ------------------------------------------- | ----------------------------------------------------------------------------- |
| /solar_api/v1/GetInverterRealtimeData.cgi   | `get_inverter_realtime_data_system()` `get_inverter_realtime_data_device()`   |
| /solar_api/v1/GetInverterInfo.cgi           | `get_inverter_info()`                                                         |
| /solar_api/v1/GetActiveDeviceInfo.cgi       | `get_active_device_info()`                                                    |
| /solar_api/v1/GetMeterRealtimeData.cgi      | `get_meter_realtime_data_system()` `get_meter_realtime_data_device()`         |
| /solar_api/v1/GetStorageRealtimeData.cgi    | `get_storage_realtime_data_system()` `get_storage_realtime_data_device()`     |
| /solar_api/v1/GetOhmPilotRealtimeData.cgi   | `get_ohm_pilot_realtime_data_system()` `get_ohm_pilot_realtime_data_device()` |
| /solar_api/v1/GetPowerFlowRealtimeData.fcgi | `get_power_flow_realtime_data()`                                              |

## Example usage

```rs
    let ip = IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1));
    let fronius = Fronius::connect(ip)?;
    println!(
        "{:#?}",
        fronius.get_inverter_realtime_data_system()?
    );
```
