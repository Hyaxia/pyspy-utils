#[derive(serde::Serialize, Debug, Deserialize, Clone)]
pub struct Shared {
    pub frames: Vec<Frame>,
}

#[derive(serde::Serialize, Debug, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub unit: String,
    #[serde(rename = "startValue")]
    pub start_value: f64,
    #[serde(rename = "endValue")]
    pub end_value: f64,
    pub samples: Vec<Vec<i32>>,
    pub weights: Vec<f64>,
    pub r#type: String,
}

use serde::Deserialize;

#[derive(serde::Serialize, Debug, Deserialize, Clone)]
pub struct Speedscope {
    pub profiles: Vec<Profile>,
    pub shared: Shared,
    #[serde(rename = "$schema")]
    pub schema: String,
    // null value
    pub exporter: String,
    pub name: String,
}

#[derive(serde::Serialize, Debug, Deserialize, Clone)]
pub struct Frame {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub col: Option<u32>,
}

impl Frame {
    pub fn hash(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.name,
            self.file,
            self.line,
            self.col.unwrap_or(0)
        )
    }
}
