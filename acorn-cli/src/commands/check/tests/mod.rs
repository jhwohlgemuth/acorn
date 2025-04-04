use crate::commands::check;
use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::cli::Check;
use mockall::predicate::*;
use mockall::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Create mock for static ResearchActivity methods
mock! {
    static ResearchActivity;

    impl ResearchActivity {
        pub fn check(paths: Vec<PathBuf>) -> usize;
        pub fn analyze(paths: Vec<PathBuf>) -> bool;
    }
}

// Helper function to setup the test environment
fn setup_test() {
    // Reset any static mocks before each test
    MockResearchActivity::check_context().reset();
    MockResearchActivity::analyze_context().reset();
}

#[test]
fn test_run_with_valid_path() {
    setup_test();
    
    // Setup expectations
    MockResearchActivity::check_context()
        .expect()
        .return_const(0usize);
    
    MockResearchActivity::analyze_context()
        .expect()
        .return_const(true);
    
    // Create a valid path for testing
    let path = Some(PathBuf::from("./valid_path"));
    let skip = vec![];
    let exit_on_first_error = false;
    
    // Run function under test
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    // Verify result
    assert!(result.is_ok());
}

#[test]
fn test_run_with_invalid_path() {
    setup_test();
    
    // Create a non-existent path
    let path = None;
    let skip = vec![];
    let exit_on_first_error = false;
    
    // Run function under test
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    // Should return Ok since the function just returns if path is None
    assert!(result.is_ok());
}

#[test]
fn test_skip_validation() {
    setup_test();
    
    // We expect analyze to be called but not check
    MockResearchActivity::analyze_context()
        .expect()
        .return_const(true);
    
    // check should not be called when validation is skipped
    MockResearchActivity::check_context()
        .expect()
        .never();
    
    let path = Some(PathBuf::from("./test_path"));
    let skip = vec![Check::Validation];
    let exit_on_first_error = false;
    
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    assert!(result.is_ok());
}

#[test]
fn test_skip_analysis() {
    setup_test();
    
    // We expect check to be called but not analyze
    MockResearchActivity::check_context()
        .expect()
        .return_const(0usize);
    
    // analyze should not be called when analysis is skipped
    MockResearchActivity::analyze_context()
        .expect()
        .never();
    
    let path = Some(PathBuf::from("./test_path"));
    let skip = vec![Check::Analysis];
    let exit_on_first_error = false;
    
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    assert!(result.is_ok());
}

#[test]
fn test_skip_both_validation_and_analysis() {
    setup_test();
    
    // Neither check nor analyze should be called
    MockResearchActivity::check_context()
        .expect()
        .never();
    
    MockResearchActivity::analyze_context()
        .expect()
        .never();
    
    let path = Some(PathBuf::from("./test_path"));
    let skip = vec![Check::Validation, Check::Analysis];
    let exit_on_first_error = false;
    
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    assert!(result.is_ok());
}

#[test]
#[should_panic]
fn test_validation_error_with_exit_on_first_error() {
    setup_test();
    
    // Setup check to return errors
    MockResearchActivity::check_context()
        .expect()
        .return_const(5usize); // Return 5 errors
    
    // We should exit before analyze is called
    MockResearchActivity::analyze_context()
        .expect()
        .never();
    
    let path = Some(PathBuf::from("./test_path"));
    let skip = vec![];
    let exit_on_first_error = true;
    
    // This should panic due to process::exit
    check::run(&path, &skip, &exit_on_first_error);
}

#[test]
#[should_panic]
fn test_validation_error_without_exit_on_first_error() {
    setup_test();
    
    // Setup check to return errors
    MockResearchActivity::check_context()
        .expect()
        .return_const(5usize); // Return 5 errors
    
    // analyze should be called since we don't exit on first error
    MockResearchActivity::analyze_context()
        .expect()
        .return_const(true);
    
    let path = Some(PathBuf::from("./test_path"));
    let skip = vec![];
    let exit_on_first_error = false;
    
    // This should still panic at the end due to validation errors
    check::run(&path, &skip, &exit_on_first_error);
}

#[test]
#[should_panic]
fn test_analysis_error() {
    setup_test();
    
    // Setup check to pass but analyze to fail
    MockResearchActivity::check_context()
        .expect()
        .return_const(0usize);
    
    MockResearchActivity::analyze_context()
        .expect()
        .return_const(false);
    
    let path = Some(PathBuf::from("./test_path"));
    let skip = vec![];
    let exit_on_first_error = false;
    
    // This should panic due to analyze failure
    check::run(&path, &skip, &exit_on_first_error);
}

#[test]
fn test_multiple_files_with_no_errors() {
    setup_test();
    
    // Setup with multiple files but no errors
    MockResearchActivity::check_context()
        .expect()
        .return_const(0usize);
    
    MockResearchActivity::analyze_context()
        .expect()
        .return_const(true);
    
    let path = Some(PathBuf::from("./multiple_files"));
    let skip = vec![];
    let exit_on_first_error = false;
    
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    assert!(result.is_ok());
}

#[test]
#[should_panic]
fn test_multiple_files_with_errors() {
    setup_test();
    
    // Setup with multiple files and errors
    MockResearchActivity::check_context()
        .expect()
        .return_const(3usize); // 3 errors across multiple files
    
    MockResearchActivity::analyze_context()
        .expect()
        .return_const(true);
    
    let path = Some(PathBuf::from("./multiple_files_with_errors"));
    let skip = vec![];
    let exit_on_first_error = false;
    
    // This should panic due to validation errors
    check::run(&path, &skip, &exit_on_first_error);
}

#[test]
fn test_check_and_analyze_function_calls() {
    setup_test();
    
    // Use atomic variables to track calls to check and analyze
    static CHECK_CALLED: AtomicBool = AtomicBool::new(false);
    static ANALYZE_CALLED: AtomicBool = AtomicBool::new(false);
    static CHECK_COUNT: AtomicUsize = AtomicUsize::new(0);
    
    // Setup expectations with side effects to track calls
    MockResearchActivity::check_context()
        .expect()
        .returning(move |paths: Vec<PathBuf>| {
            CHECK_CALLED.store(true, Ordering::SeqCst);
            CHECK_COUNT.store(paths.len(), Ordering::SeqCst);
            0usize
        });
    
    MockResearchActivity::analyze_context()
        .expect()
        .returning(move |_| {
            ANALYZE_CALLED.store(true, Ordering::SeqCst);
            true
        });
    
    let path = Some(PathBuf::from("./test_path"));
    let skip = vec![];
    let exit_on_first_error = false;
    
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    assert!(result.is_ok());
    assert!(CHECK_CALLED.load(Ordering::SeqCst));
    assert!(ANALYZE_CALLED.load(Ordering::SeqCst));
    
    // Reset for a new test
    CHECK_CALLED.store(false, Ordering::SeqCst);
    ANALYZE_CALLED.store(false, Ordering::SeqCst);
    CHECK_COUNT.store(0, Ordering::SeqCst);
    
    setup_test();
    
    // Now test with specific file paths
    let test_paths: Vec<PathBuf> = vec![
        PathBuf::from("./file1.json"),
        PathBuf::from("./file2.json"),
        PathBuf::from("./file3.json"),
    ];
    
    // Mock get_all_files to return our test paths
    mock_fn!(get_all_files).expect().returning(move |_, _| test_paths.clone());
    
    // Setup expectations again
    MockResearchActivity::check_context()
        .expect()
        .returning(move |paths: Vec<PathBuf>| {
            CHECK_CALLED.store(true, Ordering::SeqCst);
            CHECK_COUNT.store(paths.len(), Ordering::SeqCst);
            0usize
        });
    
    MockResearchActivity::analyze_context()
        .expect()
        .returning(move |_| {
            ANALYZE_CALLED.store(true, Ordering::SeqCst);
            true
        });
    
    let path = Some(PathBuf::from("./test_directory"));
    let skip = vec![];
    let exit_on_first_error = false;
    
    let result = check::run(&path, &skip, &exit_on_first_error);
    
    assert!(result.is_ok());
    assert!(CHECK_CALLED.load(Ordering::SeqCst));
    assert!(ANALYZE_CALLED.load(Ordering::SeqCst));
    assert_eq!(CHECK_COUNT.load(Ordering::SeqCst), 3); // Should have processed 3 files
}

// Alternative approach using a test context structure
struct TestContext {
    path: Option<PathBuf>,
    skip: Vec<Check>,
    exit_on_first_error: bool,
    check_return: usize,
    analyze_return: bool,
}

impl TestContext {
    fn new() -> Self {
        Self {
            path: Some(PathBuf::from("./test_path")),
            skip: vec![],
            exit_on_first_error: false,
            check_return: 0,
            analyze_return: true,
        }
    }
    
    fn with_path(mut self, path: Option<PathBuf>) -> Self {
        self.path = path;
        self
    }
    
    fn with_skip(mut self, skip: Vec<Check>) -> Self {
        self.skip = skip;
        self
    }
    
    fn with_exit_on_first_error(mut self, exit_on_first_error: bool) -> Self {
        self.exit_on_first_error = exit_on_first_error;
        self
    }
    
    fn with_check_return(mut self, check_return: usize) -> Self {
        self.check_return = check_return;
        self
    }
    
    fn with_analyze_return(mut self, analyze_return: bool) -> Self {
        self.analyze_return = analyze_return;
        self
    }
    
    fn run(&self) -> Result<(), String> {
        setup_test();
        
        // Set up expectations
        MockResearchActivity::check_context()
            .expect()
            .return_const(self.check_return);
        
        MockResearchActivity::analyze_context()
            .expect()
            .return_const(self.analyze_return);
        
        // Convert the skip vec to a slice
        let skip_slice = self.skip.as_slice();
        
        // Run the function and catch any panics
        let result = std::panic::catch_unwind(|| {
            check::run(&self.path, skip_slice, &self.exit_on_first_error)
        });
        
        match result {
            Ok(inner_result) => match inner_result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Function returned an error: {:?}", e)),
            },
            Err(_) => Err("Function panicked".to_string()),
        }
    }
}

#[test]
fn test_fluent_interface_success() {
    let result = TestContext::new()
        .with_path(Some(PathBuf::from("./valid_path")))
        .with_check_return(0)
        .with_analyze_return(true)
        .run();
    
    assert!(result.is_ok());
}

#[test]
fn test_fluent_interface_validation_error() {
    let result = TestContext::new()
        .with_check_return(1)
        .run();
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Function panicked");
}

#[test]
fn test_fluent_interface_analyze_error() {
    let result = TestContext::new()
        .with_check_return(0)
        .with_analyze_return(false)
        .run();
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Function panicked");
}

#[test]
fn test_fluent_interface_skip_both() {
    let result = TestContext::new()
        .with_skip(vec![Check::Validation, Check::Analysis])
        .run();
    
    assert!(result.is_ok());
}

