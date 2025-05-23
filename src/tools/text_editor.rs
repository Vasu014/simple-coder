use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Mutex;
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct TextEditorResult {
    pub success: bool,
    pub message: String,
    pub file_content: Option<String>,
    pub changes_made: bool,
}

#[derive(Debug)]
pub struct FileBackup {
    pub original_content: String,
    pub timestamp: DateTime<Utc>,
    pub file_path: PathBuf,
}

// Safe backup storage using Mutex
static BACKUP_STORAGE: Mutex<Option<HashMap<String, FileBackup>>> = Mutex::new(None);

fn get_backup_storage() -> std::sync::MutexGuard<'static, Option<HashMap<String, FileBackup>>> {
    BACKUP_STORAGE.lock().unwrap()
}

pub fn handle_text_editor_tool(input_params: &Value, model_version: &str) -> Result<TextEditorResult, Box<dyn std::error::Error>> {
    let command = input_params.get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let file_path = input_params.get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    println!("Command: {}", command);
    println!("File path: {}", file_path);
    match command { 
        "view" => handle_view_command(file_path),
        "str_replace" => handle_str_replace_command(input_params, file_path),
        "create" => handle_create_command(input_params, file_path),
        "insert" => handle_insert_command(input_params, file_path),
        "undo_edit" => handle_undo_command(file_path, model_version),
        _ => Ok(TextEditorResult {
            success: false,
            message: format!("Unknown command: {}", command),
            file_content: None,
            changes_made: false,
        })
    }
}

fn handle_view_command(file_path: &str) -> Result<TextEditorResult, Box<dyn std::error::Error>> {
    if file_path.is_empty() {
        return Ok(TextEditorResult {
            success: false,
            message: "File path is required for view command".to_string(),
            file_content: None,
            changes_made: false,
        });
    }

    match fs::read_to_string(file_path) {
        Ok(content) => Ok(TextEditorResult {
            success: true,
            message: format!("Successfully read file: {}", file_path),
            file_content: Some(content),
            changes_made: false,
        }),
        Err(e) => Ok(TextEditorResult {
            success: false,
            message: format!("Failed to read file {}: {}", file_path, e),
            file_content: None,
            changes_made: false,
        })
    }
}

fn handle_str_replace_command(input_params: &Value, file_path: &str) -> Result<TextEditorResult, Box<dyn std::error::Error>> {
    let old_str = input_params.get("old_str")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let new_str = input_params.get("new_str")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if file_path.is_empty() || old_str.is_empty() {
        return Ok(TextEditorResult {
            success: false,
            message: "File path and old_str are required for str_replace command".to_string(),
            file_content: None,
            changes_made: false,
        });
    }

    // Read current content
    let current_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => return Ok(TextEditorResult {
            success: false,
            message: format!("Failed to read file {}: {}", file_path, e),
            file_content: None,
            changes_made: false,
        })
    };

    // Create backup before modification
    create_backup(file_path, &current_content)?;

    // Perform string replacement
    if !current_content.contains(old_str) {
        return Ok(TextEditorResult {
            success: false,
            message: format!("String '{}' not found in file {}", old_str, file_path),
            file_content: Some(current_content),
            changes_made: false,
        });
    }

    let new_content = current_content.replace(old_str, new_str);
    
    // Write modified content
    match fs::write(file_path, &new_content) {
        Ok(_) => Ok(TextEditorResult {
            success: true,
            message: format!("Successfully replaced text in {}", file_path),
            file_content: Some(new_content),
            changes_made: true,
        }),
        Err(e) => Ok(TextEditorResult {
            success: false,
            message: format!("Failed to write to file {}: {}", file_path, e),
            file_content: None,
            changes_made: false,
        })
    }
}

fn handle_create_command(input_params: &Value, file_path: &str) -> Result<TextEditorResult, Box<dyn std::error::Error>> {
    let file_text = input_params.get("file_text")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if file_path.is_empty() {
        return Ok(TextEditorResult {
            success: false,
            message: "File path is required for create command".to_string(),
            file_content: None,
            changes_made: false,
        });
    }

    // Check if file already exists
    if Path::new(file_path).exists() {
        return Ok(TextEditorResult {
            success: false,
            message: format!("File {} already exists. Use str_replace to modify existing files.", file_path),
            file_content: None,
            changes_made: false,
        });
    }

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Write new file
    match fs::write(file_path, file_text) {
        Ok(_) => Ok(TextEditorResult {
            success: true,
            message: format!("Successfully created file: {}", file_path),
            file_content: Some(file_text.to_string()),
            changes_made: true,
        }),
        Err(e) => Ok(TextEditorResult {
            success: false,
            message: format!("Failed to create file {}: {}", file_path, e),
            file_content: None,
            changes_made: false,
        })
    }
}

fn handle_insert_command(input_params: &Value, file_path: &str) -> Result<TextEditorResult, Box<dyn std::error::Error>> {
    let insert_line = input_params.get("insert_line")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    
    let new_str = input_params.get("new_str")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if file_path.is_empty() {
        return Ok(TextEditorResult {
            success: false,
            message: "File path is required for insert command".to_string(),
            file_content: None,
            changes_made: false,
        });
    }

    // Read current content
    let current_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => return Ok(TextEditorResult {
            success: false,
            message: format!("Failed to read file {}: {}", file_path, e),
            file_content: None,
            changes_made: false,
        })
    };

    // Create backup before modification
    create_backup(file_path, &current_content)?;

    let lines: Vec<&str> = current_content.lines().collect();
    
    // Insert at specified line (1-based indexing)
    if insert_line == 0 || insert_line > lines.len() + 1 {
        return Ok(TextEditorResult {
            success: false,
            message: format!("Invalid line number: {}. File has {} lines.", insert_line, lines.len()),
            file_content: Some(current_content),
            changes_made: false,
        });
    }

    // Convert to 0-based indexing
    let insert_index = insert_line - 1;
    
    // Insert the new content
    let mut new_lines = Vec::new();
    new_lines.extend_from_slice(&lines[..insert_index]);
    new_lines.push(new_str);
    new_lines.extend_from_slice(&lines[insert_index..]);
    
    let new_content = new_lines.join("\n");

    // Write modified content
    match fs::write(file_path, &new_content) {
        Ok(_) => Ok(TextEditorResult {
            success: true,
            message: format!("Successfully inserted text at line {} in {}", insert_line, file_path),
            file_content: Some(new_content),
            changes_made: true,
        }),
        Err(e) => Ok(TextEditorResult {
            success: false,
            message: format!("Failed to write to file {}: {}", file_path, e),
            file_content: None,
            changes_made: false,
        })
    }
}

fn handle_undo_command(file_path: &str, model_version: &str) -> Result<TextEditorResult, Box<dyn std::error::Error>> {
    // Check if it's a Claude 4 model (claude-sonnet-4 or claude-4)
    if model_version.contains("claude-sonnet-4") || model_version.contains("claude-4") {
        return Ok(TextEditorResult {
            success: false,
            message: "undo_edit command is not supported in Claude 4 models. The text_editor_20250429 version removes this functionality.".to_string(),
            file_content: None,
            changes_made: false,
        });
    }

    if file_path.is_empty() {
        return Ok(TextEditorResult {
            success: false,
            message: "File path is required for undo_edit command".to_string(),
            file_content: None,
            changes_made: false,
        });
    }

    let mut backup_storage = get_backup_storage();
    if backup_storage.is_none() {
        *backup_storage = Some(HashMap::new());
    }
    let backup_map = backup_storage.as_ref().unwrap();
    
    match backup_map.get(file_path) {
        Some(backup) => {
            // Clone the data we need before removing from storage
            let restored_content = backup.original_content.clone();
            let backup_timestamp = backup.timestamp;
            
            // Restore from backup
            match fs::write(file_path, &restored_content) {
                Ok(_) => {
                    // Remove the backup after successful restore
                    backup_storage.as_mut().unwrap().remove(file_path);
                    
                    Ok(TextEditorResult {
                        success: true,
                        message: format!("Successfully restored {} from backup created at {}", 
                                       file_path, backup_timestamp.format("%Y-%m-%d %H:%M:%S UTC")),
                        file_content: Some(restored_content),
                        changes_made: true,
                    })
                },
                Err(e) => Ok(TextEditorResult {
                    success: false,
                    message: format!("Failed to restore file {}: {}", file_path, e),
                    file_content: None,
                    changes_made: false,
                })
            }
        },
        None => Ok(TextEditorResult {
            success: false,
            message: format!("No backup found for file: {}", file_path),
            file_content: None,
            changes_made: false,
        })
    }
}

fn create_backup(file_path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backup = FileBackup {
        original_content: content.to_string(),
        timestamp: Utc::now(),
        file_path: PathBuf::from(file_path),
    };
    
    let mut backup_storage = get_backup_storage();
    if backup_storage.is_none() {
        *backup_storage = Some(HashMap::new());
    }
    backup_storage.as_mut().unwrap().insert(file_path.to_string(), backup);
    
    Ok(())
}