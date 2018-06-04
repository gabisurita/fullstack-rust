use std::env;
use std::ops::Deref;

use r2d2::{Pool, PooledConnection};
use r2d2_redis::RedisConnectionManager;
use rocket::{http, request, Outcome, State};

/// Redis connection pool.
pub type RedisPool = Pool<RedisConnectionManager>;

/// Creates a new Redis connection pool.
pub fn init_pool() -> RedisPool {
    let database_url =
        env::var("DATABASE_URL").unwrap_or("redis://localhost".to_string());
    let manager = RedisConnectionManager::new(database_url.as_str()).unwrap();

    Pool::builder().build(manager).unwrap()
}

/// Redis connection wrapper.
pub type RedisConnection = PooledConnection<RedisConnectionManager>;

pub struct RedisConnectionWrapper(RedisConnection);

/// Allow access to the inner connection type using Deref.
impl Deref for RedisConnectionWrapper {
    type Target = RedisConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allow controller access to a RedisConnection.
impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnectionWrapper {
    type Error = ();

    fn from_request(
        request: &'a request::Request<'r>,
    ) -> request::Outcome<RedisConnectionWrapper, ()> {
        let pool = request.guard::<State<RedisPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(RedisConnectionWrapper(conn)),
            Err(_) => Outcome::Failure((http::Status::ServiceUnavailable, ())),
        }
    }
}
