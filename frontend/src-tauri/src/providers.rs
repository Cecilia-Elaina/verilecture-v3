use crate::{db::ProviderRow, ProviderConfig};
use keyring::Entry;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use serde_json::{json, Value};
use std::time::Duration;

const KEYRING_SERVICE: &str = "verilecture-v3";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTestResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TextCompletion {
    pub text: String,
    pub duration_ms: i64,
}

pub fn provider_from_row(row: ProviderRow) -> ProviderConfig {
    ProviderConfig {
        provider: row.provider,
        protocol: row.protocol,
        base_url: row.base_url,
        model_id: row.model_id,
        organization: row.organization,
        timeout_seconds: row.timeout_seconds,
        configured: true,
        tested: row.tested,
        secret_ref: row.secret_ref,
        consent_granted: row.consent_granted,
    }
}

pub fn save_provider_secret(secret_ref: &str, api_key: &str) -> Result<(), String> {
    if api_key.trim().is_empty() {
        return Err("PROVIDER_SECRET_MISSING".to_string());
    }
    let entry =
        Entry::new(KEYRING_SERVICE, secret_ref).map_err(|_| "SECURE_STORAGE_FAILED".to_string())?;
    entry
        .set_password(api_key)
        .map_err(|_| "SECURE_STORAGE_FAILED".to_string())
}

pub fn load_provider_secret(secret_ref: &str) -> Result<String, String> {
    Entry::new(KEYRING_SERVICE, secret_ref)
        .map_err(|_| "SECURE_STORAGE_FAILED".to_string())?
        .get_password()
        .map_err(|_| "PROVIDER_SECRET_MISSING".to_string())
}

pub fn delete_provider_secret(secret_ref: &str) {
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, secret_ref) {
        let _ = entry.delete_credential();
    }
}

pub async fn test_provider_request(
    config: &ProviderConfig,
    api_key: &str,
) -> Result<ProviderTestResult, String> {
    let completion = complete_json(
        config,
        api_key,
        "You are a connection test. Return only JSON and no Markdown.",
        "Return exactly {\"ok\":true,\"message\":\"READY\"}.",
        128,
    )
    .await?;
    let payload: Value = serde_json::from_str(&completion.text)
        .map_err(|_| "PROVIDER_JSON_SCHEMA_FAILED".to_string())?;
    if payload.get("ok").and_then(Value::as_bool) != Some(true)
        || payload.get("message").and_then(Value::as_str).is_none()
    {
        return Err("PROVIDER_JSON_SCHEMA_FAILED".to_string());
    }
    Ok(ProviderTestResult {
        ok: true,
        message: "Connection, authentication and minimal JSON response passed".to_string(),
    })
}

pub async fn complete_json(
    config: &ProviderConfig,
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    max_output_tokens: u64,
) -> Result<TextCompletion, String> {
    if system_prompt.trim().is_empty() || user_prompt.trim().is_empty() {
        return Err("PROVIDER_REQUEST_INVALID".to_string());
    }
    let completion = complete_request(
        config,
        api_key,
        Some(system_prompt),
        user_prompt,
        max_output_tokens.clamp(128, 32_768),
        true,
    )
    .await?;
    let cleaned = normalize_json(&completion.text);
    serde_json::from_str::<Value>(&cleaned)
        .map_err(|_| "PROVIDER_JSON_OUTPUT_INVALID".to_string())?;
    Ok(TextCompletion {
        text: cleaned,
        duration_ms: completion.duration_ms,
    })
}

async fn complete_request(
    config: &ProviderConfig,
    api_key: &str,
    system_prompt: Option<&str>,
    user_prompt: &str,
    max_output_tokens: u64,
    json_mode: bool,
) -> Result<TextCompletion, String> {
    if api_key.trim().is_empty() {
        return Err("PROVIDER_SECRET_MISSING".to_string());
    }
    if config.base_url.trim().is_empty() || config.model_id.trim().is_empty() {
        return Err("PROVIDER_NOT_CONFIGURED".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.clamp(5, 300)))
        .build()
        .map_err(|_| "PROVIDER_NETWORK_FAILED".to_string())?;

    let mut last_error = "PROVIDER_NETWORK_FAILED".to_string();
    for attempt in 0..2u32 {
        let (url, body, headers) = build_request(
            config,
            api_key,
            system_prompt,
            user_prompt,
            max_output_tokens,
            json_mode,
        )?;
        let started = std::time::Instant::now();
        let result = client.post(url).headers(headers).json(&body).send().await;
        match result {
            Ok(response) => {
                let status = response.status();
                let text = response
                    .text()
                    .await
                    .map_err(|_| "PROVIDER_NETWORK_FAILED".to_string())?;
                if !status.is_success() {
                    let error_code = provider_status_error(status);
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
                    {
                        last_error = error_code;
                    } else {
                        return Err(error_code);
                    }
                } else {
                    let value: Value = serde_json::from_str(&text)
                        .map_err(|_| "PROVIDER_RESPONSE_NOT_JSON".to_string())?;
                    if value
                        .pointer("/choices/0/finish_reason")
                        .and_then(Value::as_str)
                        == Some("length")
                    {
                        return Err("PROVIDER_OUTPUT_TRUNCATED".to_string());
                    }
                    if value
                        .pointer("/choices/0/finish_reason")
                        .and_then(Value::as_str)
                        == Some("content_filter")
                    {
                        return Err("PROVIDER_CONTENT_FILTERED".to_string());
                    }
                    let output = extract_text(&value);
                    if output.trim().is_empty() {
                        return Err("PROVIDER_EMPTY_RESPONSE".to_string());
                    }
                    return Ok(TextCompletion {
                        text: output,
                        duration_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
                    });
                }
            }
            Err(error) => {
                last_error = if error.is_timeout() {
                    "PROVIDER_TIMEOUT".to_string()
                } else {
                    "PROVIDER_NETWORK_FAILED".to_string()
                };
            }
        }
        if attempt == 0 {
            tokio::time::sleep(Duration::from_millis(350)).await;
        }
    }
    Err(last_error)
}

fn build_request(
    config: &ProviderConfig,
    api_key: &str,
    system_prompt: Option<&str>,
    user_prompt: &str,
    max_output_tokens: u64,
    json_mode: bool,
) -> Result<(String, Value, HeaderMap), String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let system = system_prompt.unwrap_or(
        "You are a reliable assistant. Follow the user request exactly and do not invent evidence.",
    );
    let json_instruction = if json_mode {
        "Return only one valid JSON object. Do not use Markdown fences or commentary."
    } else {
        ""
    };
    let user = format!("{user_prompt}\n\n{json_instruction}");
    let protocol = config.protocol.as_str();
    let result = match protocol {
        "anthropic_messages" => {
            headers.insert(
                HeaderName::from_static("x-api-key"),
                HeaderValue::from_str(api_key).map_err(|_| "PROVIDER_SECRET_MISSING")?,
            );
            headers.insert(
                HeaderName::from_static("anthropic-version"),
                HeaderValue::from_static("2023-06-01"),
            );
            (
                endpoint(&config.base_url, "messages"),
                json!({"model": config.model_id, "system": system, "max_tokens": max_output_tokens, "messages": [{"role":"user","content":user}]}),
            )
        }
        "gemini_generate_content" => {
            headers.insert(
                HeaderName::from_static("x-goog-api-key"),
                HeaderValue::from_str(api_key).map_err(|_| "PROVIDER_SECRET_MISSING")?,
            );
            (
                endpoint(
                    &config.base_url,
                    &format!("models/{}:generateContent", config.model_id),
                ),
                json!({"systemInstruction":{"parts":[{"text":system}]},"contents":[{"parts":[{"text":user}]}],"generationConfig":{"maxOutputTokens":max_output_tokens,"responseMimeType": if json_mode { "application/json" } else { "text/plain" }}}),
            )
        }
        "openai_responses" => {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {api_key}"))
                    .map_err(|_| "PROVIDER_SECRET_MISSING")?,
            );
            (
                endpoint(&config.base_url, "responses"),
                json!({"model": config.model_id, "instructions": system, "input": user, "max_output_tokens": max_output_tokens, "text": if json_mode { json!({"format":{"type":"json_object"}}) } else { Value::Null }}),
            )
        }
        _ => {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {api_key}"))
                    .map_err(|_| "PROVIDER_SECRET_MISSING")?,
            );
            let mut body = json!({"model": config.model_id, "messages":[{"role":"system","content":system},{"role":"user","content":user}], "max_tokens": max_output_tokens});
            if json_mode {
                body["response_format"] = json!({"type":"json_object"});
            }
            if is_deepseek(config) {
                body["thinking"] = json!({"type":"disabled"});
            }
            (endpoint(&config.base_url, "chat/completions"), body)
        }
    };
    Ok((result.0, result.1, headers))
}

fn is_deepseek(config: &ProviderConfig) -> bool {
    config.provider.eq_ignore_ascii_case("deepseek")
        || config
            .base_url
            .to_ascii_lowercase()
            .contains("api.deepseek.com")
}

fn provider_status_error(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        400 | 422 => "PROVIDER_REQUEST_REJECTED".to_string(),
        401 | 403 => "PROVIDER_AUTH_FAILED".to_string(),
        402 => "PROVIDER_BALANCE_INSUFFICIENT".to_string(),
        404 => "PROVIDER_ENDPOINT_OR_MODEL_NOT_FOUND".to_string(),
        408 => "PROVIDER_TIMEOUT".to_string(),
        429 => "PROVIDER_RATE_LIMITED".to_string(),
        code if (500..600).contains(&code) => "PROVIDER_NETWORK_FAILED".to_string(),
        code if (400..500).contains(&code) => "PROVIDER_REQUEST_REJECTED".to_string(),
        _ => "PROVIDER_RESPONSE_INVALID".to_string(),
    }
}

fn endpoint(base: &str, suffix: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with(suffix) {
        base.to_string()
    } else {
        format!("{base}/{suffix}")
    }
}

fn extract_text(value: &Value) -> String {
    if let Some(text) = value.pointer("/output_text").and_then(Value::as_str) {
        return text.to_string();
    }
    if let Some(text) = value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
    {
        return text.to_string();
    }
    if let Some(items) = value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_array)
    {
        let text = items
            .iter()
            .filter_map(|item| item.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("");
        if !text.is_empty() {
            return text;
        }
    }
    if let Some(text) = value.pointer("/choices/0/text").and_then(Value::as_str) {
        return text.to_string();
    }
    if let Some(text) = value.pointer("/content/0/text").and_then(Value::as_str) {
        return text.to_string();
    }
    if let Some(text) = value
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(Value::as_str)
    {
        return text.to_string();
    }
    if let Some(items) = value.get("content").and_then(Value::as_array) {
        return items
            .iter()
            .filter_map(|item| item.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("");
    }
    String::new()
}

fn normalize_json(text: &str) -> String {
    let trimmed = text.trim();
    let without_fence = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .strip_suffix("```")
        .unwrap_or(trimmed)
        .trim();
    let start = without_fence.find('{');
    let end = without_fence.rfind('}');
    match (start, end) {
        (Some(start), Some(end)) if start <= end => without_fence[start..=end].to_string(),
        _ => without_fence.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(protocol: &str) -> ProviderConfig {
        ProviderConfig {
            provider: "fixture".to_string(),
            protocol: protocol.to_string(),
            base_url: "https://example.test/v1".to_string(),
            model_id: "fixture-model".to_string(),
            organization: None,
            timeout_seconds: 30,
            configured: true,
            tested: false,
            secret_ref: None,
            consent_granted: true,
        }
    }

    #[test]
    fn json_normalization_removes_fences_and_preamble() {
        assert_eq!(
            normalize_json("```json\n{\"ok\":true}\n```"),
            "{\"ok\":true}"
        );
        assert_eq!(
            normalize_json("Here is the result: {\"ok\":true}"),
            "{\"ok\":true}"
        );
    }

    #[test]
    fn extracts_supported_provider_shapes() {
        assert_eq!(extract_text(&json!({"output_text":"READY"})), "READY");
        assert_eq!(
            extract_text(&json!({"choices":[{"message":{"content":[{"text":"READY"}]}}]})),
            "READY"
        );
        assert_eq!(
            extract_text(&json!({"candidates":[{"content":{"parts":[{"text":"READY"}]}}]})),
            "READY"
        );
    }

    #[test]
    fn builds_all_supported_provider_protocol_shapes_without_key_in_json() {
        for protocol in [
            "openai_responses",
            "openai_compatible",
            "anthropic_messages",
            "gemini_generate_content",
        ] {
            let (url, body, headers) = build_request(
                &config(protocol),
                "secret-fixture",
                Some("system"),
                "user",
                256,
                true,
            )
            .unwrap();
            assert!(!body.to_string().contains("secret-fixture"));
            if protocol == "gemini_generate_content" {
                assert!(!url.contains("secret-fixture"));
                assert!(headers.contains_key("x-goog-api-key"));
            } else {
                assert!(headers.contains_key(AUTHORIZATION) || headers.contains_key("x-api-key"));
            }
        }
    }

    #[test]
    fn disables_deepseek_thinking_for_json_tasks() {
        let mut deepseek = config("openai_compatible");
        deepseek.provider = "DeepSeek".to_string();
        let (_, body, _) = build_request(
            &deepseek,
            "secret-fixture",
            Some("system"),
            "return json",
            256,
            true,
        )
        .unwrap();
        assert_eq!(body["thinking"]["type"], "disabled");
    }

    #[test]
    fn maps_provider_statuses_without_exposing_response_bodies() {
        assert_eq!(
            provider_status_error(reqwest::StatusCode::PAYMENT_REQUIRED),
            "PROVIDER_BALANCE_INSUFFICIENT"
        );
        assert_eq!(
            provider_status_error(reqwest::StatusCode::NOT_FOUND),
            "PROVIDER_ENDPOINT_OR_MODEL_NOT_FOUND"
        );
        assert_eq!(
            provider_status_error(reqwest::StatusCode::UNPROCESSABLE_ENTITY),
            "PROVIDER_REQUEST_REJECTED"
        );
    }
}
