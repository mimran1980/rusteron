use crate::{Aeron, AeronArchive, AeronArchiveAsyncConnect, AeronArchiveContext, AeronContext};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{fs, io};

pub struct EmbeddedArchiveMediaDriverProcess {
    child: Child,
    pub aeron_dir: String,
    pub archive_dir: String,
    pub request_control_channel: String,
    pub response_control_channel: String,
}

impl EmbeddedArchiveMediaDriverProcess {
    pub fn build_and_start(
        aeron_dir: &str,
        archive_dir: &str,
        request_control_channel: &str,
        response_control_channel: &str,
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
            request_control_channel,
            response_control_channel,
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
                                &self.request_control_channel,
                                &self.response_control_channel,
                            )
                        {
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
                        eprintln!("aeron error: {}", aeron.errmsg());
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

    pub fn start(
        aeron_dir: &str,
        archive_dir: &str,
        request_control_channel: &str,
        response_control_channel: &str,
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
                request_control_channel
            ),
            "-Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0",
            "-Daeron.archive.control.response.channel=aeron:udp?endpoint=localhost:0",
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
            request_control_channel: request_control_channel.to_string(),
            response_control_channel: response_control_channel.to_string(),
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
}

// Use the Drop trait to ensure process cleanup and directory removal after test completion
impl Drop for EmbeddedArchiveMediaDriverProcess {
    fn drop(&mut self) {
        eprintln!("WARN: stopping aeron archive media driver!!!!");
        // Attempt to kill the Java process if it’s still running
        if let Err(e) = self.child.kill() {
            eprintln!("Failed to kill Java process: {}", e);
        }

        // Clean up directories after the process has terminated
        if let Err(e) = Self::clean_directory(&self.aeron_dir) {
            eprintln!("Failed to clean up Aeron directory: {}", e);
        }
        if let Err(e) = Self::clean_directory(&self.archive_dir) {
            eprintln!("Failed to clean up Archive directory: {}", e);
        }
    }
}
