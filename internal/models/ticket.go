package models

type JiraTicket struct {
	Key    string       `json:"key"`
	Fields TicketFields `json:"fields"`
}

type TicketFields struct {
	Summary     string       `json:"summary"`
	Description string       `json:"description"`
	Status      TicketStatus `json:"status"`
	Assignee    *TicketUser  `json:"assignee"`
}

type TicketStatus struct {
	Name string `json:"name"`
}

type TicketUser struct {
	DisplayName string `json:"displayName"`
}
