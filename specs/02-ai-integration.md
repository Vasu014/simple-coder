# AI Integration Specification

## Overview
The application integrates with Anthropic's Claude API to provide AI-powered code assistance through a tool-based interaction system.

## API Configuration

### Client Configuration
```rust
pub struct ModelConfig {
    pub provider: String,        // "anthropic"
    pub model_name: String,      // "claude-3-7-sonnet-20250219"
    pub api_key: String,         // API key for authentication
    pub temperature: f32,        // 0.5 - creativity level
    pub max_tokens: usize,       // 1000 - response length limit
    pub api_base_url: Option<String>, // Optional custom API endpoint
}
```

### HTTP Client Settings
- **Timeout**: 60 seconds
- **Endpoint**: `https://api.anthropic.com/v1/messages`
- **Headers**:
  - `x-api-key`: Authentication
  - `anthropic-version`: "2023-06-01"
  - `content-type`: "application/json"

## System Prompt
```
"You are an expert software architect and developer. You will work with the user for software development tasks.
Always remember to keep your solutions simple and easy to understand. Also, if you do not provide a correct and working solution,
the user might lose their job."
```

## Message Structure

### Request Format
```rust
struct Request {
    model: String,              // Model identifier
    messages: Vec<Message>,     // Conversation history
    max_tokens: usize,         // Response limit
    temperature: f32,          // Randomness control
    system: String,            // System prompt
    tools: Option<Vec<ToolType>> // Available tools
}

struct Message {
    role: String,              // "user" or "assistant"
    content: String,           // Message content
}
```

### Response Handling
- **Success Response**: Extract `content[0].text` for regular responses
- **Tool Use Response**: Process `content` array for tool calls
- **Stop Reason**: Check for `"tool_use"` to determine if tools were invoked

## Tool System Integration

### Tool Types
1. **Custom Tools**: User-defined with full schema specification
2. **Built-in Tools**: Anthropic-provided with type/name only

### Tool Definition Structure
```rust
// Custom Tool
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: serde_json::Value, // JSON Schema
}

// Built-in Tool  
struct BuiltInToolDefinition {
    r#type: String,  // "text_editor_20250124"
    name: String,    // "str_replace_editor"
}
```

## Available Tools

### 1. read_file (Custom)
- **Purpose**: Read file contents
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "file_path": {
        "type": "string",
        "description": "The path to the file to read"
      }
    },
    "required": ["file_path"]
  }
  ```

### 2. scan_directory (Custom)
- **Purpose**: Generate directory tree structure
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {}
  }
  ```

### 3. str_replace_editor (Built-in)
- **Type**: text_editor_20250124
- **Purpose**: Advanced text editing capabilities
- **Schema**: Predefined by Anthropic

## Tool Execution Flow

1. **Tool Invocation Detection**
   - Check `response["stop_reason"] == "tool_use"`
   - Parse `response["content"]` array for tool calls

2. **Tool Call Processing**
   ```rust
   if message_block["type"] == "tool_use" {
       let tool_name = message_block["name"].as_str();
       let input = message_block["input"].as_object();
       // Execute tool based on name
   }
   ```

3. **Tool Result Integration**
   - Execute requested tool operation
   - Format results as user message
   - Add to conversation history
   - Continue conversation loop

## Error Handling
- **Network Errors**: HTTP client timeout and connection handling
- **API Errors**: Response parsing and error message extraction
- **Tool Errors**: Individual tool execution error handling
- **Authentication**: API key validation

## Conversation Management
- **Message History**: Maintained in `Vec<Message>`
- **Context Preservation**: All messages kept for session duration
- **Tool Results**: Integrated as user messages with formatted content
- **Loop Control**: Continue until user types "exit"

## Security Considerations
- API key stored in plaintext (⚠️ **Security Risk**)
- No input validation on file paths
- No rate limiting implementation
- Direct file system access through tools 