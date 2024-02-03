use std::{future, net::IpAddr};

use fronius::{DeviceId, Fronius};
use influxdb2::{Client, FromDataPoint};
use influxdb2_derive::WriteDataPoint;
use chrono::prelude::*;
mod fronius;


#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "inverter"]
struct InverterData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    ac_power: f64,
    #[influxdb(field)]
    ac_power_abs: f64,
    #[influxdb(field)]
    ac_current: f64,
    #[influxdb(field)]
    ac_voltage: f64,
    #[influxdb(field)]
    ac_frequency: f64,
    #[influxdb(field)]
    dc_current: f64,
    #[influxdb(field)]
    dc_voltage: f64,
    #[influxdb(field)]
    total_energy: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_inverter_data(fronius: &Fronius, device_id: &DeviceId) -> Result<InverterData, Box<dyn std::error::Error>> {
    let response = fronius.get_inverter_realtime_data_device::<fronius::CommonInverterData>(device_id.to_owned())?;
    let data = InverterData {
        device: "Inverter".to_owned(),
        ac_power: response.pac.value.expect("Value not found in response data"),
        ac_power_abs: response.sac.value.expect("Value not found in response data"),
        ac_current: response.iac.value.expect("Value not found in response data"),
        ac_voltage: response.uac.value.expect("Value not found in response data"),
        ac_frequency: response.fac.value.expect("Value not found in response data"),
        dc_current: response.idc.value.expect("Value not found in response data"),
        dc_voltage: response.udc.value.expect("Value not found in response data"),
        total_energy: response.total_energy.value.expect("Value not found in response data"),
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "inverter_phase"]
struct InverterPhaseData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    ac_l1_current: f64,
    #[influxdb(field)]
    ac_l2_current: f64,
    #[influxdb(field)]
    ac_l3_current: f64,
    #[influxdb(field)]
    dc_l1_voltage: f64,
    #[influxdb(field)]
    dc_l2_voltage: f64,
    #[influxdb(field)]
    dc_l3_voltage: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_inverter_phase_data(fronius: &Fronius, device_id: &DeviceId) -> Result<InverterPhaseData, Box<dyn std::error::Error>> {
    let response = fronius.get_inverter_realtime_data_device::<fronius::ThreePhaseInverterData>(device_id.to_owned())?;
    let data = InverterPhaseData {
        device: "Inverter".to_owned(),
        ac_l1_current: response.iac_l1.value.expect("Value not found in response data"),
        ac_l2_current: response.iac_l2.value.expect("Value not found in response data"),
        ac_l3_current: response.iac_l3.value.expect("Value not found in response data"),
        dc_l1_voltage: response.uac_l1.value.expect("Value not found in response data"),
        dc_l2_voltage: response.uac_l2.value.expect("Value not found in response data"),
        dc_l3_voltage: response.uac_l3.value.expect("Value not found in response data"),
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "inverter_info"]
struct InverterInfo {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    device_type: i64,
    #[influxdb(field)]
    pv_power: i64,
    #[influxdb(field)]
    name: String,
    #[influxdb(field)]
    is_visualized: bool,
    #[influxdb(field)]
    id: String,
    #[influxdb(field)]
    error_code: i64,
    #[influxdb(field)]
    status_code: String,
    #[influxdb(field)]
    state: String,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_inverter_info(fronius: &Fronius, device_id: &DeviceId) -> Result<InverterInfo, Box<dyn std::error::Error>> {
    let device_id = u8::from(device_id.to_owned()).to_string();
    let res = fronius.get_inverter_info()?;
    let response = res[&device_id].as_ref().expect("Invalid device id");
    let data = InverterInfo {
        device: "Inverter".to_owned(),
        device_type: response.dt,
        pv_power: response.pv_power,
        name: response.custom_name.to_owned(),
        is_visualized: response.show > 0,
        id: response.unique_id.to_owned(),
        error_code: response.error_code,
        status_code: response.status_code.to_string(),
        state: response.inverter_state.to_owned(),
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "meter"]
struct MeterData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    l1_current: f64,
    #[influxdb(field)]
    l2_current: f64,
    #[influxdb(field)]
    l3_current: f64,
    #[influxdb(field)]
    current: f64,
    #[influxdb(field)]
    l1_voltage: f64,
    #[influxdb(field)]
    l2_voltage: f64,
    #[influxdb(field)]
    l3_voltage: f64,
    #[influxdb(field)]
    l12_voltage: f64,
    #[influxdb(field)]
    l23_voltage: f64,
    #[influxdb(field)]
    l31_voltage: f64,
    #[influxdb(field)]
    l1_power: f64,
    #[influxdb(field)]
    l2_power: f64,
    #[influxdb(field)]
    l3_power: f64,
    #[influxdb(field)]
    power: f64,
    #[influxdb(field)]
    frequency_average: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_meter_data(fronius: &Fronius, device_id: &DeviceId) -> Result<MeterData, Box<dyn std::error::Error>> {
    let response = fronius.get_meter_realtime_data_device(device_id)?;
    let data = MeterData {
        device: "Meter".to_owned(),
        l1_current: response.current_ac_phase_1.expect("Value not found in response data"),
        l2_current: response.current_ac_phase_2.expect("Value not found in response data"),
        l3_current: response.current_ac_phase_3.expect("Value not found in response data"),
        current: response.current_ac_sum.expect("Value not found in response data"),
        l1_voltage: response.voltage_ac_phase_1.expect("Value not found in response data"),
        l2_voltage: response.voltage_ac_phase_2.expect("Value not found in response data"),
        l3_voltage: response.voltage_ac_phase_3.expect("Value not found in response data"),
        l12_voltage: response.voltage_ac_phase_to_phase_12.expect("Value not found in response data"),
        l23_voltage: response.voltage_ac_phase_to_phase_23.expect("Value not found in response data"),
        l31_voltage: response.voltage_ac_phase_to_phase_31.expect("Value not found in response data"),
        l1_power: response.power_real_p_phase_1.expect("Value not found in response data"),
        l2_power: response.power_real_p_phase_2.expect("Value not found in response data"),
        l3_power: response.power_real_p_phase_3.expect("Value not found in response data"),
        power: response.power_real_p_sum,
        frequency_average: response.frequency_phase_average,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "storage"]
struct StorageData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    enabled: bool,
    #[influxdb(field)]
    charge_percentage: f64,
    #[influxdb(field)]
    capacity: f64,
    #[influxdb(field)]
    dc_current: f64,
    #[influxdb(field)]
    dc_voltage: f64,
    #[influxdb(field)]
    temperature_cell: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_storage_data(fronius: &Fronius, device_id: &DeviceId) -> Result<StorageData, Box<dyn std::error::Error>> {
    let response = fronius.get_storage_realtime_data_device(device_id)?;
    let data = StorageData {
        device: "Storage".to_owned(),
        enabled: response.controller.enable > 0,
        charge_percentage: response.controller.state_of_charge_relative,
        capacity: response.controller.capacity_maximum,
        dc_current: response.controller.current_dc,
        dc_voltage: response.controller.voltage_dc,
        temperature_cell: response.controller.temperature_cell,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "ohm_pilot"]
struct OhmPilotData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    state: String,
    #[influxdb(field)]
    error_code: i64,
    #[influxdb(field)]
    power: f64,
    #[influxdb(field)]
    temperature: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_ohm_pilot_data(fronius: &Fronius, device_id: &DeviceId) -> Result<OhmPilotData, Box<dyn std::error::Error>> {
    let response = fronius.get_ohm_pilot_realtime_data_device(device_id)?;
    let data = OhmPilotData {
        device: "OhmPilot".to_owned(),
        state: response.code_of_state.to_string(),
        error_code: response.code_of_error.unwrap_or(0),
        power: response.power_real_pac_sum,
        temperature: response.temperature_channel_1,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "power_flow"]
struct PowerFlowData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    akku: f64,
    #[influxdb(field)]
    grid: f64,
    #[influxdb(field)]
    load: f64,
    #[influxdb(field)]
    photovoltaik: f64,
    #[influxdb(field)]
    relative_autonomy: f64,
    #[influxdb(field)]
    relative_self_consumption: f64,
    #[influxdb(timestamp)]
    time: i64
}

fn get_power_flow_data(fronius: &Fronius) -> Result<PowerFlowData, Box<dyn std::error::Error>> {
    let response = fronius.get_power_flow_realtime_data()?;
    let data = PowerFlowData {
        device: "Unknown".to_owned(),
        akku: response.site.p_akku.expect("Value not found in response data"),
        grid: response.site.p_grid.expect("Value not found in response data"),
        load: response.site.p_load.expect("Value not found in response data"),
        photovoltaik: response.site.p_pv,
        relative_autonomy: response.site.rel_autonomy.expect("Value not found in response data"),
        relative_self_consumption: response.site.rel_self_consumption.expect("Value not found in response data"),
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

fn fetch_data(fronius: &Fronius) -> Result<(), Box<dyn std::error::Error>> {
    let interver_id = DeviceId::try_from(1).unwrap();
    let meter_id = DeviceId::try_from(0).unwrap();
    let storage_id = DeviceId::try_from(0).unwrap();
    let ohm_pilot_id = DeviceId::try_from(0).unwrap();
    let inverter_data = get_inverter_data(fronius, &interver_id)?;
    let inverter_phase_data = get_inverter_phase_data(fronius, &interver_id)?;
    let inverter_info = get_inverter_info(fronius, &interver_id)?;
    let meter_data = get_meter_data(fronius, &meter_id)?;
    let storage_data = get_storage_data(fronius, &storage_id)?;
    let ohm_pilot_data = get_ohm_pilot_data(fronius, &ohm_pilot_id)?;
    let power_flow_data = get_power_flow_data(fronius)?;

    let client = Client::new("http://10.69.0.5:8086", "Local", "7xh2Mi8NAQANT7erOTxkjKlH7tEUlFYmOq9sy4bl-JUsG0_pwu85CzUJk-77fMKVKN3hwXDoY6HPhw3n9Lga6g==");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write( "photovoltaics", futures::stream::iter(vec![inverter_data])))?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write( "photovoltaics", futures::stream::iter(vec![inverter_phase_data])))?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write( "photovoltaics", futures::stream::iter(vec![inverter_info])))?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write( "photovoltaics", futures::stream::iter(vec![meter_data])))?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write( "photovoltaics", futures::stream::iter(vec![storage_data])))?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write( "photovoltaics", futures::stream::iter(vec![ohm_pilot_data])))?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write( "photovoltaics", futures::stream::iter(vec![power_flow_data])))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip = IpAddr::V4(std::net::Ipv4Addr::new(10, 69, 0, 50));
    let fronius = Fronius::connect(ip)?;
    loop {
        fetch_data(&fronius)?;
        std::thread::sleep(std::time::Duration::from_secs(15));
        let now = Utc::now();
        println!("Reporting data at: {now}")
    }
}
