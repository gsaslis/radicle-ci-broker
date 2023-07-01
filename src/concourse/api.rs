use hyper::{Client, Request};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub access_token: String,
    pub expires_in: i64,
    pub id_token: String,
    pub token_type: String,
}

pub struct ConcourseAPI {
    client: Client<HttpConnector>,
    ci_pass: String,
    ci_user: String,
    concourse_uri: String,
}

impl ConcourseAPI {
    pub fn new(concourse_uri: String, ci_user: String, ci_pass: String) -> ConcourseAPI {
        ConcourseAPI {
            client: Client::new(),
            concourse_uri,
            ci_user,
            ci_pass,
        }
    }

    pub async fn get_access_token(&self) -> Result<Token> {
        let path = "/sky/issuer/token";

        let request = Request::builder()
            .method("POST")
            .uri(format!("{}{}", self.concourse_uri, path))
            .header(AUTHORIZATION, "Basic Zmx5OlpteDU=")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!("grant_type=password&username={}&password={}&scope=openid%20profile%20email%20federated:id%20groups", self.ci_user, self.ci_pass).into())?;

        let response = self.client.request(request).await?;
        let body = hyper::body::aggregate(response).await?;
        let token: Token = serde_json::from_reader(body.reader())?;

        Ok(token)
    }
}

#[cfg(test)]
mod test {
    use crate::concourse::api::ConcourseAPI;

    #[tokio::test]
    pub async fn hello() {
        let api = ConcourseAPI::new(
            String::from("http://127.0.0.1:8080"),
            String::from("test"),
            String::from("test"),
        );
        let result = api.get_access_token().await;

        if let Ok(token) = result {
            println!("Access token: {}", token.access_token);
        } else {
            assert!(false);
        }
    }
}