use bollard::Docker;
use sandbox::{sandbox_service_server::SandboxService, SandboxRequest, SandboxResponse};
use tonic::{Request, Response, Status, transport::Server};

use crate::sandbox::sandbox_service_server::SandboxServiceServer;

pub mod container;

pub mod sandbox {
    tonic::include_proto!("sandbox");
}

#[derive(Default)]
pub struct Sandbox {}

#[tonic::async_trait]
impl SandboxService for Sandbox {
    async fn compile(
        &self,
        request: Request<SandboxRequest>,
    ) -> Result<Response<SandboxResponse>, Status> {
        let req = request.into_inner();

        let reply = SandboxResponse {
            user_id: req.user_id,
            stdout: todo!(),
            stderr: todo!(),
            runtime: todo!(),
        };

        Ok(Response::new(reply))
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse().unwrap();
//     let greeter = Sandbox::default();

//     println!("GreeterServer listening on {}", addr);

//     Server::builder()
//         .add_service(GreeterServer::new(greeter))
//         .serve(addr)
//         .await?;

//     Ok(())

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::connect_with_socket_defaults().expect("failed to connect to docker api");

    let addr = "[::1]:50051".parse().unwrap();
    let sandbox = Sandbox::default();

    println!("GRPC Server listening on {}", addr);

    Server::builder()
        .add_service(SandboxServiceServer::new(sandbox))
        .serve(addr)
        .await?;

    // let mut container = container::Container::new();

    // container.create(&docker).await.expect("failed to create container");

    // let code = String::from("fn main() { println!(\"{} Yoo!\", 20 + 5); }");
    // container.upload_code(container::languages::Languages::Rust, &code.as_bytes(), &docker).await.expect("failed to upload code");

    // container.run(&docker).await.expect("failed to run container");

    // let output = container.get_output(&docker).await.expect("could not get container output");
    // for out in output {
    //     println!("{:?}", out);
    // }

    // container.clear_logs(&docker).await?;

    // let code = String::from("fn main() { println!(\"{} Yoo!\", 35 + 5); }");
    // container.upload_code(container::languages::Languages::Rust, &code.as_bytes(), &docker).await.expect("failed to upload code");

    // container.run(&docker).await.expect("failed to run container");

    // let output = container.get_output(&docker).await.expect("could not get container output");
    // for out in output {
    //     println!("{:?}", out);
    // }

    // container.remove(&docker).await.expect("failed to remove container");

    Ok(())
}
