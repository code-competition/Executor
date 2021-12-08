use std::{collections::HashMap, fs::File, io::Read};

use bollard::{
    container::{
        Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, StartContainerOptions,
        UploadToContainerOptions, WaitContainerOptions,
    },
    image::ListImagesOptions,
    models::HostConfig,
    Docker,
};
use futures::StreamExt;
use tar::Header;

pub mod container;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code = String::from("fn main() { println!(\"{} Hello fucking World I work!\", 5 + 5); }");
    let file = code.as_bytes();
    let mut header = Header::new_gnu();
    header.set_size(file.len() as u64);
    header.set_cksum();

    let mut output = Vec::new();
    {
        let mut tar = tar::Builder::new(&mut output);
        tar.append_data(&mut header, "main.rs", file).unwrap();
        tar.finish().unwrap();
    }

    let docker = Docker::connect_with_socket_defaults().expect("failed to connect to docker api");

    // List images
    let filters: HashMap<&str, Vec<&str>> = HashMap::new();
    let options = Some(ListImagesOptions {
        all: true,
        filters,
        ..Default::default()
    });

    let images = docker
        .list_images(options)
        .await
        .expect("could not get images");

    println!("Found {} docker images", images.len());

    // Create container
    let options = Some(CreateContainerOptions {
        name: "my-new-container",
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

    let res = docker.create_container(options, config).await;
    println!("Creating container {:?}", (res));

    // Send folder to container
    let options = Some(UploadToContainerOptions {
        path: "/tmp",
        ..Default::default()
    });

    let res = docker
        .upload_to_container("my-new-container", options, output.into())
        .await;
    println!("Sending client code to container {:?}", (res));

    // Run container
    let res = docker
        .start_container("my-new-container", None::<StartContainerOptions<String>>)
        .await;
    println!("Running docker container {:?}", (res));

    // Await container finish
    let options = Some(WaitContainerOptions {
        condition: "not-running",
    });

    let mut stream = docker.wait_container("my-new-container", options);
    let res = stream.next().await;
    println!("Waiting for container to finish {:#?}", res);

    // Get output from container
    let options = Some(LogsOptions::<String> {
        stdout: true,
        stderr: true,
        ..Default::default()
    });

    let mut stream = docker.logs("my-new-container", options);
    while let Some(Ok(val)) = stream.next().await {
        println!("Logs: {:#?}", val);
    }

    // Remove stopped container
    let option = Some(RemoveContainerOptions {
        ..Default::default()
    });
    let res = docker.remove_container("my-new-container", option).await;
    println!("Removing container {:?}", (res));

    Ok(())
}
