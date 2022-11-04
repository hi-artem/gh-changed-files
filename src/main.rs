use octocrab::Octocrab;
use octocrab::models;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    let octocrab = Octocrab::builder().personal_token(token).build()?;
    let mut page = octocrab
        .pulls("hi-artem", "fake-terraform")
        .list_files(1)
        .await?;
    let mut vec = Vec::new();

    loop {
        for issue in &page {
            let mut splitted: Vec<_> = issue.filename.split('/').collect();
            splitted.pop();
            let joined = splitted.join("/");
            vec.push(joined);
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

    for value in &vec {
        println!("{}", value);
    }

    Ok(())
}
