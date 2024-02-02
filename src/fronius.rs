use reqwest::{blocking::Client, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{borrow::Borrow, collections::HashMap, net::IpAddr};
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
        device_id: DeviceId,
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

    pub fn get_inverter_realtime_data_system(
        &self,
    ) -> Result<CumulationInverterDataSystem, Error> {
        let response: CommonResponseBody<_> = self.make_request(
            "GetInverterRealtimeData.cgi",
            [
                ("Scope", "System")
            ],
        )?;
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
    timestamp: time::OffsetDateTime,
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
    unit: String,
    values: HashMap<String, Option<T>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnitAndValue<T> {
    unit: String,
    value: Option<T>,
}


pub type CumulationInverterData = inner::CumulationInverterDataProto<inner::SingleValue>;

pub type CumulationInverterDataSystem = inner::CumulationInverterDataProto<inner::ManyValues>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct CommonInverterData {
    pac: UnitAndValue<f64>,
    sac: UnitAndValue<f64>,
    iac: UnitAndValue<f64>,
    uac: UnitAndValue<f64>,
    fac: UnitAndValue<f64>,
    idc: UnitAndValue<f64>,
    idc_2: UnitAndValue<f64>,
    idc_3: UnitAndValue<f64>,
    idc_4: UnitAndValue<f64>,
    udc: UnitAndValue<f64>,
    udc_2: UnitAndValue<f64>,
    udc_3: UnitAndValue<f64>,
    udc_4: UnitAndValue<f64>,
    day_energy: UnitAndValue<f64>,
    year_energy: UnitAndValue<f64>,
    total_energy: UnitAndValue<f64>,
    pub device_status: DeviceStatus,
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
