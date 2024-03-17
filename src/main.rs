use std::{os::fd::AsFd, process::Command, ffi::OsStr};

use nix::poll::{PollFd, PollFlags, PollTimeout};

extern crate udev;

fn main() -> anyhow::Result<()> {
	let mut enumerator = udev::Enumerator::new().unwrap();

	enumerator.match_subsystem("usb")?;
	enumerator.match_property("DEVTYPE", "usb_device")?;
	enumerator.match_is_initialized()?;

	for device in enumerator.scan_devices().unwrap() {
		let parent = device.parent();
		if let Some(parent) = parent {
			if matches!(parent.subsystem().map(|item| item.to_str().unwrap_or("")), Some("platform")) {
				continue;
			}
		}
		connect_device(device.sysname()).ok();
	}

	let enumerator = udev::MonitorBuilder::new()?
		.match_subsystem_devtype("usb", "usb_device")?
		.listen()?;

	loop {
		for device in enumerator.iter() {
			if device.event_type() == udev::EventType::Add {
				connect_device(device.sysname()).ok();
			}
		}
		nix::poll::poll(&mut [PollFd::new(enumerator.as_fd(), PollFlags::POLLIN)], PollTimeout::NONE)?;
	}
}

fn connect_device(bus_id: &OsStr) -> anyhow::Result<()> {
	println!("found device: {:?}", bus_id);
	let mut usbip = Command::new("/sbin/usbip").arg("bind").arg("-b").arg(bus_id).spawn()?;
	let result = usbip.wait()?;
	if !result.success() {
		anyhow::bail!("usbip command failed");
	}
	let mut ssh = Command::new("ssh").args(["10.0.0.10", "usbip", "attach", "-r", "10.0.0.52", "-d"]).arg(bus_id).spawn()?;
	let result = ssh.wait()?;
	if !result.success() {
		anyhow::bail!("ssh command failed");
	}
	return Ok(());
}

// fn main() {
// 	let mut enumerator = udev::Enumerator::new().unwrap();

// 	enumerator.match_subsystem("usb").unwrap();

// 	for device in enumerator.scan_devices().unwrap() {
// 		println!("found device: {:?}", device.syspath());
// 	}
// }

