use crate::commands::format::run;
use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::{Label, print_changes};
use clap_verbosity_flag::{log::Level, Verbosity};
use color_eyre::eyre::Result;
use mock::MockResearchActivity;
use pretty_assertions::assert_eq;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

// Mock modules
mod mock {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    // Mock for ResearchActivity to avoid actual file operations
    pub struct MockResearchActivity;

    // File system operations tracking
    pub struct FileOps {
        pub read_calls: Vec<PathBuf>,
        pub write_calls: HashMap<PathBuf, String>,
        pub format_calls: Vec<PathBuf>,
        pub read_results: HashMap<PathBuf, Result<String, String>>,
    }

    impl FileOps {
        pub fn new() -> Self {
            FileOps {
                read_calls: Vec::new(),
                write_calls: HashMap::new(),
                format_calls: Vec::new(),
                read_results: HashMap::new(),
            }
        }

        pub fn add_read_result(&mut self, path: PathBuf, content: Result<String, String>) {
            self.read_results.insert(path, content);
        }
    }

    // Global mock state for testing
    thread_local! {
        pub static FILE_OPS: RefCell<FileOps> = RefCell::new(FileOps::new());
    }

    // Global shared state for parallel tests
    pub static SHARED_FILE_OPS: once_cell::sync::Lazy<Arc<Mutex<FileOps>>> = 
        once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(FileOps::new())));
}

// Mock implementations for std::fs
#[cfg(test)]
mod std {
    pub mod fs {
        use super::super::mock;
        use std::io::{Error, ErrorKind, Read, Result, Write};
        use std::path::{Path, PathBuf};

        pub struct File {
            path: PathBuf,
            content: String,
        }

        impl File {
            pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
                let path_buf = path.as_ref().to_path_buf();
                mock::SHARED_FILE_OPS.with(|ops| {
                    let mut ops = ops.lock().unwrap();
                    ops.write_calls.insert(path_buf.clone(), String::new());
                });
                Ok(File {
                    path: path_buf,
                    content: String::new(),
                })
            }
        }

        impl Write for File {
            fn write(&mut self, buf: &[u8]) -> Result<usize> {
                let content = match std::str::from_utf8(buf) {
                    Ok(s) => s.to_string(),
                    Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8")),
                };
                self.content = content.clone();
                mock::SHARED_FILE_OPS.with(|ops| {
                    let mut ops = ops.lock().unwrap();
                    ops.write_calls.insert(self.path.clone(), content);
                });
                Ok(buf.len())
            }

            fn flush(&mut self) -> Result<()> {
                Ok(())
            }
        }
    }
}

// Mock for ResearchActivity
impl MockResearchActivity {
    pub fn read(path: PathBuf) -> serde_json::Result<ResearchActivity> {
        // Record the call
        mock::SHARED_FILE_OPS.with(|ops| {
            let mut ops = ops.lock().unwrap();
            ops.read_calls.push(path.clone());
        });

        // Return mocked content based on path
        mock::SHARED_FILE_OPS.with(|ops| {
            let ops = ops.lock().unwrap();
            match ops.read_results.get(&path) {
                Some(Ok(content)) => {
                    // Valid JSON content
                    let ra: ResearchActivity = serde_json::from_str(content).unwrap_or_else(|_| {
                        panic!("Invalid mock JSON for path: {:?}", path)
                    });
                    Ok(ra)
                },
                Some(Err(err)) => {
                    // Error case
                    Err(serde_json::Error::custom(err.clone()))
                },
                None => {
                    // Default error for unknown paths
                    Err(serde_json::Error::custom(format!("No mock data for path: {:?}", path)))
                }
            }
        })
    }
}

// Extension trait for ResearchActivity to add format method
trait ResearchActivityExt {
    fn format(self, path: Option<PathBuf>) -> Self;
}

impl ResearchActivityExt for ResearchActivity {
    fn format(self, path: Option<PathBuf>) -> Self {
        if let Some(p) = path {
            mock::SHARED_FILE_OPS.with(|ops| {
                let mut ops = ops.lock().unwrap();
                ops.format_calls.push(p);
            });
        }
        // Return a modified version for testing
        self
    }
}

// Test helpers
fn setup_test_environment() -> (tempdir::TempDir, Vec<PathBuf>) {
    // Create a temporary directory for test files
    let temp_dir = tempdir().expect("Failed to create temp dir");
    
    // Create test files
    let file_paths = vec![
        temp_dir.path().join("test1.json"),
        temp_dir.path().join("test2.json"),
        temp_dir.path().join("test3.json"),
    ];
    
    // Reset mock state
    mock::SHARED_FILE_OPS.with(|ops| {
        let mut ops = ops.lock().unwrap();
        *ops = mock::FileOps::new();
    });
    
    (temp_dir, file_paths)
}

fn create_test_verbosity(level: Level) -> Verbosity {
    let mut verbosity = Verbosity::new();
    // Set the verbosity level
    for _ in 0..match level {
        Level::Error => 0,
        Level::Warn => 1,
        Level::Info => 2,
        Level::Debug => 3,
        Level::Trace => 4,
    } {
        verbosity.verbose_more();
    }
    verbosity
}

fn setup_mock_read_results(file_paths: &[PathBuf], valid_content: bool) {
    mock::SHARED_FILE_OPS.with(|ops| {
        let mut ops = ops.lock().unwrap();
        
        for path in file_paths {
            if valid_content {
                let content = format!(
                    r#"{{
                    "meta": {{ "id": "test-{}", "keywords": [] }},
                    "title": "Test Title",
                    "contact": {{
                        "title": "Test",
                        "first": "Test",
                        "last": "User",
                        "email": "test@example.com",
                        "phone": "123-456-7890",
                        "profile": "https://example.com",
                        "organization": "Test Org"
                    }},
                    "sections": {{ "project": {{
                        "introduction": "Test introduction text that is long enough.",
                        "challenge": "Test challenge text that is long enough for testing purposes.",
                        "approach": "Test approach text that is long enough for testing and validation.",
                        "outcomes": ["Test outcome 1", "Test outcome 2"],
                        "research": {{
                            "focus": "Test research focus that meets requirements.",
                            "areas": ["Area 1", "Area 2"]
                        }}
                    }} }}
                }}"#,
                    path.file_name().unwrap().to_string_lossy()
                );
                ops.add_read_result(path.clone(), Ok(content));
            } else {
                ops.add_read_result(path.clone(), Err("Invalid JSON".to_string()));
            }
        }
    });
}

// Tests
#[test]
fn test_dry_run_mode() -> Result<()> {
    let (temp_dir, file_paths) = setup_test_environment();
    setup_mock_read_results(&file_paths, true);
    
    // Test with dry run enabled
    let verbose = create_test_verbosity(Level::Error);
    run(&Some(temp_dir.path().to_path_buf()), true, &verbose)?;
    
    // Verify behavior
    mock::SHARED_FILE_OPS.with(|ops| {
        let ops = ops.lock().unwrap();
        assert!(!ops.read_calls.is_empty(), "Files should be read");
        assert!(ops.write_calls.is_empty(), "No files should be written in dry run mode");
        assert!(!ops.format_calls.is_empty(), "Format should be called");
    });
    
    Ok(())
}

#[test]
fn test_verbosity_levels() -> Result<()> {
    // Test with different verbosity levels
    let verbosity_levels = vec![
        Level::Error,
        Level::Warn,
        Level::Info,
        Level::Debug,
        Level::Trace,
    ];
    
    for level in verbosity_levels {
        let (temp_dir, file_paths) = setup_test_environment();
        setup_mock_read_results(&file_paths, true);
        
        let verbose = create_test_verbosity(level);
        run(&Some(temp_dir.path().to_path_buf()), false, &verbose)?;
        
        // For levels above Error, changes should be printed
        let should_print_changes = level > Level::Error;
        
        // We can't directly test print_changes output, but we can verify
        // the correct behavior based on file operations
        mock::SHARED_FILE_OPS.with(|ops| {
            let ops = ops.lock().unwrap();
            assert!(!ops.read_calls.is_empty(), "Files should be read");
            assert!(!ops.write_calls.is_empty(), "Files should be written");
        });
    }
    
    Ok(())
}

#[test]
fn test_file_reading_and_writing() -> Result<()> {
    let (temp_dir, file_paths) = setup_test_environment();
    setup_mock_read_results(&file_paths, true);
    
    let verbose = create_test_verbosity(Level::Error);
    run(&Some(temp_dir.path().to_path_buf()), false, &verbose)?;
    
    // Verify file operations
    mock::SHARED_FILE_OPS.with(|ops| {
        let ops = ops.lock().unwrap();
        
        // Check that files were read
        for path in &file_paths {
            assert!(
                ops.read_calls.contains(path),
                "File should be read: {:?}", path
            );
        }
        
        // Check that files were written
        for path in &file_paths {
            assert!(
                ops.write_calls.contains_key(path),
                "File should be written: {:?}", path
            );
        }
        
        // Format should be called for each file
        for path in &file_paths {
            assert!(
                ops.format_calls.contains(path),
                "Format should be called for: {:?}", path
            );
        }
    });
    
    Ok(())
}

#[test]
fn test_content_formatting_and_comparison() -> Result<()> {
    let (temp_dir, file_paths) = setup_test_environment();
    setup_mock_read_results(&file_paths, true);
    
    // Set up a specific test case
    let test_path = &file_paths[0];
    let original_content = r#"{
        "meta": { "id": "test-original", "keywords": [] },
        "title": "Original Title",
        "contact": {
            "title": "Test",
            "first": "Test",
            "last": "User",
            "email": "test@example.com",
            "phone": "123-456-7890",
            "profile": "https://example.com",
            "organization": "Test Org"
        },
        "sections": { "project": {
            "introduction": "Original introduction text that is long enough.",
            "challenge": "Original challenge text that is long enough for testing purposes.",
            "approach": "Original approach text that is long enough for testing and validation.",
            "outcomes": ["Original outcome 1", "Original outcome 2"],
            "research": {
                "focus": "Original research focus that meets requirements.",
                "areas": ["Area 1", "Area 2"]
            }
        } }
    }"#;
    
    let formatted_content = r#"{
        "meta": { "id": "test-formatted", "keywords": [] },
        "title": "Formatted Title",
        "contact": {
            "title": "Test",
            "first": "Test",
            "last": "User",
            "email": "test@example.com",
            "phone": "123-456-7890",
            "profile": "https://example.com",
            "organization": "Test Org"
        },
        "sections": { "project": {
            "introduction": "Formatted introduction text that is long enough.",
            "challenge": "Formatted challenge text that is long enough for testing purposes.",
            "approach": "Formatted approach text that is long enough for testing and validation.",
            "outcomes": ["Formatted outcome 1", "Formatted outcome 2"],
            "research": {
                "focus": "Formatted research focus that meets requirements.",
                "areas": ["Area 1", "Area 2"]
            }
        } }
    }"#;
    
    mock::SHARED_FILE_OPS.with(|ops| {
        let mut ops = ops.lock().unwrap();
        ops.add_read_result(test_path.clone(), Ok(original_content.to_string()));
    });
    
    let verbose = create_test_verbosity(Level::Debug);
    run(&Some(temp_dir.path().to_path_buf()), false, &verbose)?;
    
    // Since we can't directly test the print_changes output,
    // we can verify the write operations happened correctly
    mock::SHARED_FILE_OPS.with(|ops| {
        let ops = ops.lock().unwrap();
        assert!(ops.read_calls.contains(test_path));
        assert!(ops.write_calls.contains_key(test_path));
    });
    
    Ok(())
}

#[test]
fn test_error_conditions() -> Result<()> {
    let (temp_dir, file_paths) = setup_test_environment();
    
    

