use anyhow::{Context, Result};

pub fn parse(cmd: &str, param: Option<String>) -> Result<FastbootCmd, &'static str> {
    use FastbootCmd::*;
    match cmd {
        "getvar" => Ok(Getvar(param.ok_or("Missing parameter")?)),
        _ => Err("Unknown command"),
    }
}

#[derive(Clone, Debug)]
pub enum FastbootCmd {
    Getvar(String),
    Download(u32),
    Upload,
    Flash(String),
    Erase(String),
    Boot,
    Continue,
    Reboot,
    RebootBootloader,
}

use std::fs::File;
use crate::FbReply;

impl FastbootCmd {
    pub fn run(self, ep_in: &mut File, ep_out: &mut File) -> Result<()> {
        use FastbootCmd::*;
        match self {
            Getvar(which) => {
                match which.as_str() {
                    "version" => FbReply::Okay("0.4"),
                    "product" => FbReply::Okay("fastbootd-rs"),
                    "secure" => FbReply::Okay("no"),
                    "is-userspace" => FbReply::Okay("yes"),
                    _ => FbReply::Fail("unknown variable"),
                }.send(ep_in)?;
            },
            _ => todo!(),
        }
        Ok(())
    }
}
