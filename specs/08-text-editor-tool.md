# Text Editor Tool Specification

## Overview
The text editor tool provides comprehensive file editing capabilities through a command-based interface, supporting multiple operations including viewing, creating, modifying, and managing files with automatic backup functionality.

## Purpose and Scope
- **Primary Function**: Enable AI-powered file operations through structured commands
- **Integration**: Built-in tool for Anthropic Claude API (`text_editor_20250124`)
- **Safety**: Automatic backup system prevents data loss
- **Flexibility**: Multiple editing modes and operations

## Core Architecture

### Module Location
- **File**: `src/tools/text_editor.rs`
- **Integration**: `src/main.rs` (tool handling)
- **Dependencies**: chrono, serde_json, std::fs, std::collections

### Data Structures

#### TextEditorResult
```rust
pub struct TextEditorResult {
    pub success: bool,           // Operation success status
    pub message: String,         // Descriptive result message
    pub file_content: Option<String>, // File content (if applicable)
    pub changes_made: bool,      // Whether file was modified
}
```

**Purpose**: Standardized return type for all editor operations
**Usage**: Provides consistent feedback and content delivery

#### FileBackup
```rust
pub struct FileBackup {
    pub original_content: String,    // Original file content
    pub timestamp: DateTime<Utc>,    // Backup creation time
    pub file_path: PathBuf,         // File path reference
}
```

**Purpose**: Backup storage for undo functionality
**Lifecycle**: Created before modifications, removed after undo

## Command Interface

### Main Entry Point
```rust
pub fn handle_text_editor_tool(input_params: &Value, model_version: &str) -> Result<TextEditorResult, Box<dyn std::error::Error>>
```

**Parameters**:
- `input_params`: JSON object containing command and parameters
- `model_version`: Model identifier for compatibility checking

**Returns**: `TextEditorResult` with operation outcome

## Supported Commands

### 1. View Command
**Purpose**: Read and display file contents

**Parameters**:
- `command`: "view"
- `path`: File path to read

**Behavior**:
- Validates file path is provided
- Reads entire file content into memory
- Returns content in result structure
- No modifications made

**Error Conditions**:
- Empty file path
- File not found
- Permission denied
- Invalid UTF-8 content

### 2. String Replace Command
**Purpose**: Find and replace text within files

**Parameters**:
- `command`: "str_replace"
- `path`: Target file path
- `old_str`: Text to find (required)
- `new_str`: Replacement text

**Behavior**:
- Creates backup before modification
- Validates old_str exists in file
- Performs exact string replacement
- Writes modified content to file
- Returns new file content

**Safety Features**:
- Automatic backup creation
- Validation of search string existence
- Atomic write operations

**Error Conditions**:
- Missing required parameters
- File read/write failures
- Search string not found

### 3. Create Command
**Purpose**: Create new files with specified content

**Parameters**:
- `command`: "create"
- `path`: New file path
- `file_text`: Content for new file

**Behavior**:
- Validates file doesn't already exist
- Creates parent directories if needed
- Writes content to new file
- Returns created content

**Safety Features**:
- Prevents overwriting existing files
- Automatic directory creation
- Validation of file creation

**Error Conditions**:
- File already exists
- Invalid file path
- Permission denied
- Directory creation failure

### 4. Insert Command
**Purpose**: Insert text at specific line numbers

**Parameters**:
- `command`: "insert"
- `path`: Target file path
- `insert_line`: Line number (1-based indexing)
- `new_str`: Text to insert

**Behavior**:
- Creates backup before modification
- Validates line number range
- Inserts text at specified line
- Maintains existing line structure
- Returns modified content

**Line Numbering**:
- **1-based indexing**: Line 1 is first line
- **Insertion behavior**: New text becomes the specified line
- **Validation**: Must be between 1 and (file_lines + 1)

**Error Conditions**:
- Invalid line number
- File read/write failures
- Out of range line specification

### 5. Undo Edit Command
**Purpose**: Restore files from automatic backups

**Parameters**:
- `command`: "undo_edit"
- `path`: File path to restore

**Model Compatibility**:
- **Claude 3.5/3.7**: Fully supported
- **Claude 4**: Not supported (returns error)
- **Detection**: Checks for "str_replace_based_edit_tool" in model version

**Behavior**:
- Validates model compatibility
- Restores from backup storage
- Removes backup after successful restore
- Returns restored content with timestamp

**Error Conditions**:
- Unsupported model version
- No backup available
- Restore operation failure

## Backup System

### Storage Mechanism
```rust
static mut BACKUP_STORAGE: Option<HashMap<String, FileBackup>> = None;
```

**Characteristics**:
- **In-memory storage**: Session-only persistence
- **Key**: File path as string
- **Value**: FileBackup structure with content and metadata
- **Thread Safety**: Uses unsafe static for simplicity

### Backup Lifecycle
1. **Creation**: Before any file modification (str_replace, insert)
2. **Storage**: In global HashMap with file path as key
3. **Retrieval**: During undo operations
4. **Cleanup**: Removed after successful undo

### Backup Limitations
- **Session Scope**: Lost on application restart
- **Single Backup**: Only one backup per file path
- **Memory Usage**: Full file content stored in memory
- **No Persistence**: No disk-based backup storage

## Integration with Main Application

### Tool Registration
```rust
ToolType::BuiltIn(BuiltInToolDefinition{
    r#type: "text_editor_20250124".to_string(),
    name: "str_replace_editor".to_string()
})
```

### Tool Execution Flow
1. **Detection**: API response contains `"str_replace_editor"` tool call
2. **Parameter Extraction**: Extract command and parameters from input
3. **Model Version**: Pass current model name for compatibility
4. **Execution**: Call `handle_text_editor_tool()`
5. **Result Processing**: Format response for conversation
6. **Integration**: Add formatted result as user message

### Response Formatting
```rust
let response_content = if result.success {
    if let Some(file_content) = result.file_content {
        format!("Tool execution successful: {}\n\nFile content:\n{}", result.message, file_content)
    } else {
        format!("Tool execution successful: {}", result.message)
    }
} else {
    format!("Tool execution failed: {}", result.message)
};
```

## Error Handling Strategy

### Error Categories
1. **Validation Errors**: Missing parameters, invalid inputs
2. **File System Errors**: Read/write failures, permissions
3. **Logic Errors**: String not found, invalid line numbers
4. **Compatibility Errors**: Unsupported model versions

### Error Response Pattern
- **Never panic**: All errors returned as `TextEditorResult` with `success: false`
- **Descriptive messages**: Clear error descriptions for debugging
- **Graceful degradation**: Partial success when possible
- **State preservation**: No partial modifications on errors

## Security Considerations

### Path Security Issues
- **No path validation**: Accepts any file path without restriction
- **Directory traversal**: Vulnerable to `../` attacks
- **Absolute paths**: No sandboxing to project directory
- **System files**: Can access/modify sensitive system files

### Recommended Security Enhancements
1. **Path sanitization**: Validate and canonicalize all file paths
2. **Project sandboxing**: Restrict access to project directory
3. **File type filtering**: Limit to specific file extensions
4. **Permission checking**: Validate file access permissions
5. **Size limits**: Prevent processing of excessively large files

### Current Vulnerabilities
```rust
// Examples of dangerous operations currently possible:
// read_file("/etc/passwd")
// create("/etc/malicious_config", "malicious content")
// str_replace("/usr/bin/important_script", "safe_code", "malicious_code")
```

## Performance Characteristics

### Memory Usage
- **File Loading**: Entire files loaded into memory
- **Backup Storage**: Complete file copies stored
- **String Operations**: Multiple string allocations during operations
- **Growth Pattern**: Linear with file size and backup count

### Processing Efficiency
- **Single-threaded**: All operations on main thread
- **Blocking I/O**: Synchronous file operations
- **No Streaming**: Complete file processing only
- **Memory Bound**: Limited by available RAM for large files

### Scalability Limitations
- **Large Files**: Memory exhaustion with very large files
- **Many Backups**: Memory growth with multiple file edits
- **Concurrent Access**: No file locking or conflict resolution

## Best Practices and Usage Patterns

### Recommended Usage
1. **Small to Medium Files**: Best for typical source code files
2. **Text Files**: Designed for UTF-8 text content
3. **Sequential Operations**: Perform operations one at a time
4. **Backup Awareness**: Use undo when immediate recovery needed

### Common Patterns
```rust
// View before edit pattern
1. view(file) -> examine content
2. str_replace(file, old, new) -> modify
3. undo_edit(file) -> if needed

// Safe creation pattern
1. view(file) -> confirm doesn't exist
2. create(file, content) -> safe creation

// Incremental editing
1. view(file) -> understand structure
2. insert(file, line, content) -> add content
3. str_replace(file, old, new) -> refine
```

### Anti-Patterns
- **Large binary files**: Not designed for binary content
- **Concurrent editing**: No support for simultaneous operations
- **Critical system files**: Dangerous without proper safeguards
- **Untrusted content**: No validation of file content safety

## Dependencies and Requirements

### Runtime Dependencies
```toml
chrono = { version = "0.4", features = ["serde"] }  # Timestamp functionality
serde_json = "1.0"                                  # JSON parameter parsing
```

### Standard Library Usage
- `std::fs`: File system operations
- `std::path`: Path manipulation and validation  
- `std::collections::HashMap`: Backup storage
- `std::io`: I/O error handling

### External Integration
- **Anthropic API**: Built-in tool integration
- **Main Application**: Tool execution and result handling
- **File System**: Direct filesystem access

## Future Enhancement Opportunities

### Short-term Improvements
1. **Path Validation**: Add security-focused path sanitization
2. **File Size Limits**: Prevent memory exhaustion
3. **Better Error Messages**: More specific error categorization
4. **Content Validation**: Check for binary vs text files

### Medium-term Features
1. **Persistent Backups**: Disk-based backup storage
2. **Multiple Backups**: Version history per file
3. **Async Operations**: Non-blocking file I/O
4. **Streaming Support**: Handle large files efficiently

### Long-term Vision
1. **Git Integration**: Leverage version control for backups
2. **Collaborative Editing**: Multi-user support
3. **Advanced Search**: Regex and pattern-based operations
4. **Plugin Architecture**: Extensible command system

## Testing and Validation

### Test Coverage Requirements
- **Command Validation**: All command types and parameters
- **Error Conditions**: File system errors and edge cases
- **Backup System**: Backup creation and restoration
- **Integration**: End-to-end tool execution flow

### Security Testing
- **Path Traversal**: Test for directory escape attempts
- **Permission Testing**: Validate permission handling
- **Large File Testing**: Memory usage under stress
- **Malicious Input**: Invalid or dangerous parameters

## Compliance and Standards

### File System Compliance
- **UTF-8 Encoding**: Assumes valid UTF-8 text files
- **Line Endings**: Preserves existing line ending style
- **File Permissions**: Respects system file permissions
- **Atomic Operations**: Uses atomic write operations where possible

### API Compatibility
- **Anthropic Standards**: Follows text_editor_20250124 specification
- **JSON Schema**: Compatible with expected parameter formats
- **Error Reporting**: Consistent with API error patterns
- **Version Detection**: Proper model compatibility checking 