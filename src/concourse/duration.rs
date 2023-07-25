use serde::Deserialize;
use serde_json::Number;

pub fn deserialize_to_duration<'de, D>(deserializer: D) -> Result<std::time::Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let n = Number::deserialize(deserializer)?;
    let secs = n.as_u64().map_or(Err(serde::de::Error::custom("not a u64")), Ok)?;
    Ok(std::time::Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use crate::concourse::duration::deserialize_to_duration;

    #[derive(Deserialize)]
    struct Token {
        #[serde(deserialize_with = "deserialize_to_duration")]
        expires_in: std::time::Duration,
    }

    #[test]
    fn will_successfully_deserialize_a_number_to_duration() -> Result<(), serde_json::Error> {
        let string = "{ \"expires_in\": 123456 }";
        let token: Token = serde_json::from_str(string)?;
        assert_eq!(token.expires_in, std::time::Duration::from_secs(123456));
        Ok(())
    }

    #[test]
    fn will_return_an_error_if_it_tries_to_deserialize_a_string_to_duration() -> Result<(), serde_json::Error> {
        let string = "{ \"expires_in\": \"123456\" }";
        let result = serde_json::from_str::<Token>(string);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn will_return_an_error_if_the_number_is_not_a_u64() -> Result<(), serde_json::Error> {
        let string = "{ \"expires_in\": 123456.789 }";
        let result = serde_json::from_str::<Token>(string);
        assert!(result.is_err());
        Ok(())
    }
}