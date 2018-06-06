use redis::{Commands, RedisError, RedisResult};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{self, Error as JsonError};

#[derive(Debug)]
pub enum RepositoryError {
    RedisError(RedisError),
    JsonError(JsonError),
}

impl From<RedisError> for RepositoryError {
    fn from(error: RedisError) -> Self {
        RepositoryError::RedisError(error)
    }
}

impl From<JsonError> for RepositoryError {
    fn from(error: JsonError) -> Self {
        RepositoryError::JsonError(error)
    }
}

pub trait QueueRepository<T>: Commands
where
    T: Serialize + DeserializeOwned, {
    fn key(&self) -> &str;

    fn all(&self) -> Result<Vec<T>, RepositoryError> {
        let result: Vec<String> = self.lrange(self.key(), 0, -1)?;
        Ok(result
            .iter()
            .filter_map(|row| serde_json::from_str(row.as_str()).ok())
            .collect())
    }

    fn push(&self, data: &T) -> Result<isize, RepositoryError> {
        let data = serde_json::to_string(&data)?;
        Ok(self.rpush(self.key(), data)?)
    }

    fn replace(
        &self,
        index: isize,
        data: &T,
    ) -> Result<isize, RepositoryError> {
        let _result: RedisResult<String> =
            self.lset(self.key(), index, serde_json::to_string(&data)?);
        Ok(index)
    }

    fn delete(&self, index: isize) -> Result<T, RepositoryError> {
        let value: String = self.lindex(self.key(), index)?;
        let _result: i32 = self.lrem(self.key(), 1, &value)?;
        Ok(serde_json::from_str(&value)?)
    }
}
