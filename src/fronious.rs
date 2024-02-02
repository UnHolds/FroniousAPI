use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{collections::HashMap, net::IpAddr};
use thiserror::Error;

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
    request_arguments: HashMap<String, String>,
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
    values: HashMap<String, T>,
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
    device_status: Option<HashMap<String, serde_json::Value>>,
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
