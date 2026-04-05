// Message extraction and rewriting for Anthropic/OpenAI request formats

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApiFormat {
    Anthropic,
    OpenAI,
}

/// Extract the text content of the last user message from a request body.
pub fn extract_last_user_message(body: &Value, _format: ApiFormat) -> Option<String> {
    let messages = body.get("messages")?.as_array()?;
    let last_user = messages.iter().rev().find(|m| {
        m.get("role").and_then(|r| r.as_str()) == Some("user")
    })?;

    let content = last_user.get("content")?;

    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            // Both Anthropic and OpenAI support content block arrays.
            // Find all text blocks and join them.
            let text_parts: Vec<&str> = blocks.iter().filter_map(|block| {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    block.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            }).collect();
            if text_parts.is_empty() { None } else { Some(text_parts.join(" ")) }
        }
        _ => None,
    }
}

/// Replace the last user message text in the request body with the optimized version.
/// Preserves all other fields (model, params, system, non-text content blocks like images).
pub fn replace_last_user_message(body: &mut Value, optimized: &str, _format: ApiFormat) {
    let messages = match body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        Some(m) => m,
        None => return,
    };

    if let Some(last_user) = messages.iter_mut().rev().find(|m| {
        m.get("role").and_then(|r| r.as_str()) == Some("user")
    }) {
        let content = match last_user.get("content") {
            Some(c) => c.clone(),
            None => return,
        };

        match content {
            Value::String(_) => {
                last_user["content"] = Value::String(optimized.to_string());
            }
            Value::Array(blocks) => {
                let mut new_blocks = blocks.clone();
                if let Some(last_text) = new_blocks.iter_mut().rev().find(|b| {
                    b.get("type").and_then(|t| t.as_str()) == Some("text")
                }) {
                    last_text["text"] = Value::String(optimized.to_string());
                }
                last_user["content"] = Value::Array(new_blocks);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_last_user_message_anthropic() {
        let body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 1024,
            "messages": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there"},
                {"role": "user", "content": "Please help me with something maybe"}
            ]
        });
        let extracted = extract_last_user_message(&body, ApiFormat::Anthropic);
        assert_eq!(extracted, Some("Please help me with something maybe".to_string()));
    }

    #[test]
    fn test_extract_anthropic_content_blocks() {
        let body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": "Analyze this maybe somehow"}
                ]
            }]
        });
        let extracted = extract_last_user_message(&body, ApiFormat::Anthropic);
        assert_eq!(extracted, Some("Analyze this maybe somehow".to_string()));
    }

    #[test]
    fn test_extract_last_user_message_openai() {
        let body = serde_json::json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Fix this thing somehow"}
            ]
        });
        let extracted = extract_last_user_message(&body, ApiFormat::OpenAI);
        assert_eq!(extracted, Some("Fix this thing somehow".to_string()));
    }

    #[test]
    fn test_replace_preserves_non_text_blocks() {
        let mut body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image", "source": {"type": "base64", "data": "abc"}},
                    {"type": "text", "text": "Describe this image maybe"}
                ]
            }]
        });
        replace_last_user_message(&mut body, "Describe this image concisely.", ApiFormat::Anthropic);
        let blocks = body["messages"][0]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0]["type"], "image");
        assert_eq!(blocks[1]["text"], "Describe this image concisely.");
    }

    #[test]
    fn test_no_user_message_returns_none() {
        let body = serde_json::json!({
            "messages": [{"role": "system", "content": "You are a bot."}]
        });
        assert_eq!(extract_last_user_message(&body, ApiFormat::Anthropic), None);
    }

    #[test]
    fn test_replace_string_content() {
        let mut body = serde_json::json!({
            "messages": [
                {"role": "user", "content": "something vague maybe"}
            ]
        });
        replace_last_user_message(&mut body, "Optimized prompt.", ApiFormat::OpenAI);
        assert_eq!(body["messages"][0]["content"], "Optimized prompt.");
    }
}
