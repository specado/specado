use crate::error::{Error, Result};
use crate::transformer::detect;
use crate::types::{LossinessReport, PromptSpec, ProviderSpec};
use serde_json::{json, Value};
use serde_json_path::JsonPath;

pub fn translate(prompt: &PromptSpec, provider: &ProviderSpec) -> Result<(Value, LossinessReport)> {
    let mut report = LossinessReport::new(prompt.strict_mode);
    let mut payload = json!({});

    let prompt_value = serde_json::to_value(prompt)
        .map_err(|e| Error::Transform(format!("Failed to serialize prompt: {}", e)))?;

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
                while let Some(c) = chars.next() {
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
        Constraints, EndpointConfig, Endpoints, HttpMethod, JsonSchema, Mappings, Message,
        MessageRole, ModelConfig, PromptSpec, RequestMapping, ResponseConfig, ResponseMapping,
        SamplingConfig, StrictMode, SupportFlags, Tool,
    };

    fn base_provider(mappings: Vec<RequestMapping>) -> ProviderSpec {
        ProviderSpec {
            provider: "openai".into(),
            models: vec![ModelConfig {
                id: "gpt-4o".into(),
            }],
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
