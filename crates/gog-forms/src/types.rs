// Google Forms API v1 data types.
// Matches Google Forms API v1 JSON response shapes.
// https://developers.google.com/forms/api/reference/rest/v1/forms

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// FormsError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum FormsError {
    #[error("Forms API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

/// A Google Form resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Form {
    /// The form ID.
    pub form_id: String,

    /// The form's title and description metadata.
    pub info: FormInfo,

    /// The list of items (questions, page breaks, etc.) in the form.
    #[serde(default)]
    pub items: Vec<Item>,

    /// The published URL that respondents use to fill out the form.
    #[serde(default)]
    pub responder_uri: String,

    /// An opaque server-provided revision ID.
    #[serde(default)]
    pub revision_id: String,

    /// If set, the form is linked to this Google Sheets spreadsheet ID.
    pub linked_sheet_id: Option<String>,
}

impl Form {
    /// Returns the form title, or "(Untitled)" if the title is absent or empty.
    pub fn title(&self) -> &str {
        let t = self.info.title.as_str();
        if t.is_empty() { "(Untitled)" } else { t }
    }

    /// Returns the number of items that contain a question (not page breaks, etc.).
    pub fn question_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_question()).count()
    }
}

// ---------------------------------------------------------------------------
// FormInfo
// ---------------------------------------------------------------------------

/// Metadata about a form.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FormInfo {
    /// The user-visible title shown at the top of the form.
    #[serde(default)]
    pub title: String,

    /// The form description shown beneath the title.
    #[serde(default)]
    pub description: String,

    /// The title of the underlying Google Doc (may differ from `title`).
    #[serde(default)]
    pub document_title: String,
}

// ---------------------------------------------------------------------------
// Item
// ---------------------------------------------------------------------------

/// A single item in a form (question, page break, section header, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    /// Opaque server-assigned item ID.
    #[serde(default)]
    pub item_id: String,

    /// Item display title.
    #[serde(default)]
    pub title: String,

    /// Item description / helper text.
    #[serde(default)]
    pub description: String,

    /// Set when this item is a question.
    pub question_item: Option<QuestionItem>,

    /// Set when this item is a page break.
    pub page_break_item: Option<serde_json::Value>,
}

impl Item {
    /// Returns true if this item contains a question (as opposed to a structural
    /// element like a page break).
    pub fn is_question(&self) -> bool {
        self.question_item.is_some()
    }
}

// ---------------------------------------------------------------------------
// QuestionItem
// ---------------------------------------------------------------------------

/// Wrapper that holds the question payload for a question-type item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionItem {
    pub question: Question,
}

// ---------------------------------------------------------------------------
// Question
// ---------------------------------------------------------------------------

/// The question inside a QuestionItem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    /// Opaque server-assigned question ID.
    #[serde(default)]
    pub question_id: String,

    /// Whether an answer is required.
    #[serde(default)]
    pub required: bool,

    /// Set when this is a multiple-choice / checkbox / dropdown question.
    pub choice_question: Option<ChoiceQuestion>,

    /// Set when this is a short-answer or paragraph question.
    pub text_question: Option<TextQuestion>,

    /// Set when this is a linear-scale question.
    pub scale_question: Option<ScaleQuestion>,
}

impl Question {
    /// Returns a human-readable type label for this question.
    ///
    /// - `"choice"` – RADIO, CHECKBOX, or DROP_DOWN
    /// - `"text"`   – short answer or paragraph
    /// - `"scale"`  – linear scale
    /// - `"unknown"` – none of the above (image, video, etc.)
    pub fn question_type(&self) -> &str {
        if self.choice_question.is_some() {
            "choice"
        } else if self.text_question.is_some() {
            "text"
        } else if self.scale_question.is_some() {
            "scale"
        } else {
            "unknown"
        }
    }
}

// ---------------------------------------------------------------------------
// ChoiceQuestion
// ---------------------------------------------------------------------------

/// A multiple-choice, checkbox, or dropdown question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceQuestion {
    /// The choice type: `"RADIO"`, `"CHECKBOX"`, or `"DROP_DOWN"`.
    #[serde(rename = "type")]
    pub type_: String,

    /// The list of available options.
    #[serde(default)]
    pub options: Vec<ChoiceOption>,
}

// ---------------------------------------------------------------------------
// ChoiceOption
// ---------------------------------------------------------------------------

/// A single option inside a ChoiceQuestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceOption {
    /// The display value of this option.
    pub value: String,
}

// ---------------------------------------------------------------------------
// TextQuestion
// ---------------------------------------------------------------------------

/// A short-answer or paragraph text question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextQuestion {
    /// If true, this is a paragraph (long-answer) question; otherwise short-answer.
    #[serde(default)]
    pub paragraph: bool,
}

// ---------------------------------------------------------------------------
// ScaleQuestion
// ---------------------------------------------------------------------------

/// A linear-scale question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaleQuestion {
    /// The lowest value on the scale.
    pub low: i32,

    /// The highest value on the scale.
    pub high: i32,

    /// Optional label displayed at the low end.
    pub low_label: Option<String>,

    /// Optional label displayed at the high end.
    pub high_label: Option<String>,
}

// ---------------------------------------------------------------------------
// FormResponse
// ---------------------------------------------------------------------------

/// A single respondent's answers to a form.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormResponse {
    /// Opaque server-assigned response ID.
    pub response_id: String,

    /// When the response was first submitted.
    pub create_time: String,

    /// When the response was last submitted (may differ if edits are allowed).
    pub last_submitted_time: String,

    /// Email address of the respondent, if the form collected it.
    pub respondent_email: Option<String>,

    /// Map from question ID to the answer for that question.
    #[serde(default)]
    pub answers: HashMap<String, Answer>,
}

// ---------------------------------------------------------------------------
// Answer
// ---------------------------------------------------------------------------

/// An answer to a single question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    /// The question this answer belongs to.
    pub question_id: String,

    /// Text answers (present for most question types).
    pub text_answers: Option<TextAnswers>,
}

// ---------------------------------------------------------------------------
// TextAnswers / TextAnswer
// ---------------------------------------------------------------------------

/// A collection of text answer values (may be more than one for checkboxes).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextAnswers {
    #[serde(default)]
    pub answers: Vec<TextAnswer>,
}

/// A single text answer value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextAnswer {
    pub value: String,
}

// ---------------------------------------------------------------------------
// ResponseList
// ---------------------------------------------------------------------------

/// A paginated list of form responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseList {
    #[serde(default)]
    pub responses: Vec<FormResponse>,

    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // test_form_deserialize
    // -----------------------------------------------------------------------

    #[test]
    fn test_form_deserialize() {
        let json = r#"{
            "formId": "1FAIpQLSe_abc123",
            "info": {
                "title": "Customer Survey",
                "description": "Please fill in your feedback.",
                "documentTitle": "Customer Survey (Responses)"
            },
            "items": [
                {
                    "itemId": "item01",
                    "title": "What is your name?",
                    "description": "",
                    "questionItem": {
                        "question": {
                            "questionId": "q01",
                            "required": true,
                            "textQuestion": {
                                "paragraph": false
                            }
                        }
                    }
                }
            ],
            "responderUri": "https://docs.google.com/forms/d/e/1FAIpQLSe_abc123/viewform",
            "revisionId": "rev001"
        }"#;

        let form: Form = serde_json::from_str(json).expect("deserialize form");
        assert_eq!(form.form_id, "1FAIpQLSe_abc123");
        assert_eq!(form.info.title, "Customer Survey");
        assert_eq!(form.info.description, "Please fill in your feedback.");
        assert_eq!(form.info.document_title, "Customer Survey (Responses)");
        assert_eq!(form.items.len(), 1);
        assert_eq!(form.items[0].item_id, "item01");
        assert_eq!(form.items[0].title, "What is your name?");
        assert!(form.items[0].question_item.is_some());
        assert_eq!(form.responder_uri, "https://docs.google.com/forms/d/e/1FAIpQLSe_abc123/viewform");
        assert_eq!(form.revision_id, "rev001");
        assert!(form.linked_sheet_id.is_none());
    }

    // -----------------------------------------------------------------------
    // test_form_title
    // -----------------------------------------------------------------------

    #[test]
    fn test_form_title() {
        let form = Form {
            form_id: "f1".to_string(),
            info: FormInfo {
                title: "My Survey".to_string(),
                description: String::new(),
                document_title: String::new(),
            },
            items: vec![],
            responder_uri: String::new(),
            revision_id: String::new(),
            linked_sheet_id: None,
        };
        assert_eq!(form.title(), "My Survey");
    }

    // -----------------------------------------------------------------------
    // test_form_title_missing
    // -----------------------------------------------------------------------

    #[test]
    fn test_form_title_missing() {
        let form = Form {
            form_id: "f2".to_string(),
            info: FormInfo::default(),
            items: vec![],
            responder_uri: String::new(),
            revision_id: String::new(),
            linked_sheet_id: None,
        };
        assert_eq!(form.title(), "(Untitled)");
    }

    // -----------------------------------------------------------------------
    // test_form_question_count
    // -----------------------------------------------------------------------

    #[test]
    fn test_form_question_count() {
        let question_item = Item {
            item_id: "i1".to_string(),
            title: "Q1".to_string(),
            description: String::new(),
            question_item: Some(QuestionItem {
                question: Question {
                    question_id: "q1".to_string(),
                    required: false,
                    choice_question: None,
                    text_question: Some(TextQuestion { paragraph: false }),
                    scale_question: None,
                },
            }),
            page_break_item: None,
        };

        let page_break = Item {
            item_id: "i2".to_string(),
            title: "Section 2".to_string(),
            description: String::new(),
            question_item: None,
            page_break_item: Some(serde_json::json!({})),
        };

        let form = Form {
            form_id: "f3".to_string(),
            info: FormInfo::default(),
            items: vec![question_item, page_break],
            responder_uri: String::new(),
            revision_id: String::new(),
            linked_sheet_id: None,
        };

        assert_eq!(form.question_count(), 1);
    }

    // -----------------------------------------------------------------------
    // test_item_is_question
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_is_question() {
        let question_item = Item {
            item_id: "i1".to_string(),
            title: "Q?".to_string(),
            description: String::new(),
            question_item: Some(QuestionItem {
                question: Question {
                    question_id: "q1".to_string(),
                    required: false,
                    choice_question: None,
                    text_question: Some(TextQuestion { paragraph: false }),
                    scale_question: None,
                },
            }),
            page_break_item: None,
        };
        assert!(question_item.is_question());

        let page_break = Item {
            item_id: "i2".to_string(),
            title: "Break".to_string(),
            description: String::new(),
            question_item: None,
            page_break_item: Some(serde_json::json!({})),
        };
        assert!(!page_break.is_question());
    }

    // -----------------------------------------------------------------------
    // test_question_type_choice
    // -----------------------------------------------------------------------

    #[test]
    fn test_question_type_choice() {
        let q = Question {
            question_id: "q1".to_string(),
            required: false,
            choice_question: Some(ChoiceQuestion {
                type_: "RADIO".to_string(),
                options: vec![],
            }),
            text_question: None,
            scale_question: None,
        };
        assert_eq!(q.question_type(), "choice");
    }

    // -----------------------------------------------------------------------
    // test_question_type_text
    // -----------------------------------------------------------------------

    #[test]
    fn test_question_type_text() {
        let q = Question {
            question_id: "q2".to_string(),
            required: true,
            choice_question: None,
            text_question: Some(TextQuestion { paragraph: true }),
            scale_question: None,
        };
        assert_eq!(q.question_type(), "text");
    }

    // -----------------------------------------------------------------------
    // test_question_type_scale
    // -----------------------------------------------------------------------

    #[test]
    fn test_question_type_scale() {
        let q = Question {
            question_id: "q3".to_string(),
            required: false,
            choice_question: None,
            text_question: None,
            scale_question: Some(ScaleQuestion {
                low: 1,
                high: 5,
                low_label: Some("Poor".to_string()),
                high_label: Some("Excellent".to_string()),
            }),
        };
        assert_eq!(q.question_type(), "scale");
    }

    // -----------------------------------------------------------------------
    // test_form_response_deserialize
    // -----------------------------------------------------------------------

    #[test]
    fn test_form_response_deserialize() {
        let json = r#"{
            "responseId": "resp_abc",
            "createTime": "2024-03-01T10:00:00Z",
            "lastSubmittedTime": "2024-03-01T10:05:00Z",
            "respondentEmail": "alice@example.com",
            "answers": {
                "q01": {
                    "questionId": "q01",
                    "textAnswers": {
                        "answers": [
                            {"value": "Alice"}
                        ]
                    }
                }
            }
        }"#;

        let resp: FormResponse = serde_json::from_str(json).expect("deserialize form response");
        assert_eq!(resp.response_id, "resp_abc");
        assert_eq!(resp.create_time, "2024-03-01T10:00:00Z");
        assert_eq!(resp.last_submitted_time, "2024-03-01T10:05:00Z");
        assert_eq!(resp.respondent_email.as_deref(), Some("alice@example.com"));
        assert_eq!(resp.answers.len(), 1);

        let answer = resp.answers.get("q01").expect("answer for q01");
        assert_eq!(answer.question_id, "q01");
        let text_answers = answer.text_answers.as_ref().expect("text_answers present");
        assert_eq!(text_answers.answers.len(), 1);
        assert_eq!(text_answers.answers[0].value, "Alice");
    }

    // -----------------------------------------------------------------------
    // test_response_list_deserialize
    // -----------------------------------------------------------------------

    #[test]
    fn test_response_list_deserialize() {
        let json = r#"{
            "responses": [
                {
                    "responseId": "r1",
                    "createTime": "2024-03-01T09:00:00Z",
                    "lastSubmittedTime": "2024-03-01T09:01:00Z",
                    "answers": {}
                },
                {
                    "responseId": "r2",
                    "createTime": "2024-03-01T10:00:00Z",
                    "lastSubmittedTime": "2024-03-01T10:02:00Z",
                    "answers": {}
                }
            ],
            "nextPageToken": "token_xyz"
        }"#;

        let list: ResponseList = serde_json::from_str(json).expect("deserialize response list");
        assert_eq!(list.responses.len(), 2);
        assert_eq!(list.responses[0].response_id, "r1");
        assert_eq!(list.responses[1].response_id, "r2");
        assert_eq!(list.next_page_token.as_deref(), Some("token_xyz"));
    }

    // -----------------------------------------------------------------------
    // test_choice_question_options
    // -----------------------------------------------------------------------

    #[test]
    fn test_choice_question_options() {
        let json = r#"{
            "type": "CHECKBOX",
            "options": [
                {"value": "Option A"},
                {"value": "Option B"},
                {"value": "Option C"}
            ]
        }"#;

        let cq: ChoiceQuestion = serde_json::from_str(json).expect("deserialize choice question");
        assert_eq!(cq.type_, "CHECKBOX");
        assert_eq!(cq.options.len(), 3);
        assert_eq!(cq.options[0].value, "Option A");
        assert_eq!(cq.options[1].value, "Option B");
        assert_eq!(cq.options[2].value, "Option C");
    }

    // -----------------------------------------------------------------------
    // FormsError display
    // -----------------------------------------------------------------------

    #[test]
    fn test_forms_error_api_display() {
        let err = FormsError::Api { status: 403, message: "Forbidden".to_string() };
        let msg = err.to_string();
        assert!(msg.contains("403"), "got: {msg}");
        assert!(msg.contains("Forbidden"), "got: {msg}");
    }

    #[test]
    fn test_forms_error_json_display() {
        let inner: serde_json::Error = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let err = FormsError::Json(inner);
        let msg = err.to_string();
        assert!(!msg.is_empty(), "error message should be non-empty");
    }
}
