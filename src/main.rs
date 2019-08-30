use std::{
    borrow::Cow,
    collections::BTreeMap,
    env::args,
    error::Error,
    io::{BufRead, BufReader},
    process::{exit, Command, Stdio},
};

fn main() -> Result<(), Box<dyn Error>> {
    let path = args().nth(1).unwrap_or_else(|| ".".into());
    if let Some(owners) = codeowners::locate(".").map(codeowners::from_path) {
        let cmd = Command::new("git")
            .args(&["ls-files", "--", path.as_str(), ":!node_modules"])
            .stdout(Stdio::piped())
            .spawn()?;
        {
            if let Some(stdout) = cmd.stdout {
                let result = BufReader::new(stdout).lines().filter_map(Result::ok).fold(
                    BTreeMap::new(),
                    |mut res, path| {
                        let owner = owners
                            .of(path)
                            .and_then(|owners| {
                                owners.first().map(|owner| Cow::Owned(owner.to_string()))
                            })
                            .unwrap_or_else(|| Cow::Borrowed("nobody"));
                        *res.entry(owner).or_insert(0) += 1;
                        res
                    },
                );
                for (owner, file_count) in result {
                    println!("{},{}", owner, file_count)
                }
            } else {
                eprintln!("Stdout is empty");
                exit(1);
            }
        }
    } else {
        eprintln!("Failed to locate a CODEWONERS file");
        exit(1);
    }

    Ok(())
}
