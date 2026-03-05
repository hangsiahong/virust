use anyhow::Result;
use crate::dev_orchestrator::DevOrchestrator;

pub fn execute(port: u16) -> Result<()> {
    let mut orchestrator = DevOrchestrator::new(port);
    orchestrator.run()
}