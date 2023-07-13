use std::time::Duration;

use anyhow::Error;
use tokio::time::sleep;

use radicle_term as term;

use crate::ci::CI;
use crate::concourse::api::ConcourseAPI;

pub(crate) struct ConcourseCI {
    runtime: tokio::runtime::Runtime,
    api: ConcourseAPI,
}

impl Clone for ConcourseCI {
    fn clone(&self) -> Self {
        Self {
            // TODO: Investigate if this is the right way to clone a runtime
            runtime: tokio::runtime::Runtime::new().unwrap(),
            api: self.api.clone(),
        }
    }
}

impl ConcourseCI {
    // TODO: Create and use a CIConfig struct instead of passing individual parameters
    pub fn new(concourse_uri: String, ci_user: String, ci_pass: String) -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let api = ConcourseAPI::new(concourse_uri, ci_user, ci_pass);

        Self { runtime, api }
    }
}

impl CI for ConcourseCI {
    fn setup(&mut self, project_name: String, patch_branch: String, patch_head: String, project_id: &String, git_uri: String) -> Result<(), Error> {
        self.runtime.block_on(async {
            term::info!("Getting access token");
            let result = self.api.get_access_token().await;

            if let Ok(token) = result {
                term::info!("Access token acquired {}", token.access_token);
            } else {
                return Err(anyhow::anyhow!("Failed to get access token"));
            }

            term::info!("Creating the pipeline");
            let result = self.api.create_pipeline(project_name, patch_branch, patch_head, &project_id, git_uri).await;

            // TODO: Poll until pipeline is created
            sleep(Duration::from_secs(10)).await;

            if let Ok(()) = result {
                term::info!("Pipeline configuration creation triggered");
            } else {
                return Err(anyhow::anyhow!("Failed to trigger create pipeline configuration"));
            }

            term::info!("Unpausing the pipeline");
            let result = self.api.unpause_pipeline(&project_id).await;
            if let Ok(job) = result {
                term::info!("Pipeline configuration unpaused {:?}", job);
            } else {
                return Err(anyhow::anyhow!("Failed to unpause pipeline configuration"));
            }

            Ok(())
        })
    }

    fn run_pipeline(&self, project_id: &String) -> Result<(), Error> {
        self.runtime.block_on(async {
            let result = self.api.trigger_pipeline_configuration(project_id).await;
            if let Ok(pipeline_configuration_job) = result {
                term::info!("Pipeline configuration triggered {:?}", pipeline_configuration_job);
            } else {
                return Err(anyhow::anyhow!("Failed to trigger pipeline configuration"));
            }

            // TODO: Poll until pipeline configuration is completed
            sleep(Duration::from_secs(10)).await;

            let result = self.api.get_pipeline_jobs(project_id).await;
            if let Ok(ref jobs) = &result {
                let job = jobs.get(0).unwrap();
                term::info!("Pipeline jobs {:?}", jobs);
                let result = self.api.trigger_job(project_id, &job.name).await;
                match result {
                    Ok(_) => term::info!("Job {} triggered", job.name),
                    Err(error) => term::info!("Unable to trigger job {}", job.name),
                }
            } else {
                term::info!("Failed to get pipeline jobs");
                return Err(anyhow::anyhow!("Failed to get pipeline jobs"));
            }

            Ok(())
        })
    }
}
