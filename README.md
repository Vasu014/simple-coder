# Simple Coder 🤖

An AI-powered development assistant built in Rust that provides interactive code editing, debugging, and project analysis through Anthropic's Claude 4 API.

## ✨ Features

### 🚀 **Core Capabilities**
- **Interactive Terminal Interface** - Chat-based interaction with AI assistant
- **Real-time Code Analysis** - Directory scanning and file examination
- **AI-Powered Code Editing** - Direct file modifications through natural language
- **Multi-Tool Integration** - Custom and built-in tool support
- **Session Management** - Continuous conversation with context retention

### 🛠️ **Text Editor Tools**
- **`view`** - Read and examine file contents
- **`str_replace`** - Find and replace text with precision matching
- **`create`** - Generate new files with automatic directory creation
- **`insert`** - Add content at specific line numbers
- **Automatic Backup System** - Safe undo functionality with in-memory storage

### 📊 **Analysis Tools**
- **Directory Scanning** - Complete project structure analysis
- **File Reading** - Content extraction and display
- **Code Context** - Understanding project architecture and dependencies

## 🔧 Installation

### Prerequisites
- **Rust** (1.70+) - [Install Rust](https://rustup.rs/)
- **Anthropic API Key** - [Get API Key](https://console.anthropic.com/)

### Setup Steps

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd simple-coder
   ```

2. **Install dependencies:**
   ```bash
   cargo build
   ```

3. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env and add your Anthropic API key:
   # ANTHROPIC_API_KEY=your_api_key_here
   ```

4. **Run the application:**
   ```bash
   cargo run
   ```

## 🚀 Usage

### Basic Interaction
```bash
$ cargo run
What do you want to talk about:
> Please read the main.rs file and add error handling to the file operations

# AI will automatically:
# 1. Scan project structure
# 2. Read relevant files  
# 3. Make necessary edits
# 4. Show you the changes
```

### Example Commands
- **"Fix the syntax error in src/main.rs"**
- **"Add logging to the database functions"**
- **"Create a new module for user authentication"**
- **"Review the code in utils/ and suggest improvements"**
- **"Add unit tests for the parser functions"**

### Debug Mode
Enable detailed logging:
```bash
RUST_LOG=debug cargo run
```

## ⚙️ Configuration

### Environment Variables
| Variable | Description | Required |
|----------|-------------|----------|
| `ANTHROPIC_API_KEY` | Your Anthropic API key | ✅ Yes |
| `RUST_LOG` | Log level (error/warn/info/debug/trace) | ❌ No (default: info) |

### Model Configuration
The application uses **Claude 4 Sonnet** by default. Model settings are configured in `src/main.rs`:

```rust
ModelConfig {
    model_name: "claude-sonnet-4-20250514".to_string(),
    max_tokens: 2000,  // Accommodates tool usage + response
    temperature: 0.5,  // Balanced creativity/consistency
    // ...
}
```

## 🏗️ Architecture

### Core Components
```
simple-coder/
├── src/
│   ├── main.rs              # Main application & API integration
│   ├── scan_directory.rs    # Project structure analysis
│   ├── patch_apply.rs       # Code patching utilities
│   └── tools/
│       ├── text_editor.rs   # Advanced file editing system
│       ├── read_file.rs     # File content extraction
│       └── scan_directory.rs # Directory tree operations
├── specs/                   # Comprehensive documentation
└── README.md               # This file
```

### Technology Stack
- **Language:** Rust 2021 Edition
- **AI Integration:** Anthropic Claude 4 API
- **HTTP Client:** reqwest with async/await
- **Serialization:** serde with JSON support
- **Logging:** log + env_logger
- **Environment:** dotenv for configuration

## 📚 Documentation

### Comprehensive Specifications
Detailed technical documentation is available in the `specs/` directory:

| Document | Purpose |
|----------|---------|
| [`SPECS.md`](SPECS.md) | Master documentation index |
| [`01-application-overview.md`](specs/01-application-overview.md) | System architecture |
| [`02-ai-integration.md`](specs/02-ai-integration.md) | Claude API integration |
| [`03-file-operations.md`](specs/03-file-operations.md) | File system operations |
| [`06-security-considerations.md`](specs/06-security-considerations.md) | Security analysis |
| [`08-text-editor-tool.md`](specs/08-text-editor-tool.md) | Text editor implementation |

## 🔒 Security

### Current Implementation
- ✅ **Environment Variable Configuration** - API keys stored securely
- ✅ **Input Validation** - Basic parameter checking
- ✅ **Error Handling** - Comprehensive error management
- ✅ **Thread Safety** - Mutex-protected shared state

### Security Considerations
⚠️ **Important:** This tool provides AI-powered file system access. Review the [security documentation](specs/06-security-considerations.md) for:
- Path traversal vulnerabilities
- File access restrictions
- Recommended sandboxing approaches

## 🧪 Development

### Building from Source
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
```

### Contributing
1. **Read the specifications** in `specs/` directory
2. **Follow Rust best practices** - use clippy and rustfmt
3. **Update documentation** - keep specs synchronized with code
4. **Test thoroughly** - ensure compilation and functionality

### Adding New Tools
See [`specs/02-ai-integration.md`](specs/02-ai-integration.md) for guide on extending the tool system.

## 📈 Performance

### Current Characteristics
- **Memory Usage:** Linear with file size and backup count
- **Concurrency:** Single-threaded with async I/O
- **File Size Limits:** Memory-bound (entire files loaded)
- **Session Persistence:** In-memory only

### Optimization Opportunities
- Streaming file operations for large files
- Persistent backup storage
- Multi-threading for concurrent operations
- File size limits and validation

## 🚧 Roadmap

### Short-term (v0.2.0)
- [ ] Enhanced security with path sandboxing
- [ ] Configuration file support
- [ ] Better error messages and recovery
- [ ] File size limits and validation

### Medium-term (v0.3.0)
- [ ] Persistent backup storage
- [ ] Multi-file operation support
- [ ] Plugin architecture for custom tools
- [ ] Web interface option

### Long-term (v1.0.0)
- [ ] Git integration for version control
- [ ] Collaborative editing support
- [ ] Advanced code analysis features
- [ ] Integration with popular IDEs

## 🐛 Troubleshooting

### Common Issues

**"ANTHROPIC_API_KEY environment variable must be set"**
- Ensure `.env` file exists with valid API key
- Check `.env.example` for format reference

**"Tool execution failed: Failed to read file"**
- Verify file paths are correct and accessible
- Check file permissions
- Ensure files exist in the project directory

**High memory usage**
- Large files are loaded entirely into memory
- Consider breaking down large operations
- Monitor backup storage accumulation

### Debug Mode
Enable detailed logging to diagnose issues:
```bash
RUST_LOG=debug cargo run
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Anthropic** for providing the Claude API
- **Rust Community** for excellent tooling and libraries
- **Contributors** to the open-source dependencies used in this project

## 📞 Support

- **Documentation:** Check the `specs/` directory for detailed information
- **Issues:** Report bugs and feature requests via GitHub issues
- **Discussions:** Use GitHub discussions for questions and ideas

---

**Built with ❤️ in Rust** | **Powered by Claude 4** | **Made for Developers**
