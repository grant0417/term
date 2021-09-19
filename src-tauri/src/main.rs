#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{
  borrow::BorrowMut,
  env::vars,
  error::Error,
  ffi::CString,
  fs::File,
  io::{BufReader, Read, Write},
  os::unix::prelude::FromRawFd,
  sync::{Arc, Mutex},
};

use nix::{
  ioctl_none_bad, ioctl_write_ptr_bad,
  libc::{close, TIOCSCTTY, TIOCSWINSZ},
  pty::{openpty, Winsize},
  unistd::{dup2, execvpe, fork, setsid, ForkResult},
};

#[derive(serde::Deserialize)]
struct ResizePayload {
  rows: u16,
  cols: u16,
}

ioctl_none_bad!(tiocsctty, TIOCSCTTY);
ioctl_write_ptr_bad!(tiocswinsz, TIOCSWINSZ, Winsize);

use tauri::{Manager, Window};

fn send_output(window: Arc<Window>, file_fd: i32) {
  std::thread::spawn(move || {
    let file = unsafe { File::from_raw_fd(file_fd) };
    let mut buff_reader = BufReader::new(file);
    loop {
      let mut buffer = [0; 1];
      match buff_reader.read(&mut buffer) {
        Ok(_) => {
          window.emit("write", buffer[0]).unwrap();
        }
        Err(_) => {
          window.emit("close", "").unwrap();
          break;
        }
      }
    }
  });
}

fn main() -> Result<(), Box<dyn Error>> {
  let pty_res = openpty(None, None)?;

  let fork_res = unsafe { fork()? };

  match fork_res {
    ForkResult::Parent { child: _ } => {
      unsafe { close(pty_res.slave) };
      

      tauri::Builder::default()
        .setup(move |app| {
          let main_window = Arc::new(app.get_window("main").unwrap());
          let master = Arc::new(Mutex::new(unsafe { File::from_raw_fd(pty_res.master) }));

          app.listen_global("data", move |event| {
            if let Some(data) = event.payload() {
              let master_clone = Arc::clone(&master);
              master_clone
                .lock()
                .unwrap()
                .borrow_mut()
                .write(data.as_bytes())
                .unwrap();
            }
          });

          app.listen_global("ready", move |_| {
            send_output(Arc::clone(&main_window), pty_res.master);
          });

          app.listen_global("resize", move |event| {
            if let Some(data) = event.payload() {
              let resize_payload = serde_json::from_str::<ResizePayload>(&data).unwrap();
              let winsize = Winsize {
                ws_row: resize_payload.rows,
                ws_col: resize_payload.cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
              };

              unsafe {
                tiocswinsz(pty_res.master, &winsize).unwrap();
              }
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

      // let usr = User::from_uid(getuid()).unwrap_or(User::from_name("root").unwrap()).unwrap();

      let sh = CString::new("/bin/bash")?;

      let mut env_vars = vars()
        .filter(|(key, _)| {
          key != "TERM" && key != "TERM_PROGRAM" && key != "TERM_PROGRAM_VERSION" && key != "SHELL"
        })
        .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap())
        .collect::<Vec<_>>();

      env_vars.push(CString::new("TERM=xterm-256color")?);
      env_vars.push(CString::new("TERM_PROGRAM=term")?);
      env_vars.push(CString::new("TERM_PROGRAM_VERSION=0.1")?);
      env_vars.push(CString::new("SHELL=bash")?);

      execvpe::<CString, CString>(&sh, &[], &env_vars)?;
    }
  }

  Ok(())
}
