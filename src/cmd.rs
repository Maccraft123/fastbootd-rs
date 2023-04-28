use anyhow::{Context, Result};

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
use std::io::Read;
use crate::{read_usb, FbReply};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static PREV_DATA: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(|| Mutex::new(None));

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
                let mut data = PREV_DATA.lock().unwrap().take();
                if data.is_none() {
                    FbReply::Fail("No data sent for writing").send(ep_in)?;
                    return Ok(());
                }
                FbReply::Info("Flash writing is todo").send(ep_in)?;
                FbReply::Okay("").send(ep_in)?;
            },
            _ => todo!(),
        }
        Ok(())
    }
}
