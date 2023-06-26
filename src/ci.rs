pub trait CI {
    fn setup(&self) -> Result<(), anyhow::Error>;
    fn run_pipeline(&self) -> Result<(), anyhow::Error>;
}
