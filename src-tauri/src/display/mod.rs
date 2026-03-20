use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::pin::Pin;

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
pub type DisplayFuture<'a, T> = Pin<Box<dyn Future<Output = DisplayResult<T>> + Send + 'a>>;

pub trait DisplayAdapter {
    fn connect(&mut self) -> DisplayFuture<'_, ()>;
    fn disconnect(&mut self) -> DisplayFuture<'_, ()>;
    fn render(&mut self, frame: DisplayFrame) -> DisplayFuture<'_, ()>;
    fn clear(&mut self) -> DisplayFuture<'_, ()>;
}

#[derive(Debug, Default)]
pub struct NoopDisplayAdapter;

impl DisplayAdapter for NoopDisplayAdapter {
    fn connect(&mut self) -> DisplayFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }

    fn disconnect(&mut self) -> DisplayFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }

    fn render(&mut self, _frame: DisplayFrame) -> DisplayFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }

    fn clear(&mut self) -> DisplayFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}
