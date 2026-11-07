use crate::mcp::ToolSpec;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}
#[derive(Default)]
pub struct Round {
    pub text: String,
    pub calls: BTreeMap<usize, ToolCall>,
    gemini_parts: Vec<Value>,
}

impl Round {
    pub fn collect(&mut self, value: &Value) {
        if let Some(calls) = value["choices"][0]["delta"]["tool_calls"].as_array() {
            for call in calls {
                let index = call["index"].as_u64().unwrap_or(0) as usize;
                if index >= 16 {
                    continue;
                }
                let pending = self.calls.entry(index).or_default();
                if let Some(id) = call["id"].as_str() {
                    pending.id = id.into();
                }
                if let Some(name) = call["function"]["name"].as_str() {
                    if pending.name.is_empty() {
                        pending.name = name.into();
                    }
                }
                if let Some(arguments) = call["function"]["arguments"].as_str() {
                    pending.arguments.push_str(arguments);
                }
            }
        }
        let index = value["index"].as_u64().unwrap_or(0) as usize;
        if index < 16
            && value["type"] == "content_block_start"
            && value["content_block"]["type"] == "tool_use"
        {
            self.calls.insert(
                index,
                ToolCall {
                    id: value["content_block"]["id"].as_str().unwrap_or("").into(),
                    name: value["content_block"]["name"].as_str().unwrap_or("").into(),
                    arguments: String::new(),
                },
            );
        }
        if value["delta"]["type"] == "input_json_delta" {
            if let Some(call) = self.calls.get_mut(&index) {
                call.arguments
                    .push_str(value["delta"]["partial_json"].as_str().unwrap_or(""));
            }
        }
        if let Some(parts) = value["candidates"][0]["content"]["parts"].as_array() {
            for part in parts {
                // Preserve provider signatures alongside native function-call parts.
                self.gemini_parts.push(part.clone());
                if let Some(call) = part.get("functionCall") {
                    let index = self.calls.len();
                    if index < 16 {
                        self.calls.insert(
                            index,
                            ToolCall {
                                id: call["id"].as_str().unwrap_or("").into(),
                                name: call["name"].as_str().unwrap_or("").into(),
                                arguments: call.get("args").unwrap_or(&json!({})).to_string(),
                            },
                        );
                    }
                }
            }
        }
    }
}

pub fn definitions(body: &mut Value, provider: &str, tools: &[ToolSpec]) {
    if tools.is_empty() {
        return;
    }
    if provider == "gemini" {
        body["tools"] = json!([{ "functionDeclarations": tools.iter().map(|tool| json!({ "name": tool.alias, "description": format!("{}: {}", tool.server_name, tool.description), "parametersJsonSchema": tool.schema })).collect::<Vec<_>>() }]);
    } else if provider == "claude" {
        body["tools"] = json!(tools.iter().map(|tool| json!({ "name": tool.alias, "description": format!("{}: {}", tool.server_name, tool.description), "input_schema": tool.schema })).collect::<Vec<_>>());
    } else {
        body["tools"] = json!(tools.iter().map(|tool| json!({ "type": "function", "function": { "name": tool.alias, "description": format!("{}: {}", tool.server_name, tool.description), "parameters": tool.schema } })).collect::<Vec<_>>());
    }
}

pub fn append_results(
    body: &mut Value,
    provider: &str,
    round: &Round,
    results: &[(ToolCall, Value)],
) -> Result<(), String> {
    if provider == "gemini" {
        let messages = body["contents"]
            .as_array_mut()
            .ok_or("Invalid Gemini conversation.")?;
        messages.push(json!({ "role": "model", "parts": round.gemini_parts }));
        messages.push(
            json!({ "role": "user", "parts": results.iter().map(|(call, result)| {
            let mut response = json!({"name": call.name, "response": { "result": result }});
            if !call.id.is_empty() { response["id"] = json!(call.id); }
            json!({ "functionResponse": response })
        }).collect::<Vec<_>>() }),
        );
    } else if provider == "claude" {
        let mut blocks = vec![];
        if !round.text.is_empty() {
            blocks.push(json!({"type": "text", "text": round.text}));
        }
        for (call, _) in results {
            blocks.push(json!({ "type": "tool_use", "id": call.id, "name": call.name, "input": serde_json::from_str::<Value>(if call.arguments.is_empty() { "{}" } else { &call.arguments }).unwrap_or(json!({})) }));
        }
        let messages = body["messages"]
            .as_array_mut()
            .ok_or("Invalid Claude conversation.")?;
        messages.push(json!({ "role": "assistant", "content": blocks }));
        messages.push(json!({ "role": "user", "content": results.iter().map(|(call, result)| json!({ "type": "tool_result", "tool_use_id": call.id, "content": result.to_string() })).collect::<Vec<_>>() }));
    } else {
        let messages = body["messages"]
            .as_array_mut()
            .ok_or("Invalid AI conversation.")?;
        messages.push(json!({ "role": "assistant", "content": if round.text.is_empty() { Value::Null } else { json!(round.text) }, "tool_calls": results.iter().map(|(call, _)| json!({ "id": call.id, "type": "function", "function": { "name": call.name, "arguments": if call.arguments.is_empty() { "{}" } else { &call.arguments } } })).collect::<Vec<_>>() }));
        for (call, result) in results {
            messages.push(
                json!({ "role": "tool", "tool_call_id": call.id, "content": result.to_string() }),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_arguments_and_provider_result_shapes() {
        let mut round = Round::default();
        round.collect(&json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"mcp_0_0","arguments":"{\"q\":"}}]}}]}));
        round.collect(&json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"hello\"}"}}]}}]}));
        let call = round.calls[&0].clone();
        assert_eq!(
            serde_json::from_str::<Value>(&call.arguments).unwrap()["q"],
            "hello"
        );
        let mut body = json!({"messages":[]});
        append_results(&mut body, "custom", &round, &[(call, json!({"answer":42}))]).unwrap();
        assert_eq!(body["messages"][1]["role"], "tool");
        let mut claude = Round::default();
        claude.collect(&json!({"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"use_1","name":"mcp_0_0","input":{}}}));
        claude.collect(&json!({"index":1,"delta":{"type":"input_json_delta","partial_json":"{\"q\":\"hello\"}"}}));
        let call = claude.calls[&1].clone();
        let mut body = json!({"messages":[]});
        append_results(
            &mut body,
            "claude",
            &claude,
            &[(call, json!({"answer":42}))],
        )
        .unwrap();
        assert_eq!(body["messages"][0]["content"][0]["input"]["q"], "hello");
        assert_eq!(body["messages"][1]["content"][0]["tool_use_id"], "use_1");
    }
    #[test]
    fn gemini_keeps_function_call_signatures() {
        let mut round = Round::default();
        round.collect(&json!({"candidates":[{"content":{"parts":[{"functionCall":{"name":"mcp_0_0","args":{"q":"hi"}},"thoughtSignature":"fixture-signature"}]}}]}));
        let mut body = json!({"contents":[]});
        append_results(
            &mut body,
            "gemini",
            &round,
            &[(round.calls[&0].clone(), json!({"answer":42}))],
        )
        .unwrap();
        assert_eq!(
            body["contents"][0]["parts"][0]["thoughtSignature"],
            "fixture-signature"
        );
        assert_eq!(
            body["contents"][1]["parts"][0]["functionResponse"]["response"]["result"]["answer"],
            42
        );
    }
}
