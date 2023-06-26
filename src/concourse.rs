use std::process::{Command, Output};

use anyhow::Error;

use radicle_term as term;

use crate::ci::CI;

struct Concourse {
    pipeline_config_path: String,
    pipeline_name: String,
    job_name: String,
    username: String,
    password: String,
    target: String,
    url: String,
}

impl Concourse {
    fn login(&self) -> std::io::Result<Output> {
        // fly -t tutorial login -c http://localhost:8080 -u test -p test
        Command::new("fly")
            .args([
                "-t", &self.target,
                "login",
                "-c", &self.url,
                "-u", &self.username,
                "-p", &self.password,
            ])
            .output()
    }

    fn set_pipeline(&self) -> std::io::Result<Output> {
        // fly -t tutorial set-pipeline -p hello-world -c hello-world.yml
        Command::new("fly")
            .args([
                "-t", &self.target,
                "set-pipeline",
                "--non-interactive",
                "-p", &self.pipeline_name,
                "-c", &self.pipeline_config_path,
            ])
            .output()
    }

    fn unpause_pipeline(&self) -> std::io::Result<Output> {
        // fly -t tutorial unpause-pipeline -p hello-world
        Command::new("fly")
            .args([
                "-t", &self.target,
                "unpause-pipeline",
                "-p", &self.pipeline_name
            ])
            .output()
    }

    fn trigger_job(&self) -> std::io::Result<Output> {
        // fly -t tutorial trigger-job --job hello-world/hello-world-job
        Command::new("fly")
            .args([
                "-t", &self.target,
                "trigger-job",
                "--job", format!("{}/{}", &self.pipeline_name, &self.job_name).as_str()
            ])
            .output()
    }
}

impl CI for Concourse {
    fn setup(&self) -> Result<(), Error> {
        let output = self.login()?;
        term::info!("login stdout: {}", String::from_utf8(output.stdout)?);
        term::info!("login stderr: {}", String::from_utf8(output.stderr)?);

        let output = self.set_pipeline()?;
        term::info!("set_pipeline stdout: {}", String::from_utf8(output.stdout)?);
        term::info!("set_pipeline stderr: {}", String::from_utf8(output.stderr)?);

        let output = self.unpause_pipeline()?;
        term::info!("unpause_pipeline stdout: {}", String::from_utf8(output.stdout)?);
        term::info!("unpause_pipeline stderr: {}", String::from_utf8(output.stderr)?);

        Ok(())
    }

    fn run_pipeline(&self) -> Result<(), Error> {
        let output = self.trigger_job()?;
        term::info!("trigger_job stdout: {}", String::from_utf8(output.stdout)?);
        term::info!("trigger_job stderr: {}", String::from_utf8(output.stderr)?);

        Ok(())
    }
}