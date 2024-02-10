use reqwest::{blocking::Client, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{borrow::Borrow, collections::HashMap, net::IpAddr};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported API version {0}")]
    UnsupportedApiVersion(u64),
    #[error("invalid endpoint {0:?}")]
    InvalidEndpoint(String),
    #[error("request failed")]
    Request(#[from] reqwest::Error),
    #[error("decoding response body failed")]
    Decode(#[from] serde_json::Error),
    #[error("received error response {:?}: {}", .0.code, .0.reason)]
    Response(Status),
}

pub struct Fronius {
    client: Client,
    base_url: Url,
}

impl Fronius {
    pub fn connect(ip: IpAddr) -> Result<Self, Error> {
        let client = Client::new();

        let mut url = reqwest::Url::parse("http://placeholder.local/solar_api/GetAPIVersion.cgi")
            .expect("Initial base URL should be valid");
        url.set_ip_host(ip)
            .expect("Base URL should be a valid base");
        let api_version: ApiVersion = client.get(url.clone()).send()?.json()?;

        if api_version.api_version != 1 {
            return Err(Error::UnsupportedApiVersion(api_version.api_version));
        }

        url.set_path(&api_version.base_url);

        Ok(Self {
            client,
            base_url: url,
        })
    }

    fn make_request_inner(&self, url: Url) -> Result<serde_json::Value, Error> {
        let response: FroniusResponse<serde_json::Value> = self.client.get(url).send()?.json()?;

        if response.head.status.code != StatusCode::Okay {
            return Err(Error::Response(response.head.status));
        }

        Ok(response.body)
    }

    pub fn make_request<T, I, K, V>(&self, endpoint: &str, params: I) -> Result<T, Error>
    where
        T: DeserializeOwned,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut url = self
            .base_url
            .join(endpoint)
            .map_err(|_e| Error::InvalidEndpoint(endpoint.to_string()))?;
        url.query_pairs_mut().extend_pairs(params);
        let body = self.make_request_inner(url)?;

        Ok(T::deserialize(body)?)
    }

    pub fn get_inverter_realtime_data_device<C: DataCollection>(
        &self,
        device_id: &DeviceId,
    ) -> Result<C, Error> {
        let device_id = u8::from(device_id).to_string();

        let response: CommonResponseBody<_> = self.make_request(
            "GetInverterRealtimeData.cgi",
            [
                ("Scope", "Device"),
                ("DeviceId", &device_id),
                ("DataCollection", C::param_value()),
            ],
        )?;

        Ok(response.data)
    }

    pub fn get_inverter_realtime_data_system(&self) -> Result<CumulationInverterDataSystem, Error> {
        let response: CommonResponseBody<_> =
            self.make_request("GetInverterRealtimeData.cgi", [("Scope", "System")])?;
        Ok(response.data)
    }

    pub fn get_inverter_info(&self) -> Result<InverterInfos, Error> {
        let response: CommonResponseBody<_> =
            self.make_request("GetInverterInfo.cgi", [] as [(&str, &str); 0])?;
        Ok(response.data)
    }

    pub fn get_active_device_info(&self) -> Result<DeviceInfos, Error> {
        let response: CommonResponseBody<_> =
            self.make_request("GetActiveDeviceInfo.cgi", [] as [(&str, &str); 0])?;
        Ok(response.data)
    }

    pub fn get_meter_realtime_data_system(&self) -> Result<MeterDataSystem, Error> {
        let response: CommonResponseBody<_> =
            self.make_request("GetMeterRealtimeData.cgi", [("Scope", "System")])?;
        Ok(response.data)
    }

    pub fn get_meter_realtime_data_device(&self, device_id: &DeviceId) -> Result<MeterData, Error> {
        let device_id = u8::from(device_id).to_string();
        let response: CommonResponseBody<_> = self.make_request(
            "GetMeterRealtimeData.cgi",
            [("Scope", "Device"), ("DeviceId", &device_id)],
        )?;
        Ok(response.data)
    }

    pub fn get_storage_realtime_data_system(&self) -> Result<StorageDataSystem, Error> {
        let response: CommonResponseBody<_> =
            self.make_request("GetStorageRealtimeData.cgi", [("Scope", "System")])?;
        Ok(response.data)
    }

    pub fn get_storage_realtime_data_device(
        &self,
        device_id: &DeviceId,
    ) -> Result<StorageData, Error> {
        let device_id = u8::from(device_id).to_string();
        let response: CommonResponseBody<_> = self.make_request(
            "GetStorageRealtimeData.cgi",
            [("Scope", "Device"), ("DeviceId", &device_id)],
        )?;
        Ok(response.data)
    }

    pub fn get_ohm_pilot_realtime_data_system(&self) -> Result<OhmPilotDataSystem, Error> {
        let response: CommonResponseBody<_> =
            self.make_request("GetOhmPilotRealtimeData.cgi", [("Scope", "System")])?;
        Ok(response.data)
    }

    pub fn get_ohm_pilot_realtime_data_device(
        &self,
        device_id: &DeviceId,
    ) -> Result<OhmPilotData, Error> {
        let device_id = u8::from(device_id).to_string();
        let response: CommonResponseBody<_> = self.make_request(
            "GetOhmPilotRealtimeData.cgi",
            [("Scope", "Device"), ("DeviceId", &device_id)],
        )?;
        Ok(response.data)
    }

    pub fn get_power_flow_realtime_data(&self) -> Result<PowerFlowData, Error> {
        let response: CommonResponseBody<_> =
            self.make_request("GetPowerFlowRealtimeData.fcgi", [] as [(&str, &str); 0])?;
        Ok(response.data)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FroniusResponse<T> {
    head: CommonResponseHeader,
    body: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum StatusCode {
    Okay = 0,
    NotImplemented = 1,
    Uninitialized = 2,
    Initialized = 3,
    Running = 4,
    Timeout = 5,
    ArgumentError = 6,
    LNRequestError = 7,
    LNRequestTimeout = 8,
    LNParseError = 9,
    ConfigIOError = 10,
    NotSupported = 11,
    DeviceNotAvailable = 12,
    UnknownError = 255,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    code: StatusCode,
    reason: String,
    user_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonResponseHeader {
    request_arguments: HashMap<String, String>,
    status: Status,
    #[serde(with = "time::serde::rfc3339")]
    timestamp: OffsetDateTime,
}

type DeviceStatus = Option<HashMap<String, serde_json::Value>>;

mod inner {
    use super::*;
    pub trait ValuesContainer {
        type Container<T: DeserializeOwned>: DeserializeOwned;
    }

    #[derive(Debug)]
    pub struct SingleValue;
    impl ValuesContainer for SingleValue {
        type Container<T: DeserializeOwned> = UnitAndValue<T>;
    }

    #[derive(Debug)]
    pub struct ManyValues;
    impl ValuesContainer for ManyValues {
        type Container<T: DeserializeOwned> = UnitAndValues<T>;
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub struct CumulationInverterDataProto<C: inner::ValuesContainer> {
        pub pac: C::Container<f64>,
        pub day_energy: C::Container<f64>,
        pub year_energy: C::Container<f64>,
        pub total_energy: C::Container<f64>,
        #[serde(rename = "DeviceStatus")]
        pub device_status: DeviceStatus,
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonResponseBody<T> {
    data: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ApiVersion {
    #[serde(rename = "APIVersion")]
    api_version: u64,
    #[serde(rename = "BaseURL")]
    base_url: String,
    compatibility_range: String,
}

pub struct DeviceId(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("invalid device ID, must be less than 100: {0}")]
pub struct InvalidDeviceId(u8);

impl TryFrom<u8> for DeviceId {
    type Error = InvalidDeviceId;

    fn try_from(device_id: u8) -> Result<Self, InvalidDeviceId> {
        if device_id <= 99 {
            Ok(Self(device_id))
        } else {
            Err(InvalidDeviceId(device_id))
        }
    }
}

impl From<&DeviceId> for u8 {
    fn from(device_id: &DeviceId) -> u8 {
        device_id.0
    }
}

impl From<DeviceId> for u8 {
    fn from(device_id: DeviceId) -> u8 {
        device_id.0
    }
}

pub trait DataCollection: DeserializeOwned {
    /// Returns the value of the `DataCollection` GET parameter for this collection.
    fn param_value() -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnitAndValues<T> {
    pub unit: String,
    pub values: HashMap<String, Option<T>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnitAndValue<T> {
    pub unit: String,
    pub value: Option<T>,
}

pub type CumulationInverterData = inner::CumulationInverterDataProto<inner::SingleValue>;

pub type CumulationInverterDataSystem = inner::CumulationInverterDataProto<inner::ManyValues>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct CommonInverterData {
    pub sac: UnitAndValue<f64>,
    pub pac: UnitAndValue<f64>,
    pub iac: UnitAndValue<f64>,
    pub uac: UnitAndValue<f64>,
    pub fac: Option<UnitAndValue<f64>>,
    pub idc: UnitAndValue<f64>,
    pub idc_2: UnitAndValue<f64>,
    pub idc_3: UnitAndValue<f64>,
    pub idc_4: UnitAndValue<f64>,
    pub udc: UnitAndValue<f64>,
    pub udc_2: UnitAndValue<f64>,
    pub udc_3: UnitAndValue<f64>,
    pub udc_4: UnitAndValue<f64>,
    pub day_energy: UnitAndValue<f64>,
    pub year_energy: UnitAndValue<f64>,
    pub total_energy: UnitAndValue<f64>,
    pub device_status: DeviceStatus,
}

pub type ThreePInverterData = ThreePhaseInverterData;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ThreePhaseInverterData {
    pub iac_l1: UnitAndValue<f64>,
    pub iac_l2: UnitAndValue<f64>,
    pub iac_l3: UnitAndValue<f64>,
    pub uac_l1: UnitAndValue<f64>,
    pub uac_l2: UnitAndValue<f64>,
    pub uac_l3: UnitAndValue<f64>,
    pub t_ambient: Option<UnitAndValue<f64>>,
    pub rotation_speed_fan_fl: Option<UnitAndValue<f64>>,
    pub rotation_speed_fan_fr: Option<UnitAndValue<f64>>,
    pub rotation_speed_fan_bl: Option<UnitAndValue<f64>>,
    pub rotation_speed_fan_br: Option<UnitAndValue<f64>>,
}

impl DataCollection for CumulationInverterData {
    fn param_value() -> &'static str {
        "CumulationInverterData"
    }
}

impl DataCollection for CommonInverterData {
    fn param_value() -> &'static str {
        "CommonInverterData"
    }
}

impl DataCollection for ThreePhaseInverterData {
    fn param_value() -> &'static str {
        "3PInverterData"
    }
}

pub type InverterInfos = HashMap<String, Option<InverterInfo>>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InverterInfo {
    #[serde(rename = "DT")]
    pub dt: i64,
    #[serde(rename = "PVPower")]
    pub pv_power: i64,
    pub custom_name: String,
    pub show: u64,
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
    pub error_code: i64,
    pub status_code: InverterStatusCode,
    pub inverter_state: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum InverterStatusCode {
    Startup = 0, //0-6
    Running = 7,
    Standby = 8,
    Bootloading = 9,
    Error = 10,
    Idle = 11,
    Ready = 12,
    Sleeping = 13,
    Unknown = 255,
}

impl std::fmt::Display for InverterStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InverterStatusCode::Startup => write!(f, "Startup"),
            InverterStatusCode::Running => write!(f, "Running"),
            InverterStatusCode::Standby => write!(f, "Standby"),
            InverterStatusCode::Bootloading => write!(f, "Bootloading"),
            InverterStatusCode::Error => write!(f, "Error"),
            InverterStatusCode::Idle => write!(f, "Idle"),
            InverterStatusCode::Ready => write!(f, "Ready"),
            InverterStatusCode::Sleeping => write!(f, "Sleeping"),
            InverterStatusCode::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Inverter,
    Storage,
    Ohmpilot,
    SensorCard,
    StringControl,
    Meter,
    System,
}

pub type DeviceInfos = HashMap<DeviceType, HashMap<String, Option<DeviceInfo>>>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceInfo {
    #[serde(rename = "DT")]
    dt: i64,
    serial: String,
}

pub type MeterDataSystem = HashMap<String, MeterData>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeterData {
    pub details: DeviceDetails,
    #[serde(rename = "Current_AC_Phase_1")]
    pub current_ac_phase_1: Option<f64>,
    #[serde(rename = "Current_AC_Phase_2")]
    pub current_ac_phase_2: Option<f64>,
    #[serde(rename = "Current_AC_Phase_3")]
    pub current_ac_phase_3: Option<f64>,
    #[serde(rename = "Current_AC_Sum")]
    pub current_ac_sum: Option<f64>,
    pub enable: u8,
    #[serde(rename = "EnergyReactive_VArAC_Phase_1_Consumed")]
    pub energy_reactive_va_r_ac_phase_1_consumed: Option<f64>,
    #[serde(rename = "EnergyReactive_VArAC_Phase_1_Produced")]
    pub energy_reactive_va_r_ac_phase_1_produced: Option<f64>,
    #[serde(rename = "EnergyReactive_VArAC_Sum_Consumed")]
    pub energy_reactive_va_r_ac_sum_consumed: Option<f64>,
    #[serde(rename = "EnergyReactive_VArAC_Sum_Produced")]
    pub energy_reactive_va_r_ac_sum_produced: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Minus_Absolute")]
    pub energy_real_wac_minus_absolute: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Phase_1_Consumed")]
    pub energy_real_wac_phase_1_consumed: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Phase_1_Produced")]
    pub energy_real_wac_phase_1_produced: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Phase_2_Consumed")]
    pub energy_real_wac_phase_2_consumed: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Phase_2_Produced")]
    pub energy_real_wac_phase_2_produced: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Phase_3_Consumed")]
    pub energy_real_wac_phase_3_consumed: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Phase_3_Produced")]
    pub energy_real_wac_phase_3_produced: Option<f64>,
    #[serde(rename = "EnergyReal_WAC_Plus_Absolute")]
    pub energy_real_wac_plus_absolute: f64,
    #[serde(rename = "EnergyReal_WAC_Sum_Consumed")]
    pub energy_real_wac_sum_consumed: f64,
    #[serde(rename = "EnergyReal_WAC_Sum_Produced")]
    pub energy_real_wac_sum_produced: f64,
    #[serde(rename = "Frequency_Phase_Average")]
    pub frequency_phase_average: f64,
    #[serde(rename = "Meter_Location_Current")]
    pub meter_location_current: f64,
    #[serde(rename = "PowerApparent_S_Phase_1")]
    pub power_apparent_s_phase_1: Option<f64>,
    #[serde(rename = "PowerApparent_S_Phase_2")]
    pub power_apparent_s_phase_2: Option<f64>,
    #[serde(rename = "PowerApparent_S_Phase_3")]
    pub power_apparent_s_phase_3: Option<f64>,
    #[serde(rename = "PowerApparent_S_Sum")]
    pub power_apparent_s_sum: f64,
    #[serde(rename = "PowerFactor_Phase_1")]
    pub power_factor_phase_1: Option<f64>,
    #[serde(rename = "PowerFactor_Phase_2")]
    pub power_factor_phase_2: Option<f64>,
    #[serde(rename = "PowerFactor_Phase_3")]
    pub power_factor_phase_3: Option<f64>,
    #[serde(rename = "PowerFactor_Sum")]
    pub power_factor_sum: f64,
    #[serde(rename = "PowerReactive_Q_Phase_1")]
    pub power_reactive_q_phase_1: Option<f64>,
    #[serde(rename = "PowerReactive_Q_Phase_2")]
    pub power_reactive_q_phase_2: Option<f64>,
    #[serde(rename = "PowerReactive_Q_Phase_3")]
    pub power_reactive_q_phase_3: Option<f64>,
    #[serde(rename = "PowerReactive_Q_Sum")]
    pub power_reactive_q_sum: f64,
    #[serde(rename = "PowerReal_P_Phase_1")]
    pub power_real_p_phase_1: Option<f64>,
    #[serde(rename = "PowerReal_P_Phase_2")]
    pub power_real_p_phase_2: Option<f64>,
    #[serde(rename = "PowerReal_P_Phase_3")]
    pub power_real_p_phase_3: Option<f64>,
    #[serde(rename = "PowerReal_P_Sum")]
    pub power_real_p_sum: f64,
    #[serde(with = "time::serde::timestamp")]
    pub time_stamp: OffsetDateTime,
    pub visible: u8,
    #[serde(rename = "Voltage_AC_PhaseToPhase_12")]
    pub voltage_ac_phase_to_phase_12: Option<f64>,
    #[serde(rename = "Voltage_AC_PhaseToPhase_23")]
    pub voltage_ac_phase_to_phase_23: Option<f64>,
    #[serde(rename = "Voltage_AC_PhaseToPhase_31")]
    pub voltage_ac_phase_to_phase_31: Option<f64>,
    #[serde(rename = "Voltage_AC_Phase_1")]
    pub voltage_ac_phase_1: Option<f64>,
    #[serde(rename = "Voltage_AC_Phase_2")]
    pub voltage_ac_phase_2: Option<f64>,
    #[serde(rename = "Voltage_AC_Phase_3")]
    pub voltage_ac_phase_3: Option<f64>,
    #[serde(rename = "Voltage_AC_Phase_Average")]
    pub voltage_ac_phase_average: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceDetails {
    manufacturer: String,
    model: String,
    serial: String,
}

pub type StorageDataSystem = HashMap<String, StorageData>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageData {
    pub controller: StorageController,
    pub modules: Vec<StorageModule>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageController {
    pub details: DeviceDetails,
    #[serde(with = "time::serde::timestamp")]
    pub time_stamp: OffsetDateTime,
    pub enable: u8,
    #[serde(rename = "StateOfCharge_Relative")]
    pub state_of_charge_relative: f64,
    #[serde(rename = "Capacity_Maximum")]
    pub capacity_maximum: f64,
    #[serde(rename = "Current_DC")]
    pub current_dc: f64,
    #[serde(rename = "Voltage_DC")]
    pub voltage_dc: f64,
    #[serde(rename = "Temperature_Cell")]
    pub temperature_cell: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageModule {
    pub details: Option<DeviceDetails>,
    #[serde(rename = "Capacity_Maximum")]
    pub capacity_maximum: Option<f64>,
    #[serde(rename = "Current_DC")]
    pub current_dc: Option<f64>,
    #[serde(rename = "CycleCount_BatteryCell")]
    pub cycle_count_battery_cell: Option<f64>,
    #[serde(rename = "DesignedCapacity")]
    pub designed_capacity: Option<f64>,
    pub enable: Option<u8>,
    #[serde(rename = "StateOfCharge_Relative")]
    pub state_of_charge_relative: Option<f64>,
    #[serde(rename = "Status_BatteryCell")]
    pub status_battery_cell: Option<u64>,
    #[serde(rename = "Temperature_Cell")]
    pub temperature_cell: Option<f64>,
    #[serde(rename = "Temperature_Cell_Maximum")]
    pub temperature_cell_maximum: Option<f64>,
    #[serde(rename = "Temperature_Cell_Minimum")]
    pub temperature_cell_minimum: Option<f64>,
    #[serde(with = "time::serde::timestamp")]
    pub time_stamp: OffsetDateTime,
    #[serde(rename = "Voltage_DC")]
    pub voltage_dc: Option<f64>,
    #[serde(rename = "Voltage_DC_Maximum_Cell")]
    pub voltage_dc_maximum_cell: Option<f64>,
    #[serde(rename = "Voltage_DC_Minimum_Cell")]
    pub voltage_dc_minimum_cell: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OhmPilotDetails {
    pub serial: String,
    pub model: String,
    pub manufacturer: String,
    pub software: String,
    pub hardware: String,
}

pub type OhmPilotDataSystem = HashMap<String, OhmPilotData>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OhmPilotData {
    pub details: OhmPilotDetails,
    #[serde(rename = "EnergyReal_WAC_Sum_Consumed")]
    pub energy_real_wac_sum_consumed: f64,
    pub code_of_state: OhmPilotCodeOfState,
    pub code_of_error: Option<i64>,
    #[serde(rename = "PowerReal_PAC_Sum")]
    pub power_real_pac_sum: f64,
    #[serde(rename = "Temperature_Channel_1")]
    pub temperature_channel_1: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OhmPilotCodeOfState {
    UpAndRunning = 0,
    KeepMinimumTemperature = 1,
    LegionellaProtection = 2,
    CriticalFault = 3,
    Fault = 4,
    BoostMode = 5,
}

impl std::fmt::Display for OhmPilotCodeOfState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OhmPilotCodeOfState::UpAndRunning => write!(f, "Up and Running"),
            OhmPilotCodeOfState::KeepMinimumTemperature => write!(f, "Keeping minimum Temperature"),
            OhmPilotCodeOfState::LegionellaProtection => write!(f, "Legionella Protection"),
            OhmPilotCodeOfState::CriticalFault => write!(f, "Critical Fault"),
            OhmPilotCodeOfState::Fault => write!(f, "Fault"),
            OhmPilotCodeOfState::BoostMode => write!(f, "Boost Mode"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerFlowData {
    pub version: String,
    pub site: PowerFlowSite,
    pub inverters: HashMap<String, PowerFlowInverter>,
    pub smartloads: HashMap<String, HashMap<String, PowerFlowOhmPilots>>,
    pub secondary_meters: Option<HashMap<String, PowerFlowSecondaryMeters>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerFlowSite {
    pub mode: String,
    pub battery_standby: Option<bool>,
    pub backup_mode: Option<bool>,
    #[serde(rename = "P_Grid")]
    pub p_grid: Option<f64>,
    #[serde(rename = "P_Load")]
    pub p_load: Option<f64>,
    #[serde(rename = "P_Akku")]
    pub p_akku: Option<f64>,
    #[serde(rename = "P_PV")]
    pub p_pv: f64,
    #[serde(rename = "rel_SelfConsumption")]
    pub rel_self_consumption: Option<f64>,
    #[serde(rename = "rel_Autonomy")]
    pub rel_autonomy: Option<f64>,
    #[serde(rename = "Meter_Location")]
    pub meter_location: Option<String>,
    #[serde(rename = "E_Day")]
    pub e_day: Option<f64>,
    #[serde(rename = "E_Year")]
    pub e_year: Option<f64>,
    #[serde(rename = "E_Total")]
    pub e_total: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerFlowInverter {
    #[serde(rename = "DT")]
    pub dt: i64,
    pub p: f64,
    #[serde(rename = "SOC")]
    pub soc: Option<f64>,
    #[serde(rename = "CID")]
    pub cid: Option<u32>,
    #[serde(rename = "Battery_Mode")]
    pub battery_mode: Option<String>,
    #[serde(rename = "E_Day")]
    pub e_day: Option<f64>,
    #[serde(rename = "E_Year")]
    pub e_year: Option<f64>,
    #[serde(rename = "E_Total")]
    pub e_total: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerFlowOhmPilots {
    #[serde(rename = "P_AC_Total")]
    pub p_ac_total: f64,
    pub state: String,
    pub temperature: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerFlowSecondaryMeters {
    pub p: f64,
    pub m_loc: f64,
    pub label: String,
    pub category: String,
}
