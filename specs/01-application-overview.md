# Application Overview Specification

## Project Identity
- **Project Name**: simple-coder
- **Description**: A very basic for loop for code editing
- **Language**: Rust
- **Build System**: Cargo

## Purpose and Goals
The simple-coder application is an AI-powered code editing assistant that provides an interactive terminal-based interface for developers to:
- Read and analyze code files
- Scan directory structures
- Perform code editing operations through AI assistance
- Apply intelligent code patches and modifications

## High-Level Architecture

### Core Components
1. **Main Application Loop** (`src/main.rs`)
   - Interactive terminal interface
   - Message handling and conversation management
   - Tool orchestration and execution

2. **AI Integration** 
   - Anthropic Claude API client
   - Tool-based interaction system
   - Request/response handling

3. **File Operations**
   - Directory scanning and tree visualization
   - File reading capabilities
   - Code patching and editing system

4. **Tool System**
   - Custom tool definitions (read_file, scan_directory)
   - Built-in tool integration (text_editor)
   - Mixed tool type support

## Key Features
- **Interactive Chat Interface**: Terminal-based conversation with AI assistant
- **File System Navigation**: Scan and explore project directory structures
- **Code Reading**: Read and analyze file contents
- **AI-Powered Editing**: Leverage Claude's text editing capabilities
- **Intelligent Patching**: Apply code changes with context-aware matching

## Technology Stack
- **Core Language**: Rust (Edition 2021)
- **HTTP Client**: reqwest (async)
- **Async Runtime**: tokio
- **Serialization**: serde + serde_json
- **Text Processing**: regex, similar
- **File Operations**: std::fs
- **AI Provider**: Anthropic Claude API

## Execution Model
1. Application starts and initializes AI client
2. Scans current directory structure
3. Enters interactive loop:
   - Prompts user for input
   - Sends messages to AI with available tools
   - Processes tool calls (file operations)
   - Displays AI responses
   - Continues until user exits

## Configuration
- **Model**: Claude 3.7 Sonnet (claude-3-7-sonnet-20250219)
- **Temperature**: 0.5
- **Max Tokens**: 1000
- **API Version**: 2023-06-01
- **Timeout**: 60 seconds 