use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum_06::{body::Body, http::Request, response::Response};
use axum_06::async_trait;
use axum_06::extract::FromRequestParts;
use axum_06::http::header::AUTHORIZATION;
use axum_06::http::request::Parts;
use axum_06::http::StatusCode;
use axum_06::response::IntoResponse;
use tower::{Layer, Service};

use crate::propelauth::auth::PropelAuth;
use crate::propelauth::errors::{UnauthorizedError, UnauthorizedOrForbiddenError};
use crate::propelauth::token_models::User;

#[cfg(any(feature = "schemars09", feature = "schemars-latest"))]
use schemars::JsonSchema;

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))?;

        let auth = parts
            .extensions
            .get::<Arc<PropelAuth>>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "No layer found"))?;

        match auth.verify().validate_authorization_header(auth_header) {
            Ok(user) => Ok(user),
            Err(UnauthorizedError::Unauthorized(_)) => {
                Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
            }
        }
    }
}

#[cfg(feature = "axum06-extend-request")]
pub trait ExtendRequest: Clone + Send + Sync + 'static {
    fn extend(&self, req: &mut Request<Body>, auth: Arc<PropelAuth>);
}

#[cfg(feature = "axum06-extend-request")]
#[derive(Clone, Default)]
#[cfg_attr(any(feature = "schemars09", feature = "schemars-latest"), derive(JsonSchema))]
pub struct NoopExtend;

#[cfg(feature = "axum06-extend-request")]
impl ExtendRequest for NoopExtend {
    fn extend(&self, _req: &mut Request<Body>, _auth: Arc<PropelAuth>) {}
}

#[cfg(feature = "axum06-extend-request")]
#[derive(Clone)]
#[cfg_attr(any(feature = "schemars09", feature = "schemars-latest"), derive(JsonSchema))]
pub struct PropelAuthLayer<E = NoopExtend> {
    auth: Arc<PropelAuth>,
    extender: E,
}

#[cfg(not(feature = "axum06-extend-request"))]
#[derive(Clone)]
#[cfg_attr(any(feature = "schemars09", feature = "schemars-latest"), derive(JsonSchema))]
pub struct PropelAuthLayer {
    auth: Arc<PropelAuth>,
}

#[cfg(feature = "axum06-extend-request")]
impl PropelAuthLayer {
    pub fn new(auth: PropelAuth) -> PropelAuthLayer {
        PropelAuthLayer {
            auth: Arc::new(auth),
            extender: NoopExtend,
        }
    }
}

#[cfg(not(feature = "axum06-extend-request"))]
impl PropelAuthLayer {
    pub fn new(auth: PropelAuth) -> PropelAuthLayer {
        PropelAuthLayer {
            auth: Arc::new(auth),
        }
    }
}

#[cfg(feature = "axum06-extend-request")]
impl<E> PropelAuthLayer<E>
where
    E: ExtendRequest,
{
    pub fn with_extender(auth: PropelAuth, extender: E) -> PropelAuthLayer<E> {
        PropelAuthLayer {
            auth: Arc::new(auth),
            extender,
        }
    }
}

#[cfg(feature = "axum06-extend-request")]
impl<S, E> Layer<S> for PropelAuthLayer<E>
where
    E: ExtendRequest,
{
    type Service = PropelAuthMiddleware<S, E>;

    fn layer(&self, inner: S) -> Self::Service {
        PropelAuthMiddleware {
            inner,
            auth: self.auth.clone(),
            extender: self.extender.clone(),
        }
    }
}

#[cfg(not(feature = "axum06-extend-request"))]
impl<S> Layer<S> for PropelAuthLayer {
    type Service = PropelAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PropelAuthMiddleware {
            inner,
            auth: self.auth.clone(),
        }
    }
}

#[cfg(feature = "axum06-extend-request")]
#[derive(Clone)]
#[cfg_attr(any(feature = "schemars09", feature = "schemars-latest"), derive(JsonSchema))]
pub struct PropelAuthMiddleware<S, E = NoopExtend> {
    inner: S,
    auth: Arc<PropelAuth>,
    extender: E,
}

#[cfg(not(feature = "axum06-extend-request"))]
#[derive(Clone)]
#[cfg_attr(any(feature = "schemars09", feature = "schemars-latest"), derive(JsonSchema))]
pub struct PropelAuthMiddleware<S> {
    inner: S,
    auth: Arc<PropelAuth>,
}

#[cfg(feature = "axum06-extend-request")]
impl<S, E> Service<Request<Body>> for PropelAuthMiddleware<S, E>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    E: ExtendRequest,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let auth = self.auth.clone();
        request.extensions_mut().insert(auth.clone());
        self.extender.extend(&mut request, auth);
        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}

#[cfg(not(feature = "axum06-extend-request"))]
impl<S> Service<Request<Body>> for PropelAuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        request.extensions_mut().insert(self.auth.clone());
        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}

impl IntoResponse for UnauthorizedError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

impl IntoResponse for UnauthorizedOrForbiddenError {
    fn into_response(self) -> Response {
        match self {
            UnauthorizedOrForbiddenError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
            }
            UnauthorizedOrForbiddenError::Forbidden(_) => {
                (StatusCode::FORBIDDEN, "Forbidden").into_response()
            }
        }
    }
}
