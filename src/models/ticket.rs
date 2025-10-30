use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraTicket {
    pub key: String,
    pub fields: TicketFields,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TicketFields {
    pub summary: String,
    #[serde(default)]
    pub description: Option<String>,
    pub status: Status,
    #[serde(default)]
    pub assignee: Option<User>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "displayName")]
    pub display_name: String,
}
