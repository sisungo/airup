use crate::rpc::api::Method;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Router {
    map: HashMap<&'static str, Route>,
}
impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(mut self, key: &'static str, route: Method) -> Self {
        self.map.insert(key, route.into());
        self
    }

    pub fn nest(mut self, key: &'static str, nest: Router) -> Self {
        self.map.insert(key, nest.into());
        self
    }

    pub fn get_method(&self, mut key: &str) -> Option<Method> {
        let mut router = self;
        loop {
            match key.split_once(".") {
                Some((a, b)) => {
                    router = match self.map.get(a) {
                        Some(Route::Router(x)) => x,
                        _ => break None,
                    };
                    key = b;
                    continue;
                }
                None => match router.map.get(key) {
                    Some(Route::Method(x)) => break Some(*x),
                    _ => break None,
                },
            }
        }
    }
}

#[derive(Debug)]
pub enum Route {
    Method(Method),
    Router(Box<Router>),
}
impl From<Method> for Route {
    fn from(value: Method) -> Self {
        Self::Method(value)
    }
}
impl From<Router> for Route {
    fn from(value: Router) -> Self {
        Self::Router(Box::new(value))
    }
}
