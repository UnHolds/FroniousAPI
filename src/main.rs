use std::net::IpAddr;

use fronius::{DeviceId, Fronius};
mod fronius;

//inverted dev

struct CommonInverterData {
    ac_power: f64,
    ac_power_abs: f64,
    ac_current: f64,
    ac_voltage: f64,
    ac_frequency: f64,
    dc_current: f64,
    dc_voltage: f64,
    total_energy: f64
}

fn get_common_inverted_data(fronius: Fronius, device_id: DeviceId) -> Result<CommonInverterData, Box<dyn std::error::Error>> {
    let response = fronius.get_inverter_realtime_data_device::<fronius::CommonInverterData>(device_id)?;
    let data = CommonInverterData {
        ac_power: response.pac.value.expect("Value not found in response data"),
        ac_power_abs: response.sac.value.expect("Value not found in response data"),
        ac_current: response.iac.value.expect("Value not found in response data"),
        ac_voltage: response.uac.value.expect("Value not found in response data"),
        ac_frequency: response.fac.value.expect("Value not found in response data"),
        dc_current: response.idc.value.expect("Value not found in response data"),
        dc_voltage: response.udc.value.expect("Value not found in response data"),
        total_energy: response.total_energy.value.expect("Value not found in response data"),
    };
    Ok(data)
}

struct PhaseData {
    ac_l1_current: f64,
    ac_l2_current: f64,
    ac_l3_current: f64,
    dc_l1_voltage: f64,
    dc_l2_voltage: f64,
    dc_l3_voltage: f64,
}

fn get_inverter_phase_data(fronius: Fronius, device_id: DeviceId) -> Result<PhaseData, Box<dyn std::error::Error>> {
    let response = fronius.get_inverter_realtime_data_device::<fronius::ThreePhaseInverterData>(device_id)?;
    let data = PhaseData {
        ac_l1_current: response.iac_l1.value.expect("Value not found in response data"),
        ac_l2_current: response.iac_l2.value.expect("Value not found in response data"),
        ac_l3_current: response.iac_l3.value.expect("Value not found in response data"),
        dc_l1_voltage: response.uac_l1.value.expect("Value not found in response data"),
        dc_l2_voltage: response.uac_l2.value.expect("Value not found in response data"),
        dc_l3_voltage: response.uac_l3.value.expect("Value not found in response data"),
    };
    Ok(data)
}

struct InverterInfo {
    device_type: i64,
    pv_power: i64,
    name: String,
    is_visualized: bool,
    id: String,
    error_code: i64,
    status_code: fronius::InverterStatusCode,
    state: String
}

fn get_inverter_info(fronius: Fronius, device_id: DeviceId) -> Result<InverterInfo, Box<dyn std::error::Error>> {
    let device_id = u8::from(device_id).to_string();
    let res = fronius.get_inverter_info()?;
    let response = res[&device_id].as_ref().expect("Invalid device id");
    let data = InverterInfo {
        device_type: response.dt,
        pv_power: response.pv_power,
        name: response.custom_name.to_owned(),
        is_visualized: response.show > 0,
        id: response.unique_id.to_owned(),
        error_code: response.error_code,
        status_code: response.status_code,
        state: response.inverter_state.to_owned()
    };
    Ok(data)
}

struct MeterData {
    l1_current: f64,
    l2_current: f64,
    l3_current: f64,
    current: f64,
    l1_voltage: f64,
    l2_voltage: f64,
    l3_voltage: f64,
    l12_voltage: f64,
    l23_voltage: f64,
    l31_voltage: f64,
    l1_power: f64,
    l2_power: f64,
    l3_power: f64,
    power: f64,
    frequency_average: f64
}

fn get_meter_data(fronius: Fronius, device_id: DeviceId) -> Result<MeterData, Box<dyn std::error::Error>> {
    let response = fronius.get_meter_realtime_data_device(device_id)?;
    let data = MeterData {
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
        frequency_average: response.frequency_phase_average
    };
    Ok(data)
}

struct StorageData {
    enabled: bool,
    charge_percentage: f64,
    capacity: f64,
    dc_current: f64,
    dc_voltage: f64,
    temperature_cell: f64
}

fn get_storage_data(fronius: Fronius, device_id: DeviceId) -> Result<StorageData, Box<dyn std::error::Error>> {
    let response = fronius.get_storage_realtime_data_device(device_id)?;
    let data = StorageData {
        enabled: response.controller.enable > 0,
        charge_percentage: response.controller.state_of_charge_relative,
        capacity: response.controller.capacity_maximum,
        dc_current: response.controller.current_dc,
        dc_voltage: response.controller.voltage_dc,
        temperature_cell: response.controller.temperature_cell,
    };
    Ok(data)
}

struct OhmPilotData {
    state: fronius::OhmPilotCodeOfState,
    error_code: Option<i64>,
    power: f64,
    temperature: f64
}

fn get_ohm_pilot_data(fronius: Fronius, device_id: DeviceId) -> Result<OhmPilotData, Box<dyn std::error::Error>> {
    let response = fronius.get_ohm_pilot_realtime_data_device(device_id)?;
    let data = OhmPilotData {
        state: response.code_of_state,
        error_code: response.code_of_error,
        power: response.power_real_pac_sum,
        temperature: response.temperature_channel_1
    };
    Ok(data)
}

struct PowerFlowData {
    akku: f64,
    grid: f64,
    load: f64,
    photovoltaik: f64,
    relative_autonomy: f64,
    relative_self_consumption: f64
}

fn get_power_flow(fronius: Fronius) -> Result<PowerFlowData, Box<dyn std::error::Error>> {
    let response = fronius.get_power_flow_realtime_data()?;
    let data = PowerFlowData {
        akku: response.site.p_akku.expect("Value not found in response data"),
        grid: response.site.p_grid.expect("Value not found in response data"),
        load: response.site.p_load.expect("Value not found in response data"),
        photovoltaik: response.site.p_pv,
        relative_autonomy: response.site.rel_autonomy.expect("Value not found in response data"),
        relative_self_consumption: response.site.rel_self_consumption.expect("Value not found in response data"),
    };
    Ok(data)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip = IpAddr::V4(std::net::Ipv4Addr::new(10, 69, 0, 50));
    let fronius = Fronius::connect(ip)?;
    println!("{:#?}", fronius.get_power_flow_realtime_data()?);
    //println!("{:#?}", fronious::get_inverter_realtime_data(ip, fronious::Scope::System)?);
    Ok(())
}
