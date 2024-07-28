use axum::http::StatusCode;
use serde::Serialize;

pub trait ResponseStatusCode {
    fn status_code(&self) -> StatusCode;
}

pub trait WithResultStatus {
    type Ok;
    type Err;

    fn with_http_status(
        self,
        status_code: StatusCode,
    ) -> Result<
        WithStatusCode<<Self as WithResultStatus>::Ok>,
        <Self as WithResultStatus>::Err,
    >;

    fn with_err_http_status(
        self,
        status_code: StatusCode,
    ) -> Result<
        <Self as WithResultStatus>::Ok,
        WithStatusCode<<Self as WithResultStatus>::Err>,
    >;
}

impl<T, E> WithResultStatus for Result<T, E> {
    type Ok = T;
    type Err = E;

    fn with_http_status(
        self,
        status_code: StatusCode,
    ) -> Result<
        WithStatusCode<<Self as WithResultStatus>::Ok>,
        <Self as WithResultStatus>::Err,
    > {
        self.map(|data| WithStatusCode::new(status_code, data))
    }

    fn with_err_http_status(
        self,
        status_code: StatusCode,
    ) -> Result<
        <Self as WithResultStatus>::Ok,
        WithStatusCode<<Self as WithResultStatus>::Err>,
    > {
        self.map_err(|error| WithStatusCode::new(status_code, error))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WithStatusCode<T> {
    status_code: StatusCode,
    target: T,
}

impl<T> WithStatusCode<T> {
    pub fn new(status_code: StatusCode, target: T) -> Self {
        Self { status_code, target }
    }
}

impl<T> ResponseStatusCode for WithStatusCode<T> {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}

impl<T> Serialize for WithStatusCode<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.target.serialize(serializer)
    }
}
