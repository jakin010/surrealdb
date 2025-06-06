use http::{
	HeaderMap, StatusCode,
	header::{ACCEPT, CONTENT_TYPE},
};

use crate::{
	err::Error,
	expr::{Object, Value},
	rpc::format::Format,
};

use super::{convert, err::ApiError, invocation::ApiInvocation};

#[derive(Debug)]
pub struct ApiResponse {
	pub raw: Option<bool>,
	pub status: StatusCode,
	pub body: Option<Value>,
	pub headers: HeaderMap,
}

impl TryFrom<Value> for ApiResponse {
	type Error = Error;
	fn try_from(value: Value) -> Result<Self, Self::Error> {
		if let Value::Object(mut opts) = value {
			let raw = opts.remove("raw").map(|v| v.cast_to()).transpose()?;
			let status = opts
				.remove("status")
				.map(|v| -> Result<StatusCode, Error> {
					// Convert to int
					let v: i64 = v.coerce_to()?;

					// Convert to u16
					let v: u16 = v
						.try_into()
						.map_err(|_| Error::ArithmeticOverflow(format!("{v} as u16")))?;

					// Convert to StatusCode
					v.try_into().map_err(|_| {
						ApiError::InvalidApiResponse(format!("{v} is not a valid HTTP status code"))
							.into()
					})
				})
				.transpose()?
				.unwrap_or(StatusCode::OK);

			let headers = opts
				.remove("headers")
				.map(|v| v.coerce_to::<Object>()?.try_into())
				.transpose()?
				.unwrap_or_default();

			let body = opts.remove("body");

			if !opts.is_empty() {
				Err(ApiError::InvalidApiResponse("Contains invalid properties".into()).into())
			} else {
				Ok(Self {
					raw,
					status,
					body,
					headers,
				})
			}
		} else {
			Err(ApiError::InvalidApiResponse("Expected an object".into()).into())
		}
	}
}

impl TryInto<Value> for ApiResponse {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<Value, Self::Error> {
		Ok(Value::Object(
			map! {
				"raw" => Value::from(self.raw.unwrap_or(false)),
				"status" => Value::from(self.status.as_u16() as i64),
				"headers" => Value::Object(convert::headermap_to_object(self.headers)?),
				"body", if let Some(body) = self.body => body,
			}
			.into(),
		))
	}
}

pub enum ResponseInstruction {
	Native,
	Raw,
	Format(Format),
}

impl ResponseInstruction {
	pub fn for_format(invocation: &ApiInvocation) -> Result<Self, Error> {
		let mime = invocation
			.headers
			.get(ACCEPT)
			.or_else(|| invocation.headers.get(CONTENT_TYPE))
			.and_then(|v| v.to_str().ok());

		let format = match mime {
			Some("application/json") => Format::Json,
			Some("application/cbor") => Format::Cbor,
			Some("application/surrealdb") => Format::Revision,
			Some(_) => return Err(Error::ApiError(ApiError::InvalidFormat)),
			_ => return Err(Error::ApiError(ApiError::MissingFormat)),
		};

		Ok(ResponseInstruction::Format(format))
	}
}
