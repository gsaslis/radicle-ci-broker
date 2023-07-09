pub trait CI: Clone {
    fn setup(&mut self, project_name: String, patch_branch: String, patch_head: String, project_id: &String, git_uri: String) -> Result<(), anyhow::Error>;
    fn run_pipeline(&self, project_id: &String) -> Result<(), anyhow::Error>;
}
