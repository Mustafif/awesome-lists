use reqwest::header;
use serde::Deserialize;

const MAX_PAGES: usize = 10;

#[derive(Deserialize)]
struct Repository {
    name: String,
    html_url: String,
    description: Option<String>,
}

impl Repository {
    pub fn new(name: String, html_url: String, description: Option<String>) -> Self {
        Self {
            name,
            html_url,
            description,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set User-Agent header
    let user_agent = header::HeaderValue::from_static("Awesome-Lists-App");
    let client = reqwest::Client::builder()
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(header::USER_AGENT, user_agent);
            headers
        })
        .build()?;
    let mut awesome_lists = Vec::new();
    for i in 1..=MAX_PAGES {
        let url = format!("https://api.github.com/search/repositories?q=topic:awesome&page={i}");
        let response = client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Process the repositories
        let repositories = response["items"]
            .as_array()
            .ok_or("Invalid response format")?;

        for repo in repositories {
            let name = repo["name"]
                .as_str()
                .ok_or("Invalid repository name")?
                .to_string();
            let url = repo["html_url"]
                .as_str()
                .ok_or("Invalid repository URL")?
                .to_string();
            let description = repo["description"].as_str().map(|s| s.to_string());
            awesome_lists.push(Repository::new(name, url, description));
        }
    }

    // Generate Markdown file
    let mut markdown = String::new();

    markdown.push_str(&format!("# The {} Awesome List  \n", awesome_lists.len()));

    for repo in awesome_lists {
        markdown.push_str(&format!(
            "- [{}]({})\n\t- {}\n",
            repo.name,
            repo.html_url,
            repo.description.unwrap_or_default()
        ));
    }

    tokio::fs::write("awesome_lists.md", markdown).await?;

    Ok(())
}
