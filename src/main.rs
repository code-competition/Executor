use bollard::Docker;
use sandbox::{sandbox_service_server::SandboxService, SandboxRequest, SandboxResponse};
use tonic::{transport::Server, Request, Response, Status};

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
        let code = req.code;
        let stdin = req.stdin;

        let docker =
            Docker::connect_with_socket_defaults().expect("failed to connect to docker api");

        // Execute code and return stdout and stderr
        let mut container = container::Container::new();

        container
            .create(&docker)
            .await
            .expect("failed to create container");

        let stdin_count = stdin.len();
        container
            .upload_code(
                container::languages::Languages::Rust,
                code.as_bytes(),
                stdin,
                &docker,
            )
            .await
            .expect("failed to upload code");

        let run_time = std::time::Instant::now();
        let success = matches!(container.run(&docker).await, Ok(_));
        let run_time = run_time.elapsed();

        let output = container
            .get_output(&docker)
            .await
            .expect("could not get container output");

        let mut stderr = Vec::new();
        for err in output.1 {
            stderr.push(String::from_utf8(err.to_vec()).unwrap());
        }

        let mut stdout = Vec::new();
        if stderr.is_empty() {
            String::from_utf8(output.0.first().unwrap().to_vec())
                .unwrap()
                .split("TESTING_NEXT_STDIN")
                .skip(1).for_each(|e| stdout.push(snailquote::unescape(e).unwrap()));
        }

        if success && stdout.len() == stdin_count-1 {
            return Err(Status::internal("internal error while compiling"));
        }

        container
            .remove(&docker)
            .await
            .expect("failed to remove container");

        let reply = SandboxResponse {
            stdout,
            stderr,
            success,
            runtime: run_time.as_millis() as u64,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let sandbox = Sandbox::default();

    println!("GRPC Server listening on {}", addr);

    Server::builder()
        .add_service(SandboxServiceServer::new(sandbox))
        .serve(addr)
        .await?;

    Ok(())
}
