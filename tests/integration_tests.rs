use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn code_search_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/code-search")
}

fn test_fixtures_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_test_files() {
    let fixtures = test_fixtures_dir();
    
    // Create test Python file
    fs::write(
        fixtures.join("test.py"),
        r#"
def hello_world():
    """A simple function."""
    print("Hello, World!")

def calculate_sum(a, b):
    """Calculate sum of two numbers."""
    return a + b

class Calculator:
    def __init__(self):
        self.value = 0
    
    def add(self, x):
        self.value += x
        return self
"#,
    )
    .unwrap();

    // Create test JavaScript file
    fs::write(
        fixtures.join("test.js"),
        r#"
function greet(name) {
    console.log(`Hello, ${name}!`);
}

const useState = (initial) => {
    let state = initial;
    return [state, (newState) => { state = newState; }];
};

class Component {
    constructor() {
        this.state = {};
    }
    
    render() {
        return null;
    }
}
"#,
    )
    .unwrap();

    // Create test Rust file
    fs::write(
        fixtures.join("test.rs"),
        r#"
fn main() {
    println!("Hello, World!");
}

pub fn calculate(x: i32, y: i32) -> i32 {
    x + y
}

pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn add(&mut self, x: i32) -> &mut Self {
        self.value += x;
        self
    }
}
"#,
    )
    .unwrap();
}

#[test]
fn test_plain_search() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&["search", "function", &test_fixtures_dir().to_string_lossy()])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("function"), "Should find 'function' in test files");
}

#[test]
fn test_regex_search() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "--mode", "regex",
            r"def\s+\w+",
            &test_fixtures_dir().to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("def"), "Should find function definitions");
}

#[test]
fn test_case_insensitive() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "-i",
            "HELLO",
            &test_fixtures_dir().to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello") || stdout.contains("hello"), "Should find case-insensitive match");
}

#[test]
fn test_json_output() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "--output", "json",
            "Calculator",
            &test_fixtures_dir().to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"total_matches\""), "Should output valid JSON");
    assert!(stdout.contains("\"path\""), "JSON should contain path field");
}

#[test]
fn test_count_mode() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "-c",
            "def",
            &test_fixtures_dir().to_string_lossy(),
            "-l", "python",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Count mode should succeed");
}

#[test]
fn test_files_only_mode() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "--files-only",
            "function",
            &test_fixtures_dir().to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should only show filenames, not line content
    assert!(stdout.contains(".js"), "Should list JS file");
}

#[test]
fn test_language_filter() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "function",
            &test_fixtures_dir().to_string_lossy(),
            "-l", "javascript",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test.js"), "Should find JavaScript files");
    assert!(!stdout.contains("test.py"), "Should not search Python files");
}

#[test]
fn test_max_results() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "self",
            &test_fixtures_dir().to_string_lossy(),
            "-n", "3",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check that the summary shows exactly 3 matches
    assert!(stdout.contains("Found 3 matches"), "Should limit to 3 results");
}

#[test]
fn test_init_command() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&["init", &test_fixtures_dir().to_string_lossy()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Init should succeed");
    let stdout = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Index saved"), "Should save index");
    
    // Verify index file exists
    let index_path = test_fixtures_dir().join(".code-search/bm25_index.json");
    assert!(index_path.exists(), "Index file should be created");
}

#[test]
fn test_semantic_search() {
    setup_test_files();
    
    // First init the index
    Command::new(code_search_bin())
        .args(&["init", &test_fixtures_dir().to_string_lossy()])
        .output()
        .expect("Failed to init");
    
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "--mode", "semantic",
            "calculator add value",
            &test_fixtures_dir().to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Calculator") || stdout.contains("add"), "Should find relevant results");
}

#[test]
fn test_context_lines() {
    setup_test_files();
    let output = Command::new(code_search_bin())
        .args(&[
            "search",
            "-C", "2",
            "hello_world",
            &test_fixtures_dir().to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("simple function"), "Should include context lines");
}