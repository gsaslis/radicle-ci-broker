use std::{thread, time};

use crossbeam_channel::Sender;

use radicle::{Profile};
use radicle::cob::patch::{Patches};
use radicle::node::{Event, Handle};
use radicle::prelude::ReadStorage;
use radicle::storage::git::Repository;
use radicle::storage::RefUpdate;
use radicle_term as term;

use crate::concourse::ci;
use crate::pool::Pool;
use crate::worker::CIJob;

// TODO: Capture SIGINT and SIGTERM to gracefully shutdown

pub struct CIConfig {
    pub concourse_uri: String,
    pub ci_user: String,
    pub ci_pass: String,
}

pub struct Runtime {
    #[allow(dead_code)]
    pool: Pool,
    profile: Profile,
    sender: Sender<CIJob>,
}

impl Runtime {
    pub fn new(profile: Profile, ci_config: CIConfig) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let handle = ci::ConcourseCI::new(ci_config.concourse_uri, ci_config.ci_user, ci_config.ci_pass);

        Runtime {
            pool: Pool::with(receiver, handle),
            profile,
            sender,
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let t = thread::Builder::new().name(String::from("node-events")).spawn(move || {
            subscribe_to_node_events(self.profile, self.sender)
        })?;
        t.join().unwrap()?;
        Ok(())
    }
}

fn trigger_ci_on_patch(repository: &Repository, patch_id: &str, sender: Sender<CIJob>) -> Result<(), anyhow::Error> {
    let patches = Patches::open(repository)?;
    let patch = patches.get(&patch_id.parse().unwrap()).unwrap().unwrap();
    term::info!("Triggering CI job for patch with ID {patch_id}");

    let repo_id = repository.id.canonical();
    sender.send(CIJob {
        // NOTE: For the time being we are using the repo id for the project name
        project_name: repo_id.clone(),
        patch_branch: String::from(patch_id),
        patch_head: patch.head().to_string(),
        project_id: repo_id.clone(),
        git_uri: format!("https://seed.radicle.xyz/{repo_id}.git"),
    }).expect("Failed to send job to pool");

    Ok(())
}

fn subscribe_to_node_events(profile: Profile, sender: Sender<CIJob>) -> anyhow::Result<()> {
    term::info!("Subscribing to node events ...");
    let node = radicle::Node::new(profile.socket());
    let events = node.subscribe(time::Duration::MAX)?;

    for event in events {
        let event = event?;

        term::info!("Received event {:?}", event);
        match event {
            Event::RefsFetched { remote: _, rid, updated } => {
                for refs in updated {
                    match refs {
                        RefUpdate::Updated { name, .. } => {
                            term::info!("Update reference announcement received: {name}");
                            if name.contains("xyz.radicle.patch") {
                                let patch_id = name.split('/').last().unwrap();
                                let repository = profile.storage.repository(rid)?;
                                trigger_ci_on_patch(&repository, patch_id, sender.clone())?;
                            }
                        }
                        RefUpdate::Created { name, .. } => {
                            term::info!("Creation reference announcement received: {name}");
                            if name.contains("xyz.radicle.patch") {
                                let patch_id = name.split('/').last().unwrap();
                                let repository = profile.storage.repository(rid)?;
                                trigger_ci_on_patch(&repository, patch_id, sender.clone())?;
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}
