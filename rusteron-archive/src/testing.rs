use crate::{
    Aeron, AeronArchive, AeronArchiveAsyncConnect, AeronArchiveContext,
    AeronArchiveRecordingSignal, AeronContext, AeronIdleStrategyFuncClosure, Handler,
};
use log::{error, warn};
use regex::Regex;
use std::backtrace::Backtrace;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{fs, io, panic, process};

pub struct EmbeddedArchiveMediaDriverProcess {
    child: Child,
    pub aeron_dir: String,
    pub archive_dir: String,
    pub control_request_channel: String,
    pub control_response_channel: String,
    pub recording_events_channel: String,
}

impl EmbeddedArchiveMediaDriverProcess {
    /// Builds the Aeron Archive project and starts an embedded Aeron Archive Media Driver process.
    ///
    /// This function ensures that the necessary Aeron `.jar` files are built using Gradle. If the required
    /// `.jar` files are not found in the expected directory, it runs the Gradle build tasks to generate them.
    /// Once the build is complete, it invokes the `start` function to initialize and run the Aeron Archive Media Driver.
    ///
    /// # Parameters
    /// - `aeron_dir`: The directory for the Aeron media driver to use for its IPC mechanisms.
    /// - `archive_dir`: The directory where the Aeron Archive will store its recordings and metadata.
    /// - `control_request_channel`: The channel URI used for sending control requests to the Aeron Archive.
    /// - `control_response_channel`: The channel URI used for receiving control responses from the Aeron Archive.
    /// - `recording_events_channel`: The channel URI used for receiving recording event notifications from the Aeron Archive.
    ///
    /// # Returns
    /// On success, returns an instance of `EmbeddedArchiveMediaDriverProcess` encapsulating the child process
    /// and configuration used. Returns an `io::Result` if the process fails to start or the build fails.
    ///
    /// # Errors
    /// Returns an `io::Result::Err` if:
    /// - The Gradle build fails to execute or complete.
    /// - The required `.jar` files are still not found after building.
    /// - The `start` function encounters an error starting the process.
    ///
    /// # Example
    /// ```
    /// use rusteron_archive::testing::EmbeddedArchiveMediaDriverProcess;
    /// let driver = EmbeddedArchiveMediaDriverProcess::build_and_start(
    ///     "/tmp/aeron-dir",
    ///     "/tmp/archive-dir",
    ///     "aeron:udp?endpoint=localhost:8010",
    ///     "aeron:udp?endpoint=localhost:8011",
    ///     "aeron:udp?endpoint=localhost:8012",
    /// ).expect("Failed to build and start Aeron Archive Media Driver");
    /// ```
    ///
    /// # Notes
    /// - This function assumes the presence of a Gradle wrapper script (`gradlew` or `gradlew.bat`)
    ///   in the `aeron` directory relative to the project's root (`CARGO_MANIFEST_DIR`).
    /// - The required `.jar` files will be generated in `aeron/aeron-all/build/libs` if not already present.
    /// - The `build_and_start` function is a convenience wrapper for automating the build and initialization process.
    pub fn build_and_start(
        aeron_dir: &str,
        archive_dir: &str,
        control_request_channel: &str,
        control_response_channel: &str,
        recording_events_channel: &str,
    ) -> io::Result<Self> {
        let path = std::path::MAIN_SEPARATOR;
        let gradle = if cfg!(target_os = "windows") {
            &format!("{}{path}aeron{path}gradlew.bat", env!("CARGO_MANIFEST_DIR"),)
        } else {
            "./gradlew"
        };
        let dir = format!("{}{path}aeron", env!("CARGO_MANIFEST_DIR"),);
        println!("running {} in {}", gradle, dir);

        if !Path::new(&dir).join("aeron-all/build/libs").exists() {
            Command::new(&gradle)
                .current_dir(dir)
                .args([
                    ":aeron-agent:jar",
                    ":aeron-samples:jar",
                    ":aeron-archive:jar",
                    ":aeron-all:build",
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
                .wait()?;
        }

        return Self::start(
            &aeron_dir,
            archive_dir,
            control_request_channel,
            control_response_channel,
            recording_events_channel,
        );
    }

    pub fn archive_connect(&self) -> Result<(AeronArchive, Aeron), io::Error> {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(30) {
            if let Ok(aeron_context) = AeronContext::new() {
                aeron_context.set_dir(&self.aeron_dir).expect("invalid dir");
                aeron_context
                    .set_client_name("unit_test_client")
                    .expect("invalid client name");
                if let Ok(aeron) = Aeron::new(&aeron_context) {
                    if aeron.start().is_ok() {
                        if let Ok(archive_context) =
                            AeronArchiveContext::new_with_no_credentials_supplier(
                                &aeron,
                                &self.control_request_channel,
                                &self.control_response_channel,
                                &self.recording_events_channel,
                            )
                        {
                            let signal_consumer = Handler::leak(
                                crate::AeronArchiveRecordingSignalConsumerFuncClosure::from(
                                    |signal: AeronArchiveRecordingSignal| {
                                        println!("Recording signal received: {:?}", signal);
                                    },
                                ),
                            );
                            archive_context
                                .set_recording_signal_consumer(Some(&signal_consumer))
                                .expect("Failed to set recording signal consumer");
                            let error_handler = Handler::leak(
                                crate::AeronErrorHandlerClosure::from(|code, msg| {
                                    error!("err code: {}, msg: {}", code, msg);
                                }),
                            );
                            archive_context
                                .set_error_handler(Some(&error_handler))
                                .expect("unable to set error handler");
                            archive_context
                                .set_idle_strategy(Some(&Handler::leak(
                                    AeronIdleStrategyFuncClosure::from(|work_count| {}),
                                )))
                                .expect("unable to set idle strategy");
                            if let Ok(connect) = AeronArchiveAsyncConnect::new(&archive_context) {
                                if let Ok(archive) = connect.poll_blocking(Duration::from_secs(10))
                                {
                                    let i = archive.get_archive_id();
                                    assert!(i > 0);
                                    println!("aeron archive media driver is up [connected with archive id {i}]");
                                    sleep(Duration::from_millis(100));
                                    return Ok((archive, aeron));
                                };
                            }
                        }
                        error!("aeron error: {}", aeron.errmsg());
                    }
                }
            }
            println!("waiting for aeron to start up aeron");
        }

        assert!(
            start.elapsed() < Duration::from_secs(30),
            "failed to start up aeron media driver"
        );

        return Err(std::io::Error::other(
            "unable to start up aeron media driver client",
        ));
    }

    /// Starts an embedded Aeron Archive Media Driver process with the specified configurations.
    ///
    /// This function cleans and recreates the Aeron and archive directories, configures the JVM to run
    /// the Aeron Archive Media Driver, and starts the process with the specified control channels.
    /// It ensures that the environment is correctly prepared for Aeron communication.
    ///
    /// # Parameters
    /// - `aeron_dir`: The directory for the Aeron media driver to use for its IPC mechanisms.
    /// - `archive_dir`: The directory where the Aeron Archive will store its recordings and metadata.
    /// - `control_request_channel`: The channel URI used for sending control requests to the Aeron Archive.
    /// - `control_response_channel`: The channel URI used for receiving control responses from the Aeron Archive.
    /// - `recording_events_channel`: The channel URI used for receiving recording event notifications from the Aeron Archive.
    ///
    /// # Returns
    /// On success, returns an instance of `EmbeddedArchiveMediaDriverProcess` encapsulating the child process
    /// and configuration used. Returns an `io::Result` if the process fails to start.
    ///
    /// # Errors
    /// Returns an `io::Result::Err` if:
    /// - Cleaning or creating the directories fails.
    /// - The required `.jar` files are missing or not found.
    /// - The Java process fails to start.
    ///
    /// # Example
    /// ```
    /// use rusteron_archive::testing::EmbeddedArchiveMediaDriverProcess;
    /// let driver = EmbeddedArchiveMediaDriverProcess::start(
    ///     "/tmp/aeron-dir",
    ///     "/tmp/archive-dir",
    ///     "aeron:udp?endpoint=localhost:8010",
    ///     "aeron:udp?endpoint=localhost:8011",
    ///     "aeron:udp?endpoint=localhost:8012",
    /// ).expect("Failed to start Aeron Archive Media Driver");
    /// ```
    ///
    /// # Notes
    /// - The Aeron `.jar` files must be available under the directory `aeron/aeron-all/build/libs` relative
    ///   to the project's root (`CARGO_MANIFEST_DIR`).
    /// - The function configures the JVM with properties for Aeron, such as enabling event logging and disabling bounds checks.
    pub fn start(
        aeron_dir: &str,
        archive_dir: &str,
        control_request_channel: &str,
        control_response_channel: &str,
        recording_events_channel: &str,
    ) -> io::Result<Self> {
        Self::clean_directory(aeron_dir)?;
        Self::clean_directory(archive_dir)?;

        // Ensure directories are recreated
        fs::create_dir_all(aeron_dir)?;
        fs::create_dir_all(archive_dir)?;

        let binding = fs::read_dir(format!(
            "{}/aeron/aeron-all/build/libs",
            env!("CARGO_MANIFEST_DIR")
        ))?
        .filter(|f| f.is_ok())
        .map(|f| f.unwrap())
        .filter(|f| {
            f.file_name()
                .to_string_lossy()
                .to_string()
                .ends_with(".jar")
        })
        .next()
        .unwrap()
        .path();
        let mut jar_path = binding.to_str().unwrap();
        let mut agent_jar = jar_path.replace("aeron-all", "aeron-agent");

        assert!(fs::exists(jar_path).unwrap_or_default());
        if fs::exists(&agent_jar).unwrap_or_default() {
            agent_jar = format!("-javaagent:{}", agent_jar);
        } else {
            agent_jar = " ".to_string();
        }
        let separator = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };

        let combined_jars = format!(
            "{}{separator}{}",
            jar_path,
            jar_path.replace("aeron-all", "aeron-archive")
        );
        jar_path = &combined_jars;

        let args = [
            agent_jar.as_str(),
            "--add-opens",
            "java.base/jdk.internal.misc=ALL-UNNAMED",
            "-cp",
            jar_path,
            &format!("-Daeron.dir={}", aeron_dir),
            &format!("-Daeron.archive.dir={}", archive_dir),
            "-Daeron.spies.simulate.connection=true",
            // "-Daeron.event.log=all", // this will only work if agent is built
            "-Daeron.event.archive.log=all",
            // "-Daeron.event.cluster.log=all",
            // "-Daeron.term.buffer.sparse.file=false",
            // "-Daeron.pre.touch.mapped.memory=true",
            // "-Daeron.threading.mode=DEDICATED",
            // "-Daeron.sender.idle.strategy=noop",
            // "-Daeron.receiver.idle.strategy=noop",
            // "-Daeron.conductor.idle.strategy=spin",
            "-Dagrona.disable.bounds.checks=true",
            &format!(
                "-Daeron.archive.control.channel={}",
                control_request_channel
            ),
            &format!(
                "-Daeron.archive.control.response.channel={}",
                control_response_channel
            ),
            &format!(
                "-Daeron.archive.recording.events.channel={}",
                recording_events_channel
            ),
            "-Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0",
            "io.aeron.archive.ArchivingMediaDriver",
        ];

        println!(
            "starting archive media driver [\n\tjava {}\n]",
            args.join(" ")
        );

        let child = Command::new("java")
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        println!(
            "started archive media driver [{:?}",
            fs::read_dir(aeron_dir)?.collect::<Vec<_>>()
        );

        Ok(EmbeddedArchiveMediaDriverProcess {
            child,
            aeron_dir: aeron_dir.to_string(),
            archive_dir: archive_dir.to_string(),
            control_request_channel: control_request_channel.to_string(),
            control_response_channel: control_response_channel.to_string(),
            recording_events_channel: recording_events_channel.to_string(),
        })
    }

    fn clean_directory(dir: &str) -> io::Result<()> {
        println!("cleaning directory {}", dir);
        let path = Path::new(dir);
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    pub fn kill_all_java_processes() -> io::Result<ExitStatus> {
        if cfg!(not(target_os = "windows")) {
            return Ok(std::process::Command::new("pkill")
                .args(["-9", "java"])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
                .wait()?);
        }
        Ok(ExitStatus::default())
    }
}

// Use the Drop trait to ensure process cleanup and directory removal after test completion
impl Drop for EmbeddedArchiveMediaDriverProcess {
    fn drop(&mut self) {
        warn!("WARN: stopping aeron archive media driver!!!!");
        // Attempt to kill the Java process if it’s still running
        if let Err(e) = self.child.kill() {
            error!("Failed to kill Java process: {}", e);
        }

        // Clean up directories after the process has terminated
        if let Err(e) = Self::clean_directory(&self.aeron_dir) {
            error!("Failed to clean up Aeron directory: {}", e);
        }
        if let Err(e) = Self::clean_directory(&self.archive_dir) {
            error!("Failed to clean up Archive directory: {}", e);
        }
    }
}

pub fn set_panic_hook() {
    panic::set_hook(Box::new(|info| {
        // Get the backtrace
        let backtrace = Backtrace::force_capture();
        error!("Stack trace: {backtrace:#?}");

        let backtrace = format!("{:?}", backtrace);
        // Regular expression to match the function, file, and line
        let re = Regex::new(r#"fn: "([^"]+)", file: "([^"]+)", line: (\d+)"#).unwrap();

        // Extract and print in IntelliJ format with function
        for cap in re.captures_iter(&backtrace) {
            let function = &cap[1];
            let file = &cap[2];
            let line = &cap[3];
            println!("{file}:{line} in {function}");
        }

        error!("Panic occurred: {:#?}", info);

        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            error!("Panic message: {}", payload);
        } else if let Some(payload) = info.payload().downcast_ref::<String>() {
            error!("Panic message: {}", payload);
        } else {
            // If it's not a &str or String, try to print it as Debug
            error!(
                "Panic with non-standard payload: {:?}",
                info.payload().type_id()
            );
        }

        warn!("shutdown");

        process::abort();
    }))
}
