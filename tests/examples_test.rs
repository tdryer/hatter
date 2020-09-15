use {
    hatter,
    std::{
        fs,
        io::{self, Write},
        path,
    },
};

#[test]
fn test_examples() -> io::Result<()> {
    if shell("which", &["hpp"])? == "" {
        let banner = 50;
        println!("\n{}", "-".repeat(banner));
        println!("Please install hpp to run tests:\n");
        println!("https://github.com/xvxx/hpp-cli");
        println!("{}\n", "-".repeat(banner));
        return Err(io::Error::new(io::ErrorKind::Other, "hpp not found"));
    }
    test_dir("./examples/")
}

fn test_dir<P: AsRef<path::Path>>(dir: P) -> io::Result<()> {
    let dir = dir.as_ref();
    for test in fs::read_dir(dir)? {
        let test = test?;
        let path = test.path();
        if path.is_dir() {
            test_dir(path)?;
        } else {
            let source = fs::read_to_string(&path)?;
            let test_path = format!("{}", path.clone().into_os_string().into_string().unwrap())
                .replace("./examples/", "./tests/examples/")
                .replace(".hat", ".html");

            let tmp_path = "/tmp/hatter.test";
            let mut file = fs::File::create(tmp_path)?;
            write!(file, "{}", hatter::to_html(&source)?)?;
            let (expected, actual) = (pretty(&test_path)?, pretty(tmp_path)?);
            if expected != actual {
                println!("=== EXPECTED ==========\n{}", expected);
                println!("=== ACTUAL ==========\n{}", actual);
                assert!(false);
            }
        }
    }
    Ok(())
}

/// Pretty print the HTML file at `path`.
fn pretty(path: &str) -> io::Result<String> {
    shell("hpp", &[path])
}

/// Run a script and return its output.
fn shell(path: &str, args: &[&str]) -> io::Result<String> {
    let output = std::process::Command::new(path).args(args).output()?;
    let out = if output.status.success() {
        output.stdout
    } else {
        output.stderr
    };
    match std::str::from_utf8(&out) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )),
    }
}
