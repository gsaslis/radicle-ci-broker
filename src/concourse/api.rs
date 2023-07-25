use std::io::Read;

use hyper::{Body, Client, Request, Response};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use serde::Deserialize;

use crate::concourse::response_error::ResponseError;
use crate::concourse::token::Token;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
pub struct PlanStep {
    pub get: Option<String>,
    pub version: Option<String>,
    pub file: Option<String>,
    pub set_pipeline: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PipelineJob {
    pub name: String,
    pub plan: Vec<PlanStep>,
}

#[derive(Deserialize, Debug)]
pub struct Source {
    pub branch: String,
    pub uri: String,
}

#[derive(Deserialize, Debug)]
pub struct Resource {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub source: Source,
    pub icon: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub resources: Vec<Resource>,
    pub jobs: Vec<PipelineJob>,
}

#[derive(Deserialize, Debug)]
pub struct PipelineConfiguration {
    pub config: Config,
}

#[derive(Deserialize, Debug)]
pub struct PipelineConfigurationJob {
    pub id: i64,
    pub team_name: String,
    pub name: String,
    pub status: String,
    pub api_url: String,
    pub job_name: String,
    pub pipeline_id: i64,
    pub pipeline_name: String,
    pub created_by: String,
}

#[derive(Deserialize, Debug)]
pub struct Job {
    pub id: i32,
    pub name: String,
    pub team_name: String,
    pub pipeline_id: i32,
    pub pipeline_name: String,
}

async fn deserialize_json_response<T>(response: Response<Body>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
{
    let body = hyper::body::aggregate(response).await?;
    let result: T = serde_json::from_reader(body.reader())?;
    Ok(result)
}

async fn deserialize_string_response(response: Response<Body>) -> Result<String> {
    let content_length = response.headers().get(CONTENT_LENGTH).unwrap().to_str()?.parse::<usize>().unwrap();
    let body = hyper::body::aggregate(response).await?;
    let mut dst = vec![0; content_length];
    let num = body.reader().read(&mut dst)?;
    let result = std::str::from_utf8(&dst[..num])?;
    Ok(result.to_string())
}

#[derive(Clone)]
pub struct ConcourseAPI {
    client: Client<HttpConnector>,
    ci_pass: String,
    ci_user: String,
    concourse_uri: String,
    token: Option<Token>,
}

impl ConcourseAPI {
    pub fn new(concourse_uri: String, ci_user: String, ci_pass: String) -> ConcourseAPI {
        ConcourseAPI {
            client: Client::new(),
            concourse_uri,
            ci_user,
            ci_pass,
            token: None,
        }
    }

    pub async fn get_access_token(&mut self) -> Result<Token> {
        let path = "/sky/issuer/token";

        let request = Request::builder()
            .method("POST")
            .uri(format!("{}{}", self.concourse_uri, path))
            .header(AUTHORIZATION, "Basic Zmx5OlpteDU=")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!("grant_type=password&username={}&password={}&scope=openid%20profile%20email%20federated:id%20groups", self.ci_user, self.ci_pass).into())?;

        let response = self.client.request(request).await?;
        let token = deserialize_json_response::<Token>(response).await?;

        self.token = Some(token.clone());

        Ok(token)
    }

    pub async fn get_pipeline(&self, project_id: &String) -> Result<PipelineConfiguration> {
        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/config", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Bearer {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            deserialize_json_response::<PipelineConfiguration>(response).await
        }
    }

    pub async fn create_pipeline(&self, project_name: String, patch_branch: String, patch_head: String, project_id: &String, git_uri: String) -> Result<()> {
        let body = format!(r#"
jobs:
- name: configure-pipeline
  plan:
  - get: {project_name}
    version: {patch_head}
    trigger: false
  - set_pipeline: {project_id}
    file: {project_name}/.concourse/config.yaml

resources:
- name: {project_name}
  type: git
  icon: git
  source:
    uri: {git_uri}
    branch: {patch_branch}
        "#);

        let request = Request::builder()
            .method("PUT")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/config", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Bearer {}", self.token.as_ref().unwrap().access_token))
            .header(CONTENT_TYPE, "application/x-yaml")
            .header("X-Concourse-Config-Version", "1")
            .body(body.into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            Ok(())
        }
    }

    pub async fn unpause_pipeline(&self, project_id: &String) -> Result<()> {
        let request = Request::builder()
            .method("PUT")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/unpause", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Bearer {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            Ok(())
        }
    }

    pub async fn trigger_pipeline_configuration(&self, project_id: &String) -> Result<PipelineConfigurationJob> {
        let request: Request<Body> = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/jobs/configure-pipeline/builds", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Bearer {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        // Both 4xx and 5xx responses return a string body that cannot be parsed as JSON.
        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            deserialize_json_response::<PipelineConfigurationJob>(response).await
        }
    }

    pub async fn get_pipeline_jobs(&self, project_id: &String) -> Result<Vec<Job>> {
        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Bearer {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<Vec<Job>>(response).await?;
            Ok(result)
        }
    }

    pub async fn trigger_job(&self, project_id: &String, job_name: &String) -> Result<PipelineConfigurationJob> {
        let request = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs/{}/builds", self.concourse_uri, project_id, job_name))
            .header(AUTHORIZATION, format!("Bearer {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let string = deserialize_string_response(response).await?;
            Err(Box::new(ResponseError { errors: vec![string], warnings: None }))
        } else {
            let result = deserialize_json_response::<PipelineConfigurationJob>(response).await?;
            Ok(result)
        }
    }
}
