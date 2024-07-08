use hyper::header::HeaderValue;
use serde::Deserialize;
use serde_json::Value;

/// This event occurs when there is a push to a repository branch. For example when a commit is pushed.
/// [source](https://docs.github.com/en/webhooks/webhook-events-and-payloads#push)
#[derive(Deserialize)]
pub struct PushEvent {
    /// The SHA of the most recent commit on ref after the push.
    pub after: String,

    pub base_ref: Option<String>,

    /// The SHA of the most recent commit on ref before the push.
    pub before: String,

    /// An array of commit objects describing the pushed commits. (Pushed commits are all commits that are included in the compare between the before commit and the after commit.) The array includes a maximum of 2048 commits. If necessary, you can use the Commits API to fetch additional commits.
    pub commits: Vec<Commits>,

    /// URL that shows the changes in this ref update, from the before commit to the after commit. For a newly created ref that is directly based on the default branch, this is the comparison between the head of the default branch and the after commit. Otherwise, this shows all commits until the after commit.
    pub compare: String,

    /// Whether this push created the ref.
    pub created: bool,

    /// Whether this push deleted the ref.
    pub deleted: bool,

    /// An enterprise on GitHub. Webhook payloads contain the enterprise property when the webhook is configured on an enterprise account or an organization that's part of an enterprise account. For more information, see "About enterprise accounts."
    pub enterprise: Option<Value>,

    /// Whether this push was a force push of the ref.
    pub forced: bool,

    /// Properties of head_commit
    pub head_commit: Commits,

    /// The GitHub App installation. Webhook payloads contain the installation property when the event is configured for and sent to a GitHub App. For more information, see "Using webhooks with GitHub Apps."
    pub installation: Option<Value>,

    /// A GitHub organization. Webhook payloads contain the organization property when the webhook is configured for an organization, or when the event occurs from activity in a repository owned by an organization.
    pub organization: Option<Value>,

    /// Metaproperties for Git author/committer information.
    pub pusher: User,

    /// The full git ref that was pushed. Example: refs/heads/main or refs/tags/v3.14.1.
    #[serde(rename = "ref")]
    pub ref_: String,

    /// A git repository
    pub repository: Value,

    /// The GitHub user that triggered the event. This property is included in every webhook payload.
    pub sender: Option<Value>,
}

impl PushEvent {
    pub const HEADER_EVENT: &str = "push";
}

#[derive(Deserialize)]
pub struct Commits {
    /// An array of files added in the commit. A maximum of 3000 changed files will be reported per commit.
    pub added: Vec<String>,

    /// Metaproperties for Git author/committer information.
    pub author: User,

    /// Metaproperties for Git author/committer information.
    pub committer: User,

    /// Whether this commit is distinct from any that have been pushed before.
    pub distinct: bool,

    pub id: String,

    /// The commit message.
    pub message: String,

    /// An array of files modified by the commit. A maximum of 3000 changed files will be reported per commit.
    pub modified: Option<Vec<String>>,

    /// An array of files removed in the commit. A maximum of 3000 changed files will be reported per commit.
    pub removed: Option<Vec<String>>,

    /// The ISO 8601 timestamp of the commit.
    pub timestamp: String,

    pub tree_id: String,

    /// URL that points to the commit API resource.
    pub url: String,
}

#[derive(Deserialize)]
pub struct User {
    pub date: String,
    pub email: Option<String>,
    /// The git author's name.
    pub name: String,
    pub username: String,
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct GithubHeader {
    /// The unique identifier of the webhook
    pub x_github_hook_id: String,
    /// The name of the event that triggered the delivery.
    pub x_github_event: String,
    /// A globally unique identifier (GUID) to identify the event.
    pub x_github_delivery: String,
    /// This header is sent if the webhook is configured with a secret. This is the HMAC hex digest of the request body, and is generated using the SHA-1 hash function and the secret as the HMAC key. X-Hub-Signature is provided for compatibility with existing integrations. We recommend that you use the more secure X-Hub-Signature-256 instead.
    pub x_hub_signature: Option<String>,
    /// This header is sent if the webhook is configured with a secret. This is the HMAC hex digest of the request body, and is generated using the SHA-256 hash function and the secret as the HMAC key. For more information, see "Validating webhook deliveries."
    pub x_hub_signature_256: Option<String>,
    /// This header will always have the prefix GitHub-Hookshot/.
    pub user_agent: String,
    /// The type of resource where the webhook was created.
    pub x_github_hook_installation_target_type: String,
    /// The unique identifier of the resource where the webhook was created.
    pub x_github_hook_installation_target_id: String,
}

impl GithubHeader {
    pub fn from_request_parts(parts: &hyper::http::request::Parts) -> GithubHeader {
        Self {
            x_github_hook_id: parts.headers.get("X-GitHub-Hook-ID").map_or("".into(), str_default),
            x_github_event: parts.headers.get("X-GitHub-Event").map_or("".into(), str_default),
            x_github_delivery: parts.headers.get("X-GitHub-Delivery").map_or("".into(), str_default),
            x_hub_signature: parts.headers.get("X-Hub-Signature").and_then(to_string_opt),
            x_hub_signature_256: parts.headers.get("X-Hub-Signature-256").and_then(to_string_opt),
            user_agent: parts.headers.get("User-Agent").map_or("".into(), str_default),
            x_github_hook_installation_target_type: parts.headers.get("X-GitHub-Hook-Installation-Target-Type").map_or("".into(), str_default),
            x_github_hook_installation_target_id: parts.headers.get("X-GitHub-Hook-Installation-Target-ID").map_or("".into(), str_default),
        }
    }
}

fn to_string_opt(e: &HeaderValue) -> Option<String> {
    e.to_str().map(ToString::to_string).ok()
}

fn str_default(head: &HeaderValue) -> String {
    head.to_str().map(ToString::to_string).unwrap_or_default()
}
