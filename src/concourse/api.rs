use hyper::{Client, Request, StatusCode};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub expires_in: i64,
    pub id_token: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job {
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
        let body = hyper::body::aggregate(response).await?;
        let token: Token = serde_json::from_reader(body.reader())?;

        self.token = Some(token.clone());

        Ok(token)
    }

    pub async fn create_pipeline(&self, project_name: String, patch_branch: String, patch_head: String, project_id: String, git_uri: String) -> Result<()> {
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
            .header(AUTHORIZATION, format!("Basic {}", self.token.as_ref().unwrap().access_token))
            .header(CONTENT_TYPE, "application/x-yaml")
            .header("x-concourse-config-version", "1")
            .body(body.into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result = serde_json::from_reader(body.reader())?;

        Ok(())
    }

    pub async fn unpause_pipeline(&self, project_id: String) -> Result<(())> {
        let request = Request::builder()
            .method("PUT")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/unpause", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Basic {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result = serde_json::from_reader(body.reader())?;

        Ok(())
    }

    pub async fn trigger_pipeline_configuration(&self, project_id: String) -> Result<Job> {
        let request = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/jobs/configure-pipeline/builds", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Basic {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let response = self.client.request(request).await?;
        let body = hyper::body::aggregate(response).await?;
        let job: Job = serde_json::from_reader(body.reader())?;

        Ok(job)
    }

    pub async fn get_pipeline_jobs(&self, project_id: String) -> Result<()> {
        let request = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Basic {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result = serde_json::from_reader(body.reader())?;

        Ok(())
    }

    pub async fn trigger_job(&self, project_id: String, job_name: String) -> Result<()> {
        let request = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs/{}/builds", self.concourse_uri, project_id, job_name))
            .header(AUTHORIZATION, format!("Basic {}", self.token.as_ref().unwrap().access_token))
            .body("".into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result: Job = serde_json::from_reader(body.reader())?;

        Ok(())
    }
}