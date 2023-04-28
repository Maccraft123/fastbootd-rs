use anyhow::Result;

pub fn parse(cmd: &str, param: Option<String>) -> Result<FastbootCmd, &'static str> {
    use FastbootCmd::*;
    match cmd {
        "download" => {
            let size_str = param.ok_or("Missing size parameter")?;
            let size: u32 = u32::from_str_radix(&size_str, 16)
                .map_err(|_| "Failed to parse size as u32")?;

            Ok(Download(size))
        },
        "getvar" => Ok(Getvar(param.ok_or("Missing parameter")?)),
        "flash" => Ok(Flash(param.ok_or("Missing parameter")?)),
        "reboot" => Ok(Reboot),
        "boot" => Ok(Boot),
        "continue" => Ok(Continue),
        _ => Err("Unknown command"),
    }
}

#[derive(Clone, Debug)]
pub enum FastbootCmd {
    Getvar(String),
    Download(u32),
    Flash(String),
    Erase(String),
    Boot,
    Continue,
    Reboot,
}

use std::fs::File;
use crate::{read_usb, FbReply};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::path::PathBuf;
use crate::NextAction;
use std::env;

static PREV_DATA: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(|| Mutex::new(None));

fn partition_path(name: &str) -> Option<PathBuf> {
    let tmp = if env::var("FBRS_PART_PATH").is_ok() &&
        PathBuf::from(env::var("FBRS_PART_PATH").unwrap()).exists()
    {
        Some(PathBuf::from(env::var("FBRS_PART_PATH").unwrap()))
    } else if PathBuf::from("/dev/disk/by-partlabel/").exists() {
        Some(PathBuf::from("/dev/disk/by-partlabel/"))
    } else if PathBuf::from("/dev/block/by-name/").exists() {
        Some(PathBuf::from("/dev/block/by-name/"))
    } else {
        None
    };
    tmp.map(|v| v.join(name)).filter(|v| v.exists())
}

impl FastbootCmd {
    pub fn run(self, ep_in: &mut File, ep_out: &mut File) -> Result<Option<NextAction>> {
        use FastbootCmd::*;
        match self {
            Getvar(which) => {
                match which.as_str() {
                    "version" => FbReply::Okay("0.4"),
                    "product" => FbReply::Okay("fastbootd-rs"),
                    "secure" => FbReply::Okay("no"),
                    "is-userspace" => FbReply::Okay("yes"),
                    "max-download-size" => FbReply::Okay("32768000"),
                    _ => FbReply::Fail("unknown variable"),
                }.send(ep_in)?;
            },
            Download(size) => {
                FbReply::Data(size).send(ep_in)?;

                let mut buf = Vec::with_capacity(size as usize);
                let mut read = 0;
                while read < size {
                    let mut tmp = read_usb(ep_out, 3276800)?;
                    read += tmp.len() as u32;
                    buf.append(&mut tmp);
                }

                let mut data = PREV_DATA.lock().unwrap();
                if data.is_some() {
                    FbReply::Info("Overwriting previously sent data");
                }

                let _ = data.insert(buf);

                FbReply::Okay("").send(ep_in)?;
            },
            Flash(what) => {
                let Some(data) = PREV_DATA.lock().unwrap().take() else {
                    FbReply::Fail("No data sent for writing").send(ep_in)?;
                    return Ok(None);
                };

                let Some(path) = partition_path(&what) else {
                    FbReply::Fail("Couldn't find target partition").send(ep_in)?;
                    return Ok(None);
                };

                std::fs::write(path, data)?;

                FbReply::Okay("").send(ep_in)?;
            },
            Reboot => {
                FbReply::Okay("").send(ep_in)?;
                return Ok(Some(NextAction::Reboot));
            },
            Continue => {
                FbReply::Okay("").send(ep_in)?;
                return Ok(Some(NextAction::Continue));
            },
            Boot => {
                FbReply::Fail("todo").send(ep_in)?;
            }
            _ => todo!(),
        }
        Ok(None)
    }
}
