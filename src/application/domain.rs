use serde::Serialize;

#[derive(PartialEq, Debug)]
pub struct SmlMessages {
    pub messages: Vec<SmlMessageEnvelope>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum SmlMessageEnvelope {
    GetOpenResponse(GetOpenResponseBody),
    GetListResponse(GetListResponseBody),
    GetCloseResponse,
}

#[derive(PartialEq, Debug, Clone)]
pub struct GetOpenResponseBody {
    pub server_id: Vec<u8>,
    pub req_file_id: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct GetListResponseBody {
    pub server_id: Vec<u8>,
    pub list_name: Vec<u8>,
    pub value_list: Vec<SmlListEntry>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SmlListEntry {
    pub object_name: Vec<u8>,
    pub status: Option<u32>,
    pub value_time: Vec<u8>,
    pub unit: Option<u8>,
    pub scaler: Option<i8>,
    pub value: AnyValue,
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub enum AnyValue {
    Unsigned(usize),
    Signed(isize),
    String(Vec<u8>),
}

/// Scale an SML value by the given scaler (base 10)
pub trait Scale {
    fn scale(&self, scaler: i8) -> Self;
}

impl Scale for AnyValue {
    fn scale(&self, scaler: i8) -> Self {
        match self {
            AnyValue::Unsigned(v) => {
                let scaler = scaler;
                AnyValue::Unsigned((10f64.powf(scaler as f64) * *v as f64) as usize)
            }
            AnyValue::Signed(v) => {
                let scaler = scaler;
                AnyValue::Signed((10f64.powf(scaler as f64) * *v as f64) as isize)
            }
            AnyValue::String(v) => AnyValue::String(v.clone()),
        }
    }
}
