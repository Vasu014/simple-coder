# Data Structures Specification

## Overview
This document defines all core data structures used throughout the simple-coder application, their relationships, and usage patterns.

## Core Communication Structures

### Message
```rust
#[derive(Debug, Serialize, Clone)]
struct Message {
    role: String,    // "user" or "assistant"
    content: String, // Message content
}
```

**Purpose**: Represents individual messages in conversation history
**Usage**: 
- Stored in conversation `Vec<Message>`
- Serialized in API requests
- Tool results formatted as user messages

**Constraints**:
- Role must be "user" or "assistant"
- Content is arbitrary UTF-8 string
- Cloneable for conversation management

### Request
```rust
#[derive(Debug, Serialize)]
struct Request {
    model: String,              // AI model identifier
    messages: Vec<Message>,     // Conversation history
    max_tokens: usize,         // Response length limit
    temperature: f32,          // Creativity/randomness
    system: String,            // System prompt
    tools: Option<Vec<ToolType>> // Available tools
}
```

**Purpose**: Complete API request payload to Anthropic
**Usage**: Constructed per AI interaction
**Serialization**: JSON for HTTP POST body

## Configuration Structures

### ModelConfig
```rust
#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,        // "anthropic"
    pub model_name: String,      // Model identifier
    pub api_key: String,         // Authentication key
    pub temperature: f32,        // 0.0-1.0 creativity
    pub max_tokens: usize,       // Response limit
    pub api_base_url: Option<String>, // Custom endpoint
}
```

**Purpose**: AI service configuration
**Lifecycle**: Created once at startup
**Security**: Contains sensitive API key
**Flexibility**: Optional custom API endpoint

## Tool System Structures

### ToolDefinition (Custom Tools)
```rust
#[derive(Debug, Serialize, Clone)]
pub struct ToolDefinition {
    name: String,               // Tool identifier
    description: String,        // Tool purpose/usage
    input_schema: serde_json::Value, // JSON Schema
}
```

**Purpose**: Define custom tool capabilities and parameters
**Schema Format**: JSON Schema specification
**Examples**:
- `read_file`: File path parameter
- `scan_directory`: No parameters (empty object)

### BuiltInToolDefinition (Anthropic Tools)
```rust
#[derive(Debug, Serialize, Clone)]
pub struct BuiltInToolDefinition {
    r#type: String,  // Anthropic tool type identifier
    name: String,    // Tool instance name
}
```

**Purpose**: Reference pre-defined Anthropic tools
**Note**: `r#type` escapes Rust keyword
**Example**: `text_editor_20250124` type

### ToolType (Unified Tool System)
```rust
#[derive(Debug, Serialize, Clone)]
enum ToolType {
    BuiltIn(BuiltInToolDefinition),
    Custom(ToolDefinition)
}
```

**Purpose**: Unified representation of all tool types
**Serialization**: Tagged enum for JSON differentiation
**Usage**: Array of `ToolType` sent to API

## Client Structure

### AnthropicClient
```rust
pub struct AnthropicClient {
    client: Client,      // HTTP client instance
    config: ModelConfig, // Configuration data
}
```

**Purpose**: Encapsulate HTTP client and configuration
**Lifecycle**: Created once, used throughout session
**Responsibilities**:
- HTTP request management
- API endpoint communication
- Request/response serialization

## File Operation Structures

### EditResult
```rust
#[derive(Debug)]
pub struct EditResult {
    success: bool,      // Operation success status
    file: String,       // Target file path
    is_new: bool,       // File creation vs modification
    changes: Vec<String>, // Diff lines
}
```

**Purpose**: Result data for file edit operations
**Usage**: Return value from `edit_file` function
**Change Format**: Diff-style lines with `+`/`-`/` ` prefixes

## JSON Response Structures

### API Response (Implicit)
The application works with `serde_json::Value` for API responses:

```json
{
  "content": [
    {
      "type": "text",
      "text": "Response content"
    }
    // OR
    {
      "type": "tool_use",
      "name": "tool_name",
      "input": { /* tool parameters */ }
    }
  ],
  "stop_reason": "end_turn" | "tool_use"
}
```

**Access Patterns**:
- `response["stop_reason"]` - Check for tool usage
- `response["content"]` - Array of response blocks
- `response["content"][0]["text"]` - Text responses
- Tool use: Iterate `content` array for `tool_use` blocks

## Tool Input Schemas

### read_file Schema
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

### scan_directory Schema
```json
{
  "type": "object",
  "properties": {}
}
```

## Data Flow Relationships

### Conversation Flow
```
User Input → Message → Vec<Message> → Request → API → Response → Message → Vec<Message>
```

### Tool Execution Flow
```
ToolType → Request → API Response → Tool Input → File Operation → Result → Message
```

### Configuration Flow
```
ModelConfig → AnthropicClient → Request Construction
```

## Memory Management

### Conversation Storage
- **Structure**: `Vec<Message>` - growable array
- **Persistence**: Session-only (lost on restart)
- **Growth**: Unbounded (potential memory leak)
- **Usage**: Complete history maintained for context

### Temporary Data
- **Tool Results**: Created, used, discarded per operation
- **Edit Results**: Generated for operation feedback
- **API Responses**: Parsed and discarded after processing

## Error Handling Types

### Standard Library Errors
- `std::io::Error` - File system operations
- `reqwest::Error` - HTTP client errors
- `serde_json::Error` - JSON parsing errors

### Error Propagation
- **Pattern**: `Result<T, Box<dyn std::error::Error>>`
- **Strategy**: Bubble up to main loop
- **Recovery**: Continue conversation on non-fatal errors

## Type Safety Considerations

### String vs &str Usage
- **Configuration**: Owned `String` for persistent data
- **Parameters**: `&str` for function parameters
- **API Data**: `String` for serialization compatibility

### Option Usage
- **API Base URL**: `Option<String>` for optional configuration
- **Tools**: `Option<Vec<ToolType>>` for optional tool availability
- **File Operations**: `Option<usize>` for optional position matching

### Clone Requirements
- **Message**: Cloneable for conversation management
- **Tool Definitions**: Cloneable for request building
- **Configuration**: Not cloned (single instance)

## Serialization Characteristics

### Derive Traits
- **Serialize**: Applied to API request structures
- **Deserialize**: Applied to configuration structures
- **Debug**: Applied to most structures for development
- **Clone**: Applied where duplication needed

### JSON Schema Handling
- **Type**: `serde_json::Value` for flexibility
- **Usage**: Dynamic schema construction
- **Validation**: No runtime schema validation

## Future Extensibility

### Tool System Extension
- **Pattern**: Add new variants to `ToolType` enum
- **Schema**: Define new `ToolDefinition` instances
- **Handler**: Add match arms in tool execution

### Configuration Extension
- **Approach**: Add fields to `ModelConfig`
- **Compatibility**: Use `Option` for optional fields
- **Migration**: No persistence layer to migrate

### Message Extension
- **Limitation**: Current structure is minimal
- **Enhancement**: Could add timestamps, metadata
- **Breaking**: Would require conversation format changes 