use bollard::{
    container::{
        Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions,
        StartContainerOptions, UploadToContainerOptions, WaitContainerOptions,
    },
    models::HostConfig,
    Docker,
};

use bytes::Bytes;
use futures::StreamExt;
use uuid::Uuid;

use self::{error::Error, languages::Languages};

pub mod error;
pub mod languages;

pub struct Container {
    container_id: Uuid,
    internal_id: Option<String>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            container_id: Uuid::new_v4(),
            internal_id: None,
        }
    }

    pub async fn create(&mut self, docker: &Docker) -> Result<(), Box<dyn std::error::Error>> {
        // Create container
        let options = Some(CreateContainerOptions {
            name: self.container_id.to_string(),
        });

        let config = Config {
            image: Some("container-runtime"),
            cmd: Some(vec!["./container-runtime"]),
            network_disabled: Some(true),

            // This step is important as it ensures that the code is sandboxed with gVisor
            host_config: Some(HostConfig {
                #[cfg(target_os = "linux")]
                runtime: Some("runsc".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let res = docker.create_container(options, config).await?;
        self.internal_id = Some(res.id);

        Ok(())
    }

    pub async fn upload_code(
        &self,
        languages: Languages,
        code: &[u8],
        stdin: &[u8],
        docker: &Docker,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Upload code to compile
        {
            // Assign a GNU header
            let mut header = tar::Header::new_gnu();
            header.set_size(code.len() as u64);
            header.set_cksum();

            // Tar the file
            let mut output = Vec::new();
            {
                let mut tar = tar::Builder::new(&mut output);
                tar.append_data(&mut header, languages.get_filename(), code)
                    .unwrap();
                tar.finish().unwrap();
            }

            let options = Some(UploadToContainerOptions {
                path: "/tmp",
                ..Default::default()
            });

            docker
                .upload_to_container(&self.container_id.to_string(), options, output.into())
                .await?;
        }

        // Upload stdin to run program with
        {
            // Assign a GNU header
            let mut header = tar::Header::new_gnu();
            header.set_size(code.len() as u64);
            header.set_cksum();

            // Tar the file
            let mut output = Vec::new();
            {
                let mut tar = tar::Builder::new(&mut output);
                tar.append_data(&mut header, "stdin", stdin)
                    .unwrap();
                tar.finish().unwrap();
            }

            let options = Some(UploadToContainerOptions {
                path: "/tmp",
                ..Default::default()
            });

            docker
                .upload_to_container(&self.container_id.to_string(), options, output.into())
                .await?;
        }

        Ok(())
    }

    pub async fn run(&self, docker: &Docker) -> Result<(), Error> {
        // Run container
        docker
            .start_container(
                &self.container_id.to_string(),
                None::<StartContainerOptions<String>>,
            )
            .await
            .map_err(|_| Error {
                status_code: 15,
                error: None,
            })?;

        // Await container finish
        let options = Some(WaitContainerOptions {
            condition: "not-running",
        });

        // Todo: error handling
        // Todo: add timeout, stop forever running code
        let mut stream = docker.wait_container(&self.container_id.to_string(), options);
        let res = stream.next().await;
        let res = match res {
            Some(e) => e,
            None => {
                return Err(Error {
                    status_code: 1,
                    error: None,
                });
            }
        }
        .map_err(|_| Error {
            status_code: 13,
            error: None,
        })?;

        if res.status_code != 0 {
            return Err(Error {
                status_code: res.status_code,
                error: res.error,
            });
        }

        Ok(())
    }

    pub async fn get_output(
        &self,
        docker: &Docker,
    ) -> Result<(Vec<Bytes>, Vec<Bytes>), Box<dyn std::error::Error>> {
        let options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        });

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let mut stream = docker.logs(&self.container_id.to_string(), options);
        while let Some(Ok(val)) = stream.next().await {
            match val {
                LogOutput::StdErr { message } => stderr.push(message),
                LogOutput::StdOut { message } => stdout.push(message),
                _ => {}
            }
        }

        Ok((stdout, stderr))
    }

    pub async fn clear_logs(&self, _docker: &Docker) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn remove(&self, docker: &Docker) -> Result<(), Box<dyn std::error::Error>> {
        // Remove stopped container
        let option = Some(RemoveContainerOptions {
            ..Default::default()
        });

        docker
            .remove_container(&self.container_id.to_string(), option)
            .await?;

        Ok(())
    }

    /// Get a reference to the container's container id.
    pub fn container_id(&self) -> Uuid {
        self.container_id
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
