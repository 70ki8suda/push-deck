use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayTarget {
    Main,
    TopStrip,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayFrame {
    pub target: DisplayTarget,
    pub payload: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayError;

impl Display for DisplayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("display adapter error")
    }
}

impl Error for DisplayError {}

pub type DisplayResult<T> = Result<T, DisplayError>;

pub trait DisplayAdapter {
    fn connect(&mut self) -> DisplayResult<()>;
    fn disconnect(&mut self) -> DisplayResult<()>;
    fn render(&mut self, frame: DisplayFrame) -> DisplayResult<()>;
    fn clear(&mut self) -> DisplayResult<()>;
}

#[derive(Debug, Default)]
pub struct NoopDisplayAdapter;

impl DisplayAdapter for NoopDisplayAdapter {
    fn connect(&mut self) -> DisplayResult<()> {
        Ok(())
    }

    fn disconnect(&mut self) -> DisplayResult<()> {
        Ok(())
    }

    fn render(&mut self, _frame: DisplayFrame) -> DisplayResult<()> {
        Ok(())
    }

    fn clear(&mut self) -> DisplayResult<()> {
        Ok(())
    }
}
