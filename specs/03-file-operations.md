# File Operations Specification

## Overview
The application provides comprehensive file system operations including directory scanning, file reading, and intelligent code patching capabilities.

## Directory Scanning (`src/scan_directory.rs`)

### Purpose
Generate tree-structure representations of directory hierarchies with intelligent filtering of common build artifacts and system files.

### Core Functions

#### `scan_directory_tree_from_path(dir_path: &Path) -> io::Result<String>`
- **Purpose**: Scan specified directory path and return tree structure
- **Input**: Path reference to target directory
- **Output**: String representation of directory tree
- **Error Handling**: Returns `io::Error` for file system access issues

#### `scan_current_working_directory() -> io::Result<String>`
- **Purpose**: Scan current working directory
- **Output**: Tree structure string of current directory

### Filtering Rules

#### Ignored Directories
```rust
const IGNORED_DIRS: &[&str] = &[
    "target",        // Rust build artifacts
    ".git",          // Git repository data
    "node_modules",  // Node.js dependencies
    "dist",          // Distribution builds
    "build",         // Build outputs
    "out",           // Output directories
    "__pycache__",   // Python cache
    ".venv", "venv", "env", // Python virtual environments
];
```

#### Ignored Files
```rust
const IGNORED_FILES: &[&str] = &[
    ".DS_Store",     // macOS system files
];
```

### Tree Structure Format
```
project_name/
├── src/
│   ├── main.rs
│   └── lib.rs
├── Cargo.toml
└── README.md
```

#### Formatting Rules
- `├──` for non-last items at each level
- `└──` for last item at each level
- `    ` (4 spaces) for indentation per level
- Alphabetical sorting of entries

### Performance Characteristics
- **Recursive traversal**: Full directory tree exploration
- **Memory usage**: Builds complete tree string in memory
- **Sorting overhead**: Alphabetical ordering at each level
- **Filter efficiency**: Early filtering to reduce processing

## File Reading

### Core Function
```rust
fn read_file(file_path: &str) -> Result<String, std::io::Error>
```

### Features
- **Full file content reading**: Loads entire file into memory
- **UTF-8 encoding**: Assumes text files with UTF-8 encoding
- **Error propagation**: Returns `io::Error` for file access issues
- **Path flexibility**: Accepts relative and absolute paths

### Usage Pattern
1. Called via `read_file` tool from AI
2. Prints reading confirmation to console
3. Returns complete file contents as string
4. Integrated into conversation as user message

## Code Patching System (`src/patch_apply.rs`)

### Overview
Intelligent code modification system that applies edits to existing files or creates new files with context-aware matching.

### Core Function
```rust
pub fn edit_file(target_file: &str, instructions: &str, code_edit: &str) -> Result<EditResult, io::Error>
```

### Edit Result Structure
```rust
pub struct EditResult {
    success: bool,      // Operation success status
    file: String,       // Target file path
    is_new: bool,       // Whether file was created
    changes: Vec<String>, // Diff representation
}
```

## Edit Processing Logic

### New File Creation
- **Condition**: Target file doesn't exist
- **Action**: Write `code_edit` content directly to file
- **Result**: Mark as `is_new: true`, generate addition diff

### Existing File Modification

#### Marker-Based Editing
- **Marker Pattern**: `// ... existing code ...`
- **Purpose**: Indicate unchanged code sections in edit blocks
- **Processing**: Split edit into blocks separated by markers

#### Edit Block Identification
1. Parse edit content line by line
2. Split on marker pattern occurrences
3. Collect edit blocks between markers
4. Handle edge case of no markers (treat entire edit as one block)

#### Context Matching Algorithm

##### Best Match Position Finding
```rust
fn find_best_match_position(block: &[&str], original_lines: &[&str]) -> Option<usize>
```

1. **Exact Match**: Look for identical lines first
2. **Fuzzy Match**: Calculate similarity scores using Levenshtein distance
3. **Threshold**: Require 60% similarity for match acceptance
4. **Fallback**: Default to file beginning if no good match

##### Similarity Scoring
- **Algorithm**: Levenshtein distance calculation
- **Scoring**: `1.0 - (distance / max_length)`
- **Context**: Uses first 3 non-empty lines for matching

#### Change Application
1. **Context Window**: 3 lines before/after match position
2. **Replacement Strategy**: Replace calculated number of lines
3. **Line Management**: Reconstruct file with new content
4. **Content Writing**: Save modified content back to file

## Diff Generation
- **Library**: `similar` crate for text diffing
- **Format**: Standard unified diff format
- **Symbols**:
  - `+` for additions
  - `-` for deletions  
  - ` ` for unchanged context

## Error Handling

### File System Errors
- **Permission Issues**: IO error propagation
- **Path Resolution**: Invalid path handling
- **File Not Found**: Graceful error return

### Edit Operation Errors
- **Invalid Content**: Malformed edit handling
- **Encoding Issues**: UTF-8 decoding errors
- **Write Failures**: File system write error handling

## Security and Safety

### Path Security
- **No Validation**: Accepts any file path (⚠️ **Security Risk**)
- **Directory Traversal**: Vulnerable to `../` attacks
- **Absolute Paths**: No restriction on file system access

### Data Safety
- **No Backup**: No automatic backup creation
- **Overwrite Risk**: Direct file overwriting without confirmation
- **Concurrent Access**: No file locking mechanism

## Performance Considerations

### Memory Usage
- **Full File Loading**: Entire files loaded into memory
- **String Operations**: Heavy string manipulation for large files
- **Diff Generation**: Additional memory for diff calculation

### Processing Efficiency
- **Single-threaded**: All operations run on main thread
- **No Caching**: Repeated file reads not cached
- **Algorithm Complexity**: O(n×m) for similarity matching

## Integration Points
- **Tool System**: Exposed via `read_file` and `scan_directory` tools
- **AI Workflow**: Results formatted for AI consumption
- **User Interface**: Progress printed to console
- **Error Reporting**: Errors propagated to main application loop 