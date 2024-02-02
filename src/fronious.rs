use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
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

#[derive(Debug, Serialize_repr, Deserialize_repr)]
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

pub trait DataCollection: DeserializeOwned {
    /// Returns the value of the `DataCollection` GET parameter for this collection.
    fn param_value() -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct CumulationInverterData {
    pac: UnitAndValue<u64>,
    day_energy: UnitAndValue<f64>,
    year_energy: UnitAndValue<f64>,
    total_energy: UnitAndValue<f64>,
    #[serde(rename = "DeviceStatus")]
    device_status: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl DataCollection for CumulationInverterData {
    fn param_value() -> &'static str {
        "CumulationInverterData"
    }
}

pub fn get_inverter_realtime_data_device<C: DataCollection>(
    ip: IpAddr,
    device_id: DeviceId,
) -> Result<FroniousResponse<CommonResponseBody<C>>, Box<dyn std::error::Error>> {
    let device_id = u8::from(device_id).to_string();
    let params = [
        ("Scope", "Device"),
        ("DeviceId", &device_id),
        ("DataCollection", C::param_value()),
    ];

    let mut url = reqwest::Url::parse_with_params(
        "http://placeholder.local/solar_api/v1/GetInverterRealtimeData.cgi",
        &params,
    )?;
    let _ = url.set_ip_host(ip);
    Ok(reqwest::blocking::Client::new().get(url).send()?.json()?)
}
