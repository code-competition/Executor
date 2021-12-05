use std::collections::HashMap;

use bollard::{
    container::{Config, CreateContainerOptions, WaitContainerOptions, LogsOptions, StartContainerOptions},
    image::ListImagesOptions,
    Docker, models::HostConfig,
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("Found {} images", images.len());

    // Create container
    let options = Some(CreateContainerOptions {
        name: "my-new-container",
    });

    let config = Config {
        image: Some("ubuntu"),
        cmd: Some(vec!["dmesg"]),
        
        // This step is important as it ensures that the code is sandboxed with gVisor
        host_config: Some(HostConfig {
            runtime: Some("runsc".into()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let res = docker.create_container(options, config).await;
    println!("{:?}", (res));

    // Run container
    let res = docker.start_container("my-new-container", None::<StartContainerOptions<String>>).await;
    println!("{:?}", (res));

    // Await finish
    let options = Some(WaitContainerOptions{
        condition: "not-running",
    });

    let mut stream = docker.wait_container("my-new-container", options);
    let res = stream.next().await;
    println!("{:#?}", res);

    // Get output from container
    let options = Some(LogsOptions::<String>{
        stdout: true,
        ..Default::default()
    });

    let mut stream = docker.logs("my-new-container", options);
    while let Some(Ok(val)) = stream.next().await {
        println!("{:#?}", val);
    }

    Ok(())
}