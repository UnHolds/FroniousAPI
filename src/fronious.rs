use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug)]
pub struct ValueError(String);

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid value given! Reason: {}", self.0)
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FroniousResponse<T> {
    head: CommonResponseHeader,
    body: T,
}

#[derive(Debug, Serialize, Deserialize)]
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
    request_arguments: std::collections::HashMap<String, String>,
    status: Status,
    #[serde(with = "time::serde::rfc3339")]
    timestamp: time::OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnitAndValue<T> {
    unit: String,
    value: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonResponseBody<T> {
    data: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnitAndValues<T> {
    unit: String,
    values: std::collections::HashMap<String, T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiVersion {
    #[serde(rename = "APIVersion")]
    api_version: i8,
    #[serde(rename = "BaseURL")]
    base_url: String,
    compatibility_range: String,
}

pub fn get_api_version(ip: IpAddr) -> Result<ApiVersion, Box<dyn std::error::Error>> {
    let mut url = reqwest::Url::parse("http://placeholder.local/solar_api/GetAPIVersion.cgi")?;
    let _ = url.set_ip_host(ip);
    Ok(reqwest::blocking::Client::new().get(url).send()?.json()?)
}

pub enum Scope {
    System,
    Device {
        device_id: DeviceId,
        data_collection: DataCollection,
    },
}

pub struct DeviceId(u8);

impl TryFrom<u8> for DeviceId {
    type Error = ValueError;

    fn try_from(device_id: u8) -> Result<Self, ValueError> {
        if device_id <= 99 {
            Ok(Self(device_id))
        } else {
            Err(ValueError("device id not in range!".into()))
        }
    }
}

impl From<DeviceId> for u8 {
    fn from(device_id: DeviceId) -> u8 {
        device_id.0
    }
}

#[derive(strum_macros::Display)]
pub enum DataCollection {
    CumulationInverterData,
    CommonInverterData,
    ThreePInverterData,
    MinMaxInverterData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CumulationInverterData<T> {
    pac: T,
    day_energy: T,
    year_energy: T,
    total_energy: T,
    device_status: Option<std::collections::HashMap<String, String>>,
}

pub fn get_inverter_realtime_data(
    ip: IpAddr,
    scope: Scope,
) -> Result<FroniousResponse, Box<dyn std::error::Error>> {
    let mut params: Vec<(&str, String)> = vec![];

    match scope {
        Scope::System => {
            params.push(("Scope", "System".to_owned()));
        }
        Scope::Device {
            device_id,
            data_collection,
        } => {
            params.push(("Scope", "Device".to_owned()));
            params.push(("DeviceId", (u8::from(device_id)).to_string()));
            params.push(("DataCollection", data_collection.to_string()));
        }
    }

    let mut url = reqwest::Url::parse_with_params(
        &format!("http://placeholder.local/solar_api/v1/GetInverterRealtimeData.cgi"),
        &params,
    )?;
    let _ = url.set_ip_host(ip);
    Ok(reqwest::blocking::Client::new().get(url).send()?.json()?)
}
