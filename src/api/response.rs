use std::error::Error;

use axum::http::StatusCode;
use serde::{ser::SerializeStruct, Serialize, Serializer};

use crate::status::ResponseStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiResponse<T, E> {
    result: Result<T, E>,
}

impl<T, E> ResponseStatus for ApiResponse<T, E>
where
    T: Serialize + ResponseStatus,
    E: Error + ResponseStatus,
{
    fn status_code(&self) -> StatusCode {
        match &self.result {
            Ok(data) => data.status_code(),
            Err(errors) => errors.status_code(),
        }
    }
}

impl<T, E> Serialize for ApiResponse<T, E>
where
    T: Serialize + ResponseStatus,
    E: Error + ResponseStatus,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struct_serializer =
            serializer.serialize_struct("ApiResponse", 2)?;
        struct_serializer
            .serialize_field("status", &self.status_code().as_u16())?;
        match &self.result {
            Ok(data) => {
                struct_serializer.serialize_field("data", data)?;
            },
            Err(errors) => {
                struct_serializer
                    .serialize_field("errors", &ErrorChain::new(errors))?;
            },
        }
        struct_serializer.end()
    }
}

impl<T, E> From<Result<T, E>> for ApiResponse<T, E>
where
    T: Serialize + ResponseStatus,
    E: Error + ResponseStatus,
{
    fn from(result: Result<T, E>) -> Self {
        Self { result }
    }
}

#[derive(Debug, Clone, Copy)]
struct ErrorChain<'a> {
    curr: Option<&'a (dyn Error + 'a)>,
}

impl<'a> ErrorChain<'a> {
    fn new(main: &'a (dyn Error + 'a)) -> Self {
        Self { curr: Some(main) }
    }
}

impl<'a> Iterator for ErrorChain<'a> {
    type Item = &'a (dyn Error + 'a);

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr.take()?;
        self.curr = curr.source();
        Some(curr)
    }
}

impl<'a> Serialize for ErrorChain<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.map(SerializeError))
    }
}

struct SerializeError<'a>(&'a (dyn Error + 'a));

impl<'a> Serialize for SerializeError<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}
