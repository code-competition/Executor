use std::io::Write;

fn main() -> Result<(), std::io::Error> {
    let out = std::process::Command::new("rustc")
        .current_dir("/tmp")
        .args(["./main.rs"])
        .output()
        .expect("failed to execute process");

    // If rustc fails compilation
    if !out.status.success() {
        // Write output to io
        if out.stdout.len() > 0 {
            std::io::stdout()
                .write_all(&out.stdout)
                .expect("failed to write output");
        }

        if out.stderr.len() > 0 {
            std::io::stderr()
                .write_all(&out.stderr)
                .expect("failed to write output");
        }

        std::process::exit(12);
    }

    // Run executable after compilation
    let out = std::process::Command::new("./main")
        .current_dir("/tmp")
        .output()
        .expect("failed to execute process");

    // Write output to io
    if out.stdout.len() > 0 {
        std::io::stdout()
            .write_all(&out.stdout)
            .expect("failed to write output");
    }

    if out.stderr.len() > 0 {
        std::io::stderr()
            .write_all(&out.stderr)
            .expect("failed to write output");
    }

    Ok(())
}
