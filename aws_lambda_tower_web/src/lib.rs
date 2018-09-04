extern crate aws_lambda_gateway;
extern crate aws_lambda_runtime;
extern crate bytes;
extern crate failure;
#[macro_use]
extern crate futures;
extern crate http;
extern crate tower_service;
extern crate tower_web;
extern crate void;

use std::error::Error as StdError;

use aws_lambda_gateway::NewApiGatewayProxy;
use aws_lambda_runtime::Runtime;

use tower_web::error::IntoCatch;
use tower_web::response::DefaultSerializer;
use tower_web::routing::{IntoResource, RoutedService};
use tower_web::util::http::HttpMiddleware;
use tower_web::util::BufStream;
use tower_web::ServiceBuilder;

mod body;
use body::RequestBody;
mod service;
use service::NewServiceWrapper;

pub trait ServiceBuilderExt {
    fn run_lambda(self);
}

impl<T, C, M> ServiceBuilderExt for ServiceBuilder<T, C, M>
where
    T: IntoResource<DefaultSerializer, RequestBody>,
    T::Resource: 'static,
    C: IntoCatch<DefaultSerializer>,
    C::Catch: 'static,
    M: HttpMiddleware<RoutedService<T::Resource, C::Catch>, RequestBody = ::RequestBody> + 'static,
    M::Error: StdError + Send + Sync + 'static,
    <M::ResponseBody as BufStream>::Error: StdError + Send + Sync + 'static,
{
    fn run_lambda(self) {
        let new_service = NewServiceWrapper {
            inner: self.build_new_service(),
        };
        let new_proxy = NewApiGatewayProxy::new(new_service);
        Runtime::new()
            .and_then(|runtime| runtime.start_service(new_proxy))
            .unwrap_or_else(|err| panic!("failed to start runtime: {}", err))
    }
}