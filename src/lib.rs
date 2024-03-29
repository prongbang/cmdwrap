use std::io;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use futures_core::stream::Stream;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Payload {
    pub success: bool,
    pub output: String,
}

/// run
///
/// # Arguments
///
/// * `command`:
///
/// returns: Result<String, String>
///
/// # Examples
///
/// ```
/// let command = "pwd";
/// match cmdwrap::run(command) {
///     Ok(output) => {
///         println!("{}", output)
///     }
///     Err(error) => {
///         println!("\tCommand execution failed:\n{}", error);
///     }
/// }
/// ```
#[allow(dead_code)]
pub fn run(command: &str) -> Result<String, String> {
    // Execute the command in the shell
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|err| format!("Failed to execute the command: {}", err))?;

    // Check if the command executed successfully
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).map_err(|err| format!("Failed to convert stdout to UTF-8: {}", err))?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8(output.stderr).map_err(|err| format!("Failed to convert stderr to UTF-8: {}", err))?;
        Err(stderr)
    }
}

#[cfg(unix)]
fn run_command(command: &str) -> io::Result<std::process::Child> {
    Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

#[cfg(windows)]
fn run_command(command: &str) -> io::Result<std::process::Child> {
    Command::new("cmd")
        .args(&["/C", &command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// run_stream
///
/// # Arguments
///
/// * `command`:
///
/// returns: impl Stream<Item=Payload>+Sized
///
/// # Examples
///
/// ```
/// use futures_util::pin_mut;
/// use futures_util::stream::StreamExt;
///
/// let command = "pwd";
/// let mut s = cmdwrap::run_stream(command);
/// pin_mut!(s); // needed for iteration
/// while let Some(value) = s.next().await {
///     println!("{}", value.output);
/// }
/// ```
pub fn run_stream<'a>(command: &'a str) -> impl Stream<Item=Payload> + 'a {
    async_stream::stream! {
        // Execute the command in the shell
        let cmd = run_command(command);

        match cmd {
            Ok(cmd) => {

                let cmd = Arc::new(Mutex::new(cmd));
                let stdout = cmd.as_ref().lock().unwrap().stdout.take(); // Take ownership of stdout
                match stdout {
                    Some(stdout) => {
                        let reader = BufReader::new(stdout);

                        for line in reader.lines() {
                            let line = line.unwrap();
                            yield Payload{ success: false, output: line };
                        }

                        // Check if the command executed successfully
                        let command_status = cmd.as_ref().lock().unwrap().wait();
                        match command_status {
                            Ok(_) => {
                                yield Payload{ success: true, output: "".to_string() };
                            }
                            Err(e) => {
                                yield Payload{ success: false, output: e.to_string() };
                            }
                        }
                    }
                    None => {
                        yield Payload{ success: false, output: "".to_string() };
                    }
                }
            }
            Err(e) => {
                yield Payload{ success: false, output: e.to_string() };
            }
        }
    }
}