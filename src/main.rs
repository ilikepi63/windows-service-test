#[cfg(windows)]
#[tokio::main]
async fn main() -> windows_service::Result<()> {
    ping_service::run()
}

// TODO - add the unix based daemon as well
#[cfg(not(windows))]
fn main() {
    panic!("This program is only intended to run on Windows.");
}

#[cfg(windows)]
mod ping_service {
    use std::io::{Error, ErrorKind};
    use std::{ffi::OsString, time::Duration};
    use tokio::runtime::Runtime;
    use std::sync::mpsc;
    use tokio::time;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher, Result,
    };

    const SERVICE_NAME: &str = "test_service";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
    const URL: &str = "https://eolr5l8a0t401et.m.pipedream.net";

    pub fn run() -> Result<()> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    // generate boilerplate
    define_windows_service!(ffi_service_main, service_main);

    // entry point
    pub fn service_main(_arguments: Vec<OsString>) {
        if let Err(_e) = run_service() {
            // no stdout, log to file potentially?
        }
    }

    pub fn run_service() -> Result<()> {
        // Create a channel to be able to poll a stop event from the service worker loop.
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // Define system service event handler that will be receiving service events.
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // SCM check to see if the service is still healthy
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                ServiceControl::Stop =>  {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                },
                _ => ServiceControlHandlerResult::NotImplemented,

            }
        };

        // Register system service event handler.
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Tell the system that service is running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        let rt = match Runtime::new() {
            Ok(rt) => Ok(rt),
            Err(_err) => Err(windows_service::Error::Winapi(Error::new(
                ErrorKind::Other,
                "Could not create the runtime.",
            ))),
        }?;

        async fn await_shutdown(rx: mpsc::Receiver<()>) -> Result<()> {
            // polls the current rx, if a there was a shutdown receiver, this will then resolve the future
            loop{
                match rx.recv_timeout(Duration::from_millis(100)) {
                    // Break the loop either upon stop or channel disconnect
                    Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
    
                    // Continue work if no events were received within the timeout
                    Err(mpsc::RecvTimeoutError::Timeout) => (),
                };
            };

            Ok(())
        }


        async fn runtime() -> Result<()> {
            loop{
                time::sleep(time::Duration::from_millis(10000)).await;

                if let Ok(result) = reqwest::get(URL).await {
                    if let  Ok(_body) = result.text().await {
                    }
                }
            }
        }

        // Spawn the root task
        rt.block_on(async {

            let shutdown_join_handle = tokio::spawn(await_shutdown(shutdown_rx));
            let runtime_handle = tokio::spawn(runtime());


            tokio::select! {
                _ = shutdown_join_handle => {},
                _ = runtime_handle => {}
            }
        });

        // Tell the system that service has stopped.
        match status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        }) {
            Ok(_) => {},
            Err(_) => {}
        };
        
        Ok(())
    }
}
