use std::{
    io::{ErrorKind, Read, Write},
    process::Stdio,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::process::Command::new("rustc")
        .current_dir("/tmp")
        .args(["./main.rs"])
        .output()
        .expect("failed to execute process");

    // If rustc fails compilation
    if !out.status.success() {
        if !out.stderr.is_empty() {
            std::io::stderr()
                .write_all(&out.stderr)
                .expect("failed to write output");
        }

        std::process::exit(12);
    }

    // Todo: chroot a read only filesystem?

    let mut perms = std::fs::metadata("/tmp/main")?.permissions();
    perms.set_readonly(true);
    std::fs::set_permissions("/tmp/main", perms)?;

    // Read stdin info
    let mut file = std::fs::File::open("/tmp/stdin_info")?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    let stdin_count = string.parse::<usize>()?;

    for i in 0..stdin_count {
        let output = test_executable(i)?;

        std::io::stdout().write_all("TESTING_NEXT_STDIN".as_bytes())?;

        // Write output to io
        if !output.stdout.is_empty() {
            if String::from_utf8_lossy(&output.stdout).contains("TESTING_NEXT_STDIN") {
                return Err(Box::new(std::io::Error::new(ErrorKind::InvalidInput, "")));
            }

            let output = String::from_utf8(output.stdout)?;
            let output = snailquote::escape(&output);

            std::io::stdout()
                .write_all(output.as_bytes())
                .expect("failed to write output");
        }
    }
    
    std::io::stdout().flush()?;

    Ok(())
}

/// Run executable after compilation
fn test_executable(index: usize) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut process = std::process::Command::new("./main")
        .current_dir("/tmp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    // Read what to write
    let mut file = std::fs::File::open(format!("/tmp/stdin_{}", index))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // Write stdin
    if let Some(mut stdin) = process.stdin.take() {
        stdin.write_all(buf.as_ref())?;
    }

    Ok(process.wait_with_output()?)
}
