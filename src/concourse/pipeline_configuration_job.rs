use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PipelineConfigurationJob {
    pub id: u64,
    pub name: String,
    pub pipeline_id: u64,
    pub pipeline_name: String,
    pub team_name: String,
}

#[derive(Deserialize, Debug)]
pub struct PipelineConfigurationJobExtended {
    pub id: u64,
    pub name: String,
    pub pipeline_id: u64,
    pub pipeline_name: String,
    pub team_name: String,

    pub status: String,
    pub api_url: String,
    pub job_name: String,
    pub created_by: String,
}

