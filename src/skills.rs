use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::GearClawError;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub path: PathBuf,
}

pub struct SkillManager {
    pub skills: Vec<Skill>,
}

impl SkillManager {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), GearClawError> {
        let dir = dir.as_ref();
        if !dir.exists() {
            warn!("Skills directory not found: {:?}", dir);
            return Ok(());
        }

        info!("Loading skills from {:?}", dir);
        self.load_recursive(dir)?;
        
        info!("Loaded {} skills", self.skills.len());
        Ok(())
    }

    fn load_recursive(&mut self, dir: &Path) -> Result<(), GearClawError> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir).map_err(GearClawError::IoError)? {
            let entry = entry.map_err(GearClawError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                // Check if SKILL.md exists in this directory
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Err(e) = self.load_skill(&skill_file) {
                        warn!("Failed to load skill from {:?}: {}", skill_file, e);
                    }
                } else {
                    // Recurse into subdirectory
                    self.load_recursive(&path)?;
                }
            }
        }
        Ok(())
    }

    fn load_skill(&mut self, path: &Path) -> Result<(), GearClawError> {
        let content = std::fs::read_to_string(path).map_err(GearClawError::IoError)?;
        
        // Parse frontmatter
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            // Try to be more robust if the file starts with --- immediately
            if content.starts_with("---") {
                 // splitn might have behaved differently depending on if there is text before the first ---
                 // If file starts with ---, splitn(3, "---") -> ["", "frontmatter", "body"]
            } else {
                return Err(GearClawError::ConfigParseError(format!("Invalid skill file format in {:?}: missing frontmatter", path)));
            }
        }
        
        // Check if the first part is empty (standard frontmatter)
        let (frontmatter, instructions) = if parts[0].trim().is_empty() {
             (parts[1], parts[2])
        } else {
             // Maybe the file doesn't start with ---?
             // But valid frontmatter usually starts with ---
             return Err(GearClawError::ConfigParseError(format!("Invalid skill file format in {:?}: content before frontmatter", path)));
        };

        let meta: SkillMetadata = serde_yaml::from_str(frontmatter)
            .map_err(|e| GearClawError::ConfigParseError(format!("Invalid skill metadata in {:?}: {}", path, e)))?;

        let skill = Skill {
            name: meta.name,
            description: meta.description,
            instructions: instructions.trim().to_string(),
            path: path.to_path_buf(),
        };

        self.skills.push(skill);
        Ok(())
    }

    pub fn get_prompt_context(&self) -> String {
        if self.skills.is_empty() {
            return String::new();
        }

        let mut context = String::from("\n\n## Available Skills\n\n");
        context.push_str("You have access to the following skills. You can use them by executing the shell commands described in their instructions.\n\n");

        for skill in &self.skills {
            context.push_str(&format!("### Skill: {}\n", skill.name));
            context.push_str(&format!("**Description**: {}\n\n", skill.description));
            context.push_str(&format!("{}\n\n", skill.instructions));
            context.push_str("---\n\n");
        }
        context
    }
}
