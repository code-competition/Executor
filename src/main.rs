use bollard::Docker;

pub mod container;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::connect_with_socket_defaults().expect("failed to connect to docker api");
    let mut container = container::Container::new();

    container.create(&docker).await.expect("failed to create container");

    let code = String::from("fn main() { println!(\"{} Yoo!\", 20 + 5); }");
    container.upload_code(container::languages::Languages::Rust, &code.as_bytes(), &docker).await.expect("failed to upload code");

    container.run(&docker).await.expect("failed to run container");

    let output = container.get_output(&docker).await.expect("could not get container output");
    for out in output {
        println!("{:?}", out);
    }

    container.remove(&docker).await.expect("failed to remove container");

    Ok(())
}
