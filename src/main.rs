use octocrab::Octocrab;
use octocrab::models;
use regex::Regex;
use clap::Parser;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   #[arg(short)]
   organization: String,
   #[arg(short)]
   repository: String,
   #[arg(short, default_value_t = 0)]
   pull_request: u64,
   #[arg(short, default_value = ".*")]
   directories_regex: String,
   #[arg(short, default_value = "")]
   commit: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let mut pull_request = args.pull_request;

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    if pull_request == 0 {
        assert_ne!(args.commit, "", "Provide commit SHA");

        let page = octocrab
            .repos(&args.organization, &args.repository)
            .list_pulls(args.commit)
            .send()
            .await?;
        assert_eq!(page.items.len(), 1, "Found {} PRs for commit", page.items.len());

        for pr in &page {
            pull_request = pr.number;
        }
    }

    let mut page = octocrab
        .pulls(&args.organization, &args.repository)
        .list_files(pull_request)
        .await?;
    let mut vec = Vec::new();
    let re = Regex::new(&args.directories_regex).unwrap();

    loop {
        for issue in &page {
            let mut splitted: Vec<_> = issue.filename.split('/').collect();
            splitted.pop();
            let joined = splitted.join("/");
            if re.is_match(&joined) {
                vec.push(joined);
            }
        }
        page = match octocrab
            .get_page::<models::pulls::FileDiff>(&page.next)
            .await?
        {
            Some(next_page) => next_page,
            None => break,
        }
    }

    vec.sort_unstable();
    vec.dedup();

    let result = json!({
        "result": vec
    });

    println!("{}", result.to_string());

    Ok(())
}
