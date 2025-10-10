use crate::error::{Error, Result};
use crate::transformer::detect;
use crate::types::{
    LossinessCode, LossinessEntry, LossinessLevel, LossinessReport, MessageRole, PromptSpec,
    ProviderApi, ProviderSpec,
};
use crate::AdapterRegistry;
use serde_json::{json, Map, Value};
use serde_json_path::JsonPath;

pub fn translate(prompt: &PromptSpec, provider: &ProviderSpec) -> Result<(Value, LossinessReport)> {
    let (base_payload, report) = translate_with_mappings(prompt, provider)?;

    let selection = AdapterRegistry::select(provider);

    let payload = match selection.kind() {
        ProviderApi::ChatCompletions => base_payload,
        ProviderApi::OpenaiResponses => reshape_openai_responses(prompt, provider, base_payload)?,
        ProviderApi::AnthropicMessagesClaude4 => {
            reshape_anthropic_claude4(prompt, provider, base_payload)?
        }
    };

    Ok((payload, report))
}

fn translate_with_mappings(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
) -> Result<(Value, LossinessReport)> {
    let mut report = LossinessReport::new(prompt.strict_mode);
    let mut payload = json!({});

    let prompt_value = serde_json::to_value(prompt)
        .map_err(|e| Error::Transform(format!("Failed to serialize prompt: {}", e)))?;

    detect_unsupported_parameters(&prompt_value, provider, &mut report)?;

    for mapping in &provider.mappings.request {
        let path = JsonPath::parse(&mapping.from).map_err(|e| {
            Error::Transform(format!("Invalid JSONPath '{}' : {}", mapping.from, e))
        })?;

        let matches = path.query(&prompt_value).all();
        if matches.is_empty() {
            continue;
        }

        let mut value = if matches.len() == 1 {
            matches[0].clone()
        } else {
            Value::Array(matches.iter().map(|v| (*v).clone()).collect())
        };

        if value.is_null() {
            continue;
        }

        if let Some(range) = mapping.clamp {
            if let Some(num) = value.as_f64() {
                let clamped = detect::clamp_value(num, range, mapping.from.clone(), &mut report);
                value = json!(clamped);
            }
        }

        set_value_at_path(&mut payload, &mapping.to, value)?;
    }

    detect::detect_relocate(prompt, provider, &mut report);
    detect::detect_unsupported(prompt, provider, &mut report);
    detect::detect_drops(prompt, provider, &mut report);

    Ok((payload, report))
}

fn reshape_openai_responses(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
    base_payload: Value,
) -> Result<Value> {
    let source = base_payload
        .as_object()
        .cloned()
        .ok_or_else(|| Error::Transform("Expected object payload for OpenAI Responses".into()))?;

    let mut root = Map::new();

    let model = source
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| provider.models.first().map(|m| m.id.clone()))
        .ok_or_else(|| Error::Transform("Model id missing for OpenAI request".into()))?;
    root.insert("model".into(), Value::String(model));

    if let Some(tokens) = source.get("max_output_tokens") {
        root.insert("max_output_tokens".into(), tokens.clone());
    } else {
        root.insert("max_output_tokens".into(), json!(1024));
    }

    if let Some(reasoning) = source.get("reasoning") {
        if !reasoning.is_null() {
            root.insert("reasoning".into(), reasoning.clone());
        }
    }

    if let Some(seed) = source.get("seed") {
        if !seed.is_null() {
            root.insert("seed".into(), seed.clone());
        }
    }

    if let Some(text) = source.get("text") {
        if !text.is_null() {
            root.insert("text".into(), text.clone());
        }
    }

    if let Some(service_tier) = source.get("service_tier") {
        if !service_tier.is_null() {
            root.insert("service_tier".into(), service_tier.clone());
        }
    }

    let mut instructions = source.get("instructions").and_then(value_to_string);

    let mut input_items = Vec::new();
    for message in &prompt.messages {
        match message.role {
            MessageRole::System => {
                if instructions.is_none() {
                    instructions = Some(message.content.clone());
                }
            }
            MessageRole::User | MessageRole::Assistant => {
                let role = match message.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => unreachable!(),
                };
                input_items.push(json!({
                    "role": role,
                    "content": [
                        {
                            "type": "input_text",
                            "text": message.content.clone()
                        }
                    ]
                }));
            }
        }
    }

    if input_items.is_empty() {
        return Err(Error::Transform(
            "OpenAI responses request requires at least one non-system message".into(),
        ));
    }

    if let Some(instr) = instructions {
        root.insert("instructions".into(), Value::String(instr));
    }

    root.insert("input".into(), Value::Array(input_items));

    Ok(Value::Object(root))
}

fn reshape_anthropic_claude4(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
    base_payload: Value,
) -> Result<Value> {
    let source = base_payload
        .as_object()
        .cloned()
        .ok_or_else(|| Error::Transform("Expected object payload for Anthropic request".into()))?;

    let mut root = Map::new();

    let model = source
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| provider.models.first().map(|m| m.id.clone()))
        .ok_or_else(|| Error::Transform("Model id missing for Anthropic request".into()))?;
    root.insert("model".into(), Value::String(model));

    let mut max_tokens_value = source
        .get("max_tokens")
        .cloned()
        .unwrap_or_else(|| json!(512));

    let thinking_value = source.get("thinking").cloned();
    let mut temperature_value = source.get("temperature").cloned();

    if let Some(thinking) = thinking_value.as_ref() {
        let requires_unity_temperature = thinking
            .as_object()
            .and_then(|obj| obj.get("type"))
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("enabled"))
            .unwrap_or(false);

        if requires_unity_temperature {
            temperature_value = Some(json!(1.0));
        }
    }

    if let Some(temp) = temperature_value {
        if !temp.is_null() {
            root.insert("temperature".into(), temp);
        }
    }

    if let Some(thinking) = thinking_value.as_ref() {
        if !thinking.is_null() {
            root.insert("thinking".into(), thinking.clone());
        }
    }

    let mut system_prompt = source.get("system").and_then(value_to_string);

    let mut messages = Vec::new();
    for message in &prompt.messages {
        match message.role {
            MessageRole::System => {
                if system_prompt.is_none() {
                    system_prompt = Some(message.content.clone());
                }
            }
            MessageRole::User | MessageRole::Assistant => {
                let role = match message.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => unreachable!(),
                };
                messages.push(json!({
                    "role": role,
                    "content": [
                        {
                            "type": "text",
                            "text": message.content.clone()
                        }
                    ]
                }));
            }
        }
    }

    if messages.is_empty() {
        return Err(Error::Transform(
            "Anthropic request requires at least one non-system message".into(),
        ));
    }

    if let Some(system) = system_prompt {
        root.insert("system".into(), Value::String(system));
    }

    if let Some(budget) = thinking_value
        .as_ref()
        .and_then(|val| val.as_object())
        .and_then(|obj| obj.get("budget_tokens"))
        .and_then(|v| v.as_u64())
    {
        if let Some(current_max) = max_tokens_value.as_u64() {
            if current_max <= budget {
                max_tokens_value = json!(budget + 64);
            }
        }
    }

    root.insert("max_tokens".into(), max_tokens_value);
    root.insert("messages".into(), Value::Array(messages));

    Ok(Value::Object(root))
}

fn value_to_string(value: &Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        Some(s.to_string())
    } else if let Some(array) = value.as_array() {
        array
            .iter()
            .find_map(|item| item.as_str().map(|s| s.to_string()))
    } else {
        None
    }
}

fn detect_unsupported_parameters(
    prompt_value: &Value,
    provider: &ProviderSpec,
    report: &mut LossinessReport,
) -> Result<()> {
    for path in &provider.unsupported_parameters {
        let json_path = JsonPath::parse(path).map_err(|e| {
            Error::Config(format!(
                "Invalid unsupported parameter path '{}' in provider spec: {}",
                path, e
            ))
        })?;

        let matches = json_path.query(prompt_value).all();
        let mut values = Vec::new();
        let mut present = false;

        for value in matches {
            if !value.is_null() {
                present = true;
                if values.len() < 3 {
                    values.push(value.clone());
                }
            }
        }

        if present {
            report.add_entry(LossinessEntry {
                code: LossinessCode::Unsupported,
                level: LossinessLevel::Warn,
                path: path.clone(),
                reason: format!("Parameter '{}' is unsupported by this provider", path),
                suggested_fix: Some(
                    "Remove the parameter or choose a provider that supports it".into(),
                ),
                details: Some(json!({ "examples": values })),
            });
        }
    }

    Ok(())
}

fn set_value_at_path(target: &mut Value, path: &str, value: Value) -> Result<()> {
    let segments = parse_path(path)?;
    if segments.is_empty() {
        return Err(Error::Transform(format!("Empty target path: {}", path)));
    }

    let mut current = target;
    for i in 0..segments.len() {
        let is_last = i == segments.len() - 1;
        match &segments[i] {
            PathSegment::Key(key) => {
                if is_last {
                    ensure_object(current, path)?.insert(key.clone(), value);
                    return Ok(());
                }

                let next_segment = &segments[i + 1];
                let entry = ensure_object(current, path)?
                    .entry(key.clone())
                    .or_insert_with(|| {
                        if matches!(next_segment, PathSegment::Index(_)) {
                            Value::Array(Vec::new())
                        } else {
                            Value::Object(Default::default())
                        }
                    });
                current = entry;
            }
            PathSegment::Index(index) => {
                let arr = ensure_array(current, path)?;
                if arr.len() <= *index {
                    arr.resize(index + 1, Value::Null);
                }
                if is_last {
                    arr[*index] = value;
                    return Ok(());
                } else {
                    let next_segment = &segments[i + 1];
                    if arr[*index].is_null() {
                        arr[*index] = match next_segment {
                            PathSegment::Index(_) => Value::Array(Vec::new()),
                            PathSegment::Key(_) => Value::Object(Default::default()),
                        };
                    }
                    current = &mut arr[*index];
                }
            }
        }
    }

    Ok(())
}

fn ensure_object<'a>(
    value: &'a mut Value,
    path: &str,
) -> Result<&'a mut serde_json::Map<String, Value>> {
    if !value.is_object() {
        if value.is_null() {
            *value = Value::Object(Default::default());
        } else {
            return Err(Error::Transform(format!(
                "Expected object while setting path '{}'.",
                path
            )));
        }
    }
    Ok(value.as_object_mut().expect("value checked as object"))
}

fn ensure_array<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Vec<Value>> {
    if !value.is_array() {
        if value.is_null() {
            *value = Value::Array(Vec::new());
        } else {
            return Err(Error::Transform(format!(
                "Expected array while setting path '{}'.",
                path
            )));
        }
    }
    Ok(value.as_array_mut().expect("value checked as array"))
}

fn parse_path(path: &str) -> Result<Vec<PathSegment>> {
    let mut segments = Vec::new();
    let mut buf = String::new();
    let mut chars = path
        .trim_start_matches('$')
        .trim_start_matches('.')
        .chars()
        .peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !buf.is_empty() {
                    segments.push(PathSegment::Key(std::mem::take(&mut buf)));
                }
            }
            '[' => {
                if !buf.is_empty() {
                    segments.push(PathSegment::Key(std::mem::take(&mut buf)));
                }
                let mut idx_buf = String::new();
                for c in chars.by_ref() {
                    if c == ']' {
                        break;
                    }
                    idx_buf.push(c);
                }
                if idx_buf.is_empty() {
                    return Err(Error::Transform(format!(
                        "Invalid array index in path '{}'.",
                        path
                    )));
                }
                let index = idx_buf.parse::<usize>().map_err(|_| {
                    Error::Transform(format!(
                        "Invalid array index '{}' in path '{}'.",
                        idx_buf, path
                    ))
                })?;
                segments.push(PathSegment::Index(index));
            }
            _ => buf.push(ch),
        }
    }

    if !buf.is_empty() {
        segments.push(PathSegment::Key(buf));
    }

    Ok(segments)
}

#[derive(Debug, Clone)]
enum PathSegment {
    Key(String),
    Index(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Capabilities, Constraints, EndpointConfig, Endpoints, HttpMethod, JsonSchema, Mappings,
        Message, MessageRole, ModelConfig, PromptSpec, RequestMapping, ResponseConfig,
        ResponseMapping, SamplingConfig, StrictMode, SupportFlags, Tool,
    };
    use std::collections::HashMap;

    fn base_provider(mappings: Vec<RequestMapping>) -> ProviderSpec {
        ProviderSpec {
            provider: "openai".into(),
            models: vec![ModelConfig {
                id: "gpt-4o".into(),
            }],
            interface: Some("conversational.generate".into()),
            contract_version: Some("1.0.0".into()),
            inherits: None,
            endpoints: Endpoints {
                chat: EndpointConfig {
                    method: HttpMethod::Post,
                    url: "https://api.openai.com".into(),
                    headers: Default::default(),
                },
            },
            mappings: Mappings {
                request: mappings,
                response: vec![ResponseMapping {
                    from: "$.choices[0].message".into(),
                    to: "content".into(),
                }],
            },
            constraints: Constraints {
                supports: SupportFlags {
                    json_mode: false,
                    tools: false,
                },
            },
            auth: crate::auth::AuthScheme::Bearer {
                token_env: "OPENAI_KEY".into(),
            },
            capabilities: Capabilities::default(),
            capabilities_extra: HashMap::new(),
            extensions: HashMap::new(),
            unsupported_parameters: Vec::new(),
        }
    }

    fn prompt_with_sampling() -> PromptSpec {
        PromptSpec {
            version: "1".into(),
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: "You are helpful.".into(),
                },
                Message {
                    role: MessageRole::User,
                    content: "Hi".into(),
                },
            ],
            sampling: SamplingConfig {
                temperature: Some(1.5),
                top_k: Some(50),
                ..Default::default()
            },
            response: ResponseConfig {
                format: crate::types::ResponseFormat::Json,
                json_schema: Some(JsonSchema {
                    name: "result".into(),
                    description: None,
                    schema: serde_json::json!({"type": "object"}),
                    strict: false,
                }),
            },
            tools: vec![Tool {
                name: "search".into(),
                description: None,
                json_schema: serde_json::json!({"type": "object"}),
            }],
            tool_choice: None,
            strict_mode: StrictMode::Warn,
            metadata: Default::default(),
        }
    }

    #[test]
    fn translates_with_clamp_and_relocate_detection() {
        let prompt = prompt_with_sampling();
        let provider = base_provider(vec![
            RequestMapping {
                from: "$.sampling.temperature".into(),
                to: "$.body.params.temperature".into(),
                code: None,
                clamp: Some([0.0, 1.0]),
            },
            RequestMapping {
                from: "$.messages[0]".into(),
                to: "$.body.system".into(),
                code: Some("Relocate".into()),
                clamp: None,
            },
        ]);

        let (payload, report) = translate(&prompt, &provider).expect("translate");

        assert_eq!(
            payload.pointer("/body/params/temperature").unwrap(),
            &json!(1.0)
        );
        assert!(report
            .entries
            .iter()
            .any(|e| e.code == crate::types::LossinessCode::Clamp));
        assert!(report
            .entries
            .iter()
            .any(|e| e.code == crate::types::LossinessCode::Relocate));
    }

    #[test]
    fn detects_drops_and_unsupported_capabilities() {
        let prompt = prompt_with_sampling();
        let provider = base_provider(vec![RequestMapping {
            from: "$.messages".into(),
            to: "$.body.messages".into(),
            code: None,
            clamp: None,
        }]);

        let (_payload, report) = translate(&prompt, &provider).expect("translate");

        let codes: Vec<_> = report.entries.iter().map(|e| e.code).collect();
        assert!(codes.contains(&crate::types::LossinessCode::Drop));
        assert!(codes.contains(&crate::types::LossinessCode::Unsupported));
        assert!(report.omissions.contains(&"$.sampling.top_k".to_string()));
    }

    #[test]
    fn reports_configured_unsupported_parameters() {
        let prompt = prompt_with_sampling();
        let mut provider = base_provider(vec![RequestMapping {
            from: "$.messages".into(),
            to: "$.body.messages".into(),
            code: None,
            clamp: None,
        }]);
        provider.unsupported_parameters = vec!["$.sampling.temperature".into()];

        let (_payload, report) = translate(&prompt, &provider).expect("translate");
        let unsupported = report
            .entries
            .iter()
            .find(|entry| entry.code == LossinessCode::Unsupported)
            .expect("lossiness entry");
        assert_eq!(unsupported.path, "$.sampling.temperature");
        assert_eq!(unsupported.level, LossinessLevel::Warn);
    }

    #[test]
    fn builds_nested_arrays_when_needed() {
        let prompt = prompt_with_sampling();
        let provider = base_provider(vec![RequestMapping {
            from: "$.messages[*].content".into(),
            to: "$.body.messages[0].content".into(),
            code: None,
            clamp: None,
        }]);

        let (payload, _report) = translate(&prompt, &provider).expect("translate");
        assert!(payload.pointer("/body/messages/0/content").is_some());
    }
}
