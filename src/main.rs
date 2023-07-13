use std::{process, time};

use radicle::cob::patch::{Patch, Patches};
use radicle::node::{Event, Handle};
use radicle::prelude::ReadStorage;
use radicle::storage::git::Repository;
use radicle::storage::RefUpdate;
use radicle::profile::Profile;

use radicle_term as term;

fn profile() -> Result<Profile, anyhow::Error> {
    match Profile::load() {
        Ok(profile) => Ok(profile),
        Err(_) => Err(anyhow::anyhow!("Could not load radicle profile")),
    }
}

fn show_patch_info(repository: &Repository, patch_id: &str) -> Result<Patch, anyhow::Error> {
    let patches = Patches::open(repository)?;
    let patch = patches.get(&patch_id.parse().unwrap()).unwrap().unwrap();
    term::info!("--------------------");
    term::info!("Title {}", patch.title());
    term::info!("Head {:?}", *patch.head());

    Ok(patch)
}

pub fn execute() -> anyhow::Result<()> {
    term::info!("CI Broker init ...");
    let profile = profile()?;

    let node = radicle::Node::new(profile.socket());
    term::info!("Subscribing to node events ...");
    let events = node.subscribe(time::Duration::MAX)?;

    for event in events {
        let event = event?;

        term::info!("Received event {:?}", event);
        match event {
            Event::RefsFetched { remote: _, rid, updated } => {
                for refs in updated {
                    match refs {
                        RefUpdate::Updated { name, old, new } => {
                            term::info!("RefUpdate::Updated: {}, old {}, new {}", name.clone(), old, new);
                            if name.contains("xyz.radicle.patch") {
                                let patch_id = name.split('/').last().unwrap();
                                let repository = profile.storage.repository(rid)?;
                                let patch = show_patch_info(&repository, patch_id)?;
                                term::info!("RefUpdate::Updated: {}, oid {}, {}", name, patch_id, patch.title());
                            }
                        }
                        RefUpdate::Created { name, oid } => {
                            term::info!("RefUpdate::Updated: {}, {}", name.clone(), oid);
                            if name.contains("xyz.radicle.patch") {
                                let patch_id = name.split('/').last().unwrap();
                                let repository = profile.storage.repository(rid)?;
                                let patch = show_patch_info(&repository, patch_id)?;
                                term::info!("RefUpdate::Created: {}, oid {}, {}", name, patch_id, patch.title());
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

fn main() {
    if let Err(err) = execute() {
        term::info!("Fatal: {err}");
        process::exit(1);
    }
}
