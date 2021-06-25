use std::net::{TcpListener, Shutdown};
use std::io::{Read, Write};
use windows_service::{define_windows_service, service_dispatcher, service_control_handler};
use std::ffi::OsString;
use windows_service::service_control_handler::ServiceControlHandlerResult;
use windows_service::service::{ServiceControl, ServiceStatus, ServiceState, ServiceType, ServiceControlAccept, ServiceExitCode};
use winapi::_core::time::Duration;
use std::convert::{TryFrom};

define_windows_service!(ffi_service_main, service_main);

#[repr(u8)]
enum Command {
    Ping = 0,
    Hibernate = 1,
}

impl TryFrom<u8> for Command {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Command::Ping),
            1 => Ok(Command::Hibernate),
            _ => Err(())
        }
    }
}

fn rcd_mainloop() {
    let password = "hahahaha"; // std::env::var("RCD_PASSWORD").expect("RCD_PASSWORD env var not present");
    if password.len() != 32 {
        panic!("RCD_PASSWORD exists but does not have required length = 32 bytes")
    }

    let local = TcpListener::bind("0.0.0.0:7305").expect("cannot start tcp server");

    loop {
        let (mut sock, _) = local.accept().expect("cannot accept new connection");

        /* auth */
        let mut buff_psw = [0u8; 32];
        if let Err(_) = sock.read_exact(&mut buff_psw) {
            let _ = sock.shutdown(Shutdown::Both);
            continue;
        }

        if buff_psw != password.as_bytes() {
            let _ = sock.write("bad auth".as_bytes());
            let _ = sock.shutdown(Shutdown::Both);
            continue;
        }

        /* dispatch cmd */
        let mut buff_cmd = [0u8];
        if let Err(_) = sock.read_exact(&mut buff_cmd) {
            let _ = sock.shutdown(Shutdown::Both);
            continue;
        }

        match Command::try_from(buff_cmd[0]) {
            Ok(Command::Ping) => {
                let _ = sock.write("pong".as_bytes());
            }
            Ok(Command::Hibernate) => {
                let _ = sock.write("ok".as_bytes());
                let _ = sock.shutdown(Shutdown::Both);

                /* https://docs.microsoft.com/sk-sk/windows/win32/api/powrprof/nf-powrprof-setsuspendstate */
                unsafe { winapi::um::powrprof::SetSuspendState(1, 0, 0) };

                continue;
            }
            _ => {
                let _ = sock.write("bad cmd".as_bytes());
            }
        }

        let _ = sock.shutdown(Shutdown::Both);
    }
}

fn main() -> Result<(), windows_service::Error> {
    service_dispatcher::start("rcd", ffi_service_main)?;
    Ok(())
}

fn service_main(arguments: Vec<OsString>) {
    if let Err(_e) = run_service(arguments) {
        // Handle errors in some way.
    }
}

fn run_service(_arguments: Vec<OsString>) -> Result<(), windows_service::Error> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register("rcd", event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
    })?;

    rcd_mainloop();

    Ok(())
}