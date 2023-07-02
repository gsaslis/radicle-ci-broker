use hyper::{Client, Request};
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

        println!("setting token");
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
            .header(AUTHORIZATION, format!("Basic {}", self.token.clone().unwrap().access_token))
            .header(CONTENT_TYPE, "application/x-yaml")
            .header("x-concourse-config-version", "1")
            .body(body.into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result = serde_json::from_reader(body.reader())?;

        Ok(())
    }

    pub async fn unpause_pipeline(&self, project_id: String) -> Result<()> {
        let request = Request::builder()
            .method("PUT")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/unpause", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Basic {}", self.token.clone().unwrap().access_token))
            .body("".into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result = serde_json::from_reader(body.reader())?;

        Ok(())
    }

    pub async fn trigger_pipeline_configuration(&self, project_id: String) -> Result<()> {
        let request = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}-configure/jobs/configure-pipeline/builds", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Basic {}", self.token.clone().unwrap().access_token))
            .body("".into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result = serde_json::from_reader(body.reader())?;

        Ok(())
    }

    pub async fn get_pipeline_jobs(&self, project_id: String) -> Result<()> {
        let request = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v1/teams/main/pipelines/{}/jobs", self.concourse_uri, project_id))
            .header(AUTHORIZATION, format!("Basic {}", self.token.clone().unwrap().access_token))
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
            .header(AUTHORIZATION, format!("Basic {}", self.token.clone().unwrap().access_token))
            .body("".into())?;

        let _response = self.client.request(request).await?;
        // let body = hyper::body::aggregate(response).await?;
        // let result = serde_json::from_reader(body.reader())?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::concourse::api::ConcourseAPI;

    #[tokio::test]
    pub async fn hello() {
        let mut api = ConcourseAPI::new(
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

        let project_name = String::from("heartwood");
        let git_uri = String::from("https://seed.radicle.xyz/z3gqcJUoA1n9HaHKufZs5FCSGazv5.git");
        let project_id = String::from("heartwood");
        let patch_branch = String::from("d718d61870a1634455292d3ab6d2ba928157db19");
        let patch_head = String::from("ae16039b9d809a07f69e66844f4c539f6564ea3e");


        let result = api.create_pipeline(
            project_name,
            patch_branch,
            patch_head,
            project_id.clone(),
            git_uri,
        ).await;
        match result {
            Ok(_) => println!("create pipeline success"),
            Err(error) => println!("create pipeline error {}", error),
        }

        let result = api.unpause_pipeline(project_id.clone()).await;
        match result {
            Ok(_) => println!("unpause pipeline success"),
            Err(error) => println!("upause pipeline error {}", error),
        }

        let result = api.trigger_pipeline_configuration(project_id.clone()).await;
        match result {
            Ok(_) => println!("trigger pipeline configuration success"),
            Err(error) => println!("trigger pipeline configuration error {}", error),
        }

        let result = api.get_pipeline_jobs(project_id.clone()).await;
        match result {
            Ok(_) => println!("get pipeline jobs success"),
            Err(error) => println!("get pipeline jobs error {}", error),
        }

        let result = api.trigger_job(project_id.clone(), String::from("job name")).await;
        match result {
            Ok(_) => println!("trigger job success"),
            Err(error) => println!("trigger job error {}", error),
        }
    }
}