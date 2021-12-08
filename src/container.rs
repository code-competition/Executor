use bollard::{
    container::{Config, CreateContainerOptions, UploadToContainerOptions},
    models::HostConfig,
    Docker,
};
use uuid::Uuid;

use self::languages::Languages;

pub mod languages;

pub struct Container {
    container_id: Uuid,
    internal_id: Option<String>,
}

impl Container {
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

    pub async fn upload_code(languages: Languages, code: &[u8], docker: &Docker) -> Result<(), Box<dyn std::error::Error>> {
        // Assign a GNU header
        let mut header = tar::Header::new_gnu();
        header.set_size(code.len() as u64);
        header.set_cksum();

        // Tar the file
        // Todo: GZip
        let mut output = Vec::new();
        {
            let mut tar = tar::Builder::new(&mut output);
            tar.append_data(&mut header, languages.get_filename(), code).unwrap();
            tar.finish().unwrap();
        }

        let options = Some(UploadToContainerOptions {
            path: "/tmp",
            ..Default::default()
        });
    
        docker
            .upload_to_container("my-new-container", options, output.into())
            .await?;

        Ok(())
    }
}
