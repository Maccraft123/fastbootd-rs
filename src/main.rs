mod usb;
use usb::FASTBOOT_DESCRIPTOR_V2;
use usb::FASTBOOT_STRINGS;

mod cmd;
use cmd::FastbootCmd;

use anyhow::{Context, Result};

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use std::os::fd::AsRawFd;
use nix::errno;

fn next_cmd(input: &mut File) -> Result<Result<FastbootCmd, &'static str>> {
    let mut tmp: Vec<u8> = Vec::with_capacity(64);
    unsafe {
        let ret = libc::read(input.as_raw_fd(),
                tmp.as_mut_ptr().cast(),
                64);

        if ret < 0 {
            return Err(errno::from_i32(nix::errno::errno()).into());
        }

        tmp.set_len(ret as usize);
    }

    let raw = String::from_utf8_lossy(&tmp);
    let mut split = raw.split(':');

    let cmd = split.next().unwrap();
    let param = split.next().map(|v| v.to_string());

    Ok(cmd::parse(cmd, param))
}

#[derive(Clone, Debug)]
pub enum FbReply<'a> {
    Info(&'a str),
    Text(&'a str),
    Fail(&'a str),
    Okay(&'a str),
    Data(u32),
}

impl FbReply<'_> {
    fn to_bytes(self) -> Vec<u8> {
        use FbReply::*;
        let string = match self {
            Info(v) => format!("INFO{v}"),
            Text(v) => format!("TEXT{v}"),
            Fail(v) => format!("FAIL{v}"),
            Okay(v) => format!("OKAY{v}"),
            Data(v) => format!("DATA{v:x}"),
        };
        
        let mut ret: Vec<u8> = string.into();
        ret.truncate(256);
        ret
    }
    pub fn send(self, ep_in: &mut File) -> Result<()> {
        ep_in.write_all(&self.to_bytes());
        Ok(())
    }
}

fn main() -> Result<()> {
    let endpoint_path = PathBuf::from(env::args().skip(1).next()
        .expect("First argument has to be functionfs path"));

    let mut ep_control = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(endpoint_path.join("ep0"))
        .context("Failed to open ep0")?;

    ep_control.write_all(FASTBOOT_DESCRIPTOR_V2.as_bytes())?;
    ep_control.write_all(FASTBOOT_STRINGS.as_bytes())?;


    let mut ep_out = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(endpoint_path.join("ep1"))
        .context("Failed to open ep1")?;

    let mut ep_in = OpenOptions::new()
        .read(false)
        .write(true)
        .create(false)
        .open(endpoint_path.join("ep2"))
        .context("Failed to open ep2")?;

    loop {
        let cmd = next_cmd(&mut ep_out)
            .context("Failed to read next command")?;
        println!("{:?}", &cmd);
        match cmd {
            Ok(cmd) => {
                match cmd.run(&mut ep_in, &mut ep_out) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("Failed to run fastboot command: {}, due to:", e);
                        for cause in e.chain() {
                            eprintln!("{}", cause);
                        }
                        ep_in.write_all(&FbReply::Fail("See screen").to_bytes())?;
                    },
                }
            },
            Err(e) => {
                ep_in.write_all(&FbReply::Fail(e).to_bytes())?;
            }
        }

    }
}
