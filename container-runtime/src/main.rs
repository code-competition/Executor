use std::{io::{Write, Read}, process::Stdio};

fn main() -> Result<(), std::io::Error> {
    let out = std::process::Command::new("rustc")
        .current_dir("/tmp")
        .args(["./main.rs"])
        .output()
        .expect("failed to execute process");

    // If rustc fails compilation
    if !out.status.success() {
        // Write output to io
        if !out.stdout.is_empty() {
            std::io::stdout()
                .write_all(&out.stdout)
                .expect("failed to write output");
        }

        if !out.stderr.is_empty() {
            std::io::stderr()
                .write_all(&out.stderr)
                .expect("failed to write output");
        }

        std::process::exit(12);
    }

    // Run executable after compilation
    let mut process = std::process::Command::new("./main")
        .current_dir("/tmp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    // Read what to write
    let mut file = std::fs::File::open("/tmp/stdin")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // Write stdin
    if let Some(mut stdin) = process.stdin.take() {
        stdin.write_all(buf.as_ref())?;
    }

    // Read stdout
    let output = process.wait_with_output()?;

    // Write output to io
    if !output.stdout.is_empty() {
        std::io::stdout()
            .write_all(&output.stdout)
            .expect("failed to write output");
    }

    if !output.stderr.is_empty() {
        std::io::stderr()
            .write_all(&output.stderr)
            .expect("failed to write output");
    }

    Ok(())
}
