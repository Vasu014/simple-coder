use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use regex::Regex;
use similar::{ChangeTag, TextDiff};

#[derive(Debug)]
pub struct EditResult {
    success: bool,
    file: String,
    is_new: bool,
    changes: Vec<String>,
}

pub fn edit_file(target_file: &str, instructions: &str, code_edit: &str) -> Result<EditResult, io::Error> {
    println!("Editing {} - {}", target_file, instructions);
    
    let file_path = Path::new(target_file);
    let file_exists = file_path.exists();
    let original_content = if file_exists {
        fs::read_to_string(file_path)?
    } else {
        String::new()
    };
    
    // If creating a new file, just write the content
    if !file_exists {
        fs::write(file_path, code_edit)?;
        return Ok(EditResult {
            success: true,
            file: target_file.to_string(),
            is_new: true,
            changes: code_edit.lines().map(|line| format!("+ {}", line)).collect(),
        });
    }
    
    // Process existing file edits using the special comment marker
    let marker_pattern = Regex::new(r"//\s*\.\.\.\s*existing\s*code\s*\.\.\.\s*").unwrap();
    
    // Split both original and edited code into lines
    let original_lines: Vec<&str> = original_content.lines().collect();
    let edit_lines: Vec<&str> = code_edit.lines().collect();
    
    // Identify edit blocks (sections between markers)
    let mut edit_blocks: Vec<Vec<&str>> = Vec::new();
    let mut current_block: Vec<&str> = Vec::new();
    
    for line in &edit_lines {
        if marker_pattern.is_match(line) {
            if !current_block.is_empty() {
                edit_blocks.push(current_block);
                current_block = Vec::new();
            }
        } else {
            current_block.push(line);
        }
    }
    
    // Add the last block if any
    if !current_block.is_empty() {
        edit_blocks.push(current_block);
    }
    
    // If no markers were used, treat the whole edit as one block
    if edit_blocks.is_empty() && !edit_lines.is_empty() {
        edit_blocks.push(edit_lines);
    }
    
    // Apply the changes to the original content
    let mut result_lines = original_lines.clone();
    
    for block in &edit_blocks {
        // Find the best match location for this block in the original file
        if let Some(best_match_pos) = find_best_match_position(block, &result_lines) {
            // Apply the change at the matched position
            let context_lines = 3; // Amount of context to consider
            let start = best_match_pos.saturating_sub(context_lines);
            let end = std::cmp::min(result_lines.len(), best_match_pos + context_lines);
            
            // Calculate how many lines to replace
            let replace_count = std::cmp::min(end - start, block.len());
            
            // Replace the lines
            let mut new_result_lines = Vec::with_capacity(result_lines.len() - replace_count + block.len());
            new_result_lines.extend_from_slice(&result_lines[0..start]);
            new_result_lines.extend_from_slice(block);
            new_result_lines.extend_from_slice(&result_lines[start + replace_count..]);
            
            result_lines = new_result_lines;
        }
    }
    
    // Join result lines and write back to file
    let new_content = result_lines.join("\n");
    fs::write(file_path, &new_content)?;
    
    // Generate a simple diff for display
    let diff = TextDiff::from_lines(&original_content, &new_content);
    let mut changes = Vec::new();
    
    for change in diff.iter_all_changes() {
        let prefix = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        changes.push(format!("{} {}", prefix, change.value()));
    }
    
    Ok(EditResult {
        success: true,
        file: target_file.to_string(),
        is_new: false,
        changes,
    })
}

fn find_best_match_position(block: &[&str], original_lines: &[&str]) -> Option<usize> {
    if block.is_empty() || original_lines.is_empty() {
        return Some(0);
    }
    
    // Get the first few non-empty lines from the block for matching
    let context_lines: Vec<&str> = block.iter()
        .filter(|&&line| !line.trim().is_empty())
        .take(3)
        .copied()
        .collect();
    
    if context_lines.is_empty() {
        return Some(0);
    }
    
    // Try to find a match for the first context line
    let context_line = context_lines[0];
    
    // Look for exact matches first
    for (i, &line) in original_lines.iter().enumerate() {
        if line == context_line {
            return Some(i);
        }
    }
    
    // Fall back to fuzzy matching
    let mut best_score = 0.0;
    let mut best_pos = 0;
    
    for (i, &line) in original_lines.iter().enumerate() {
        // Calculate similarity score
        let score = similarity_score(line, context_line);
        
        if score > best_score {
            best_score = score;
            best_pos = i;
        }
    }
    
    // If we found a reasonable match
    if best_score > 0.6 {
        Some(best_pos)
    } else {
        // Default to beginning of file
        Some(0)
    }
}

fn similarity_score(s1: &str, s2: &str) -> f64 {
    
    let mut matches = 0;
    let mut possible_matches = s1.len() + s2.len();
    
    if possible_matches == 0 {
        return 1.0;
    }
    
    // Simple Levenshtein distance algorithm
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    
    let mut dp = vec![vec![0; s2_chars.len() + 1]; s1_chars.len() + 1];
    
    // Initialize first row and column
    for i in 0..=s1_chars.len() {
        dp[i][0] = i;
    }
    
    for j in 0..=s2_chars.len() {
        dp[0][j] = j;
    }
    
    // Fill the DP table
    for i in 1..=s1_chars.len() {
        for j in 1..=s2_chars.len() {
            let cost = if s1_chars[i-1] == s2_chars[j-1] { 
                0 
            } else { 
                1 
            };
            
            let deletion = dp[i-1][j] + 1;
            let insertion = dp[i][j-1] + 1;
            let substitution = dp[i-1][j-1] + cost;
            
            dp[i][j] = std::cmp::min(
                std::cmp::min(deletion, insertion),
                substitution
            );
            
            // Log character matches for debugging
            if cost == 0 {
                matches += 1;
            }
        }
    }
    
    let distance = dp[s1_chars.len()][s2_chars.len()] as f64;
    let max_len = std::cmp::max(s1_chars.len(), s2_chars.len()) as f64;
    let similarity = if max_len == 0.0 { 
        1.0 
    } else { 
        1.0 - (distance / max_len) 
    };    
    similarity
}
