#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{borrow::BorrowMut, cell::RefCell, error::Error, ffi::CString, io::Write, os::unix::prelude::FromRawFd, sync::{Arc, Mutex}};

use nix::{
  ioctl_none_bad,
  libc::{close, TIOCSCTTY},
  pty::openpty,
  unistd::{dup2, execvpe, fork, setsid, ForkResult},
};

ioctl_none_bad!(tiocsctty, TIOCSCTTY);

use tauri::Manager;

fn main() -> Result<(), Box<dyn Error>> {
  let pty_res = openpty(None, None)?;

  let fork_res = unsafe { fork()? };

  match fork_res {
    ForkResult::Parent { child: _ } => {
      unsafe { close(pty_res.slave) };

      let master = Arc::new(Mutex::new(unsafe { std::fs::File::from_raw_fd(pty_res.master) }));

      tauri::Builder::default()
      .setup(move |app| {
        let master_clone = Arc::clone(&master);
        app.listen_global("data", move |event| {
          println!("got event-name with payload {:?}", event.payload());
          if let Some(data) = event.payload() {
            let master_clone = Arc::clone(&master_clone);
            let mut master_unlock = master_clone.lock().unwrap();
            master_unlock.borrow_mut().write(data.as_bytes()).unwrap();
          }
        });
  
        Ok(())
      })
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
    }
    ForkResult::Child => {
      unsafe { close(pty_res.master) };

      setsid()?;

      unsafe { tiocsctty(pty_res.slave)? };

      dup2(pty_res.slave, 0)?;
      dup2(pty_res.slave, 1)?;
      dup2(pty_res.slave, 2)?;

      unsafe { close(pty_res.slave) };

      execvpe::<CString, CString>(&CString::new("/bin/sh")?, &[], &[])?;
    }
  }

  Ok(())
}
