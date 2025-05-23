use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::fs;
use log::debug;
mod scan_directory;
mod tools {
    pub mod text_editor;
}


const SYSTEM_MESSAGE: &str = "You are an expert software architect and developer. You will work with the user for software development tasks.
Always remember to keep your solutions simple and easy to understand. Also, if you do not provide a correct and working solution,
the user might lose their job.

When users ask about code, files, or want you to make changes, USE THE AVAILABLE TOOLS to provide hands-on assistance.

<available_tools>
You have access to these tools:
1. scan_directory - to see the project structure
2. read_file - to read file contents 
3. str_replace_based_edit_tool - to edit files

The str_replace_based_edit_tool supports these commands:
- view: Read file contents (parameter: path)
- str_replace: Replace text in files (parameters: path, old_str, new_str)
- create: Create new files (parameters: path, file_text)
- insert: Insert text at specific line (parameters: path, insert_line, new_str)

IMPORTANT: When users ask you to fix code, add features, or make changes, you should:
1. Use scan_directory or read_file to understand the current state
2. Use str_replace_based_edit_tool to make the actual changes
3. Show the user what you changed
</available_tools>

<tone>
Always reply in a warm tone, never be rude with the user. If you need clarifying questions, ask them in a friendly way.
</tone>

<response_format>
For general questions: Reply with a short answer from your knowledge.
For code questions: Use the available tools to provide a working solution.

When making file changes:
1. First use scan_directory or read_file to understand the current state
2. Use str_replace_based_edit_tool with appropriate commands to make changes
3. Provide a summary of what was changed
</response_format>

";

const READ_FILE_TOOL_DESCRIPTION: &str = "Read the contents of a file. The input is a string that is the path to the file. 
The output is a string that is the complete contents of the file. If the file does not exist, return an error message.
";

const SCAN_DIRECTORY_TOOL_DESCRIPTION: &str = "Scan the current directory and return the tree structure. 
The output is a string that is the tree structure.
The input is an empty object.
You should use this tool whenever you are unsure about current directory structure.
You should also use this tool when you don't really know where a particular file is located. 
";

#[derive(Debug, Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}
#[derive(Debug, Serialize, Clone)]
pub struct ToolDefinition{
	name: String,
	description: String,
	input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct BuiltInToolDefinition{
	r#type: String,
	name: String
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
enum ToolType{
	BuiltIn(BuiltInToolDefinition),
	Custom(ToolDefinition)
}

#[derive(Debug, Serialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
    max_tokens: usize,
    temperature: f32,
	system: String,
	tools: Option<Vec<ToolType>>
}


#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model_name: String,
    pub api_key: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub api_base_url: Option<String>,
}

pub struct AnthropicClient {
    client: Client,
    config: ModelConfig,
}

impl AnthropicClient {
	pub fn new(config: ModelConfig) -> Result<Self, Box<dyn std::error::Error>>{
		let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

		Ok(Self{client, config})
	}

	async fn send_request(&self, messages: Vec<Message>) -> Result<serde_json::Value, Box<dyn std::error::Error>>{
		let tool_definitions = vec![ToolType::Custom(ToolDefinition{
			name: "read_file".to_string(),
			description: READ_FILE_TOOL_DESCRIPTION.to_string(),
			input_schema: serde_json::json!({
				"type": "object",
				"properties": {
					"file_path": {
						"type": "string",
						"description": "The path to the file to read"
					}
				},
				"required": ["file_path"]
			})
		}),
		ToolType::Custom(ToolDefinition{
			name: "scan_directory".to_string(),
			description: SCAN_DIRECTORY_TOOL_DESCRIPTION.to_string(),
			input_schema: serde_json::json!({
				"type": "object",
				"properties": {},
			})
		}),
		ToolType::BuiltIn(BuiltInToolDefinition{
			r#type: "text_editor_20250429".to_string(),
			name: "str_replace_based_edit_tool".to_string()
		})
		];

		let request = Request {
			model: self.config.model_name.clone(),
			system: SYSTEM_MESSAGE.to_string(),
			messages,
			tools: Some(tool_definitions),
			max_tokens: self.config.max_tokens,
			temperature: self.config.temperature,
		};

		let response = self.client
		.post("https://api.anthropic.com/v1/messages")
		.header("x-api-key", self.config.api_key.clone())
		.header("anthropic-version", "2023-06-01")
		.json(&request)
		.send()
		.await?;

		
		let response_json = response.json::<serde_json::Value>().await?;
		Ok(response_json)
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	// Load environment variables from .env file
	dotenv::dotenv().ok();
	env_logger::init();

	let mut messages = Vec::new();
	let current_directory_pathbuf = std::env::current_dir()?;
	let _tree_structure = scan_directory::scan_directory_tree_from_path(&current_directory_pathbuf)?;
	let _current_directory_string = current_directory_pathbuf.to_string_lossy().into_owned();

	// Get API key from environment variable
	let api_key = std::env::var("ANTHROPIC_API_KEY")
		.expect("ANTHROPIC_API_KEY environment variable must be set. Create a .env file with your API key.");

	// Initialize the Anthropic client
	let anthropic_client = AnthropicClient::new(ModelConfig {
		provider: "anthropic".to_string(),
		model_name: "claude-sonnet-4-20250514".to_string(),
		api_key,
		temperature: 0.5,
		max_tokens: 2000,  // Increased to accommodate tool usage (700 tokens) + response
		api_base_url: None
	})?;

	let mut ask_user = true;

	// Main loop
	loop {
		let mut tool_use = false;

		if ask_user {
			println!("What do you want to talk about:");
			let line = read_line().expect("Failed to read line");

			if line == "exit" {
				println!("I guess we are done here.... Bye!");
				break;
			}
			
			messages.push(Message {
				role: "user".to_string(),
				content: line.clone()
			});
		}

		let response_json = anthropic_client.send_request(messages.clone()).await?;
		
		// Check for tool calls
		if response_json["stop_reason"].as_str() == Some("tool_use") {
			tool_use = true;
			ask_user = false;  // Don't ask for new user input
			
			// Process tool calls...
			if let Some(content_array) = response_json["content"].as_array() {
				for message_block in content_array {
					if message_block["type"].as_str() == Some("tool_use") {
						let tool_name = message_block["name"].as_str().unwrap();
						let input = message_block["input"].as_object().unwrap();
						debug!("Tool Name: {}", tool_name);
						match tool_name {
							"scan_directory" => {
								println!("Tool Call: scan_directory");
								let tree_structure = scan_directory::scan_directory_tree_from_path(&current_directory_pathbuf)?;
								let message = Message {
									role: "user".to_string(),
									content: format!("Here is the tree structure: {}", tree_structure)
								};
								messages.push(message);
							},
							"read_file" => {
								let file_path = input["file_path"].as_str().unwrap_or_default();
								let file_content = read_file(&file_path)?;
								let message = Message {
									role: "user".to_string(),
									content: format!("Here are the contents of the file {} : \n {}", file_path, file_content)
								};
								messages.push(message);
							},
							"str_replace_based_edit_tool" => {
								println!("Tool Call: str_replace_editor");
								let model_version = &anthropic_client.config.model_name;
								let input_value = serde_json::Value::Object(input.clone());
								println!("Input value: {:?}", input_value);
								let result = tools::text_editor::handle_text_editor_tool(&input_value, model_version)?;
								
								let response_content = if result.success {
									if let Some(file_content) = result.file_content {
										format!("Tool execution successful: {}\n\nFile content:\n{}", result.message, file_content)
									} else {
										format!("Tool execution successful: {}", result.message)
									}
								} else {
									format!("Tool execution failed: {}", result.message)
								};
								
								println!("Response content: {}", response_content);
								let message = Message {
									role: "user".to_string(),
									content: response_content
								};
								messages.push(message);
							},
							_ => {
								eprintln!("Tool name match not found...");
								continue
							}
						}
					}
				}
				// After processing tools, continue the loop to send another request
				// This will get the AI's response to the tool results
				continue;
			} else {
				eprintln!("Content is not an ARRAY");
				continue;
			}
		}

		// Display AI response (only when no tool use)
		if !tool_use {
			let response_text = response_json["content"][0]["text"].as_str().unwrap_or_default().to_string();
			println!("{}", response_text);
			ask_user = true;  // Ready for next user input
		}
	}

	Ok(())
}

fn read_line() -> Result<String, std::io::Error>{

	let mut line = String::new();
	std::io::stdin().read_line(&mut line)?;
	Ok(line.trim().to_string())
}

fn read_file(file_path: &str) -> Result<String, std::io::Error>{
	println!("Reading the file: {}", file_path);
	let content = fs::read_to_string(file_path)?;
	Ok(content)	
}
