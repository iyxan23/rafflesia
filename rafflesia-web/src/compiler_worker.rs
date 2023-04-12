use serde::{Deserialize, Serialize};
use yew_agent::{HandlerId, Public, WorkerLink};

use crate::{virtfs::VirtualFs, compiler};

pub struct CompilerWorker {
    link: WorkerLink<Self>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompilerWorkerInput {
    pub fs: VirtualFs,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CompilerWorkerOutput {
    Success(ProjectData),
    Failure,
}

// encrypted and ready-to-go
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectData {
    pub project: Vec<u8>,

    pub file: Vec<u8>,
    pub logic: Vec<u8>,
    pub view: Vec<u8>,
    pub resource: Vec<u8>,
    pub library: Vec<u8>,
}

impl yew_agent::Worker for CompilerWorker {
    type Input = CompilerWorkerInput;
    type Message = ();
    type Output = CompilerWorkerOutput;
    type Reach = Public<Self>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {
        // no messaging
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        // resolve the swproj.toml file and parse it
        // invoke the compiler module to do the actual compilation

        self.link.respond(id, compiler::compile(msg.fs)
            .map(|raw| {
                // todo: feature to upload and set resources
                CompilerWorkerOutput::Success(ProjectData {
                    project: swrs::encrypt_sw(raw.project.as_bytes()),
                    file: swrs::encrypt_sw(raw.file.as_bytes()),
                    logic: swrs::encrypt_sw(raw.logic.as_bytes()),
                    view: swrs::encrypt_sw(raw.view.as_bytes()),
                    resource: swrs::encrypt_sw(raw.resource.as_bytes()),
                    library: swrs::encrypt_sw(raw.library.as_bytes()),
                })
            })
            .unwrap_or(CompilerWorkerOutput::Failure));
    }

    fn name_of_resource() -> &'static str {
        "compiler_worker.js"
    }
}