//! Integration tests for fusefiles
//!
//! These tests match the functionality of the Python pytest suite

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a command for our binary
fn cmd() -> Command {
    Command::cargo_bin("fuse").unwrap()
}

/// Extract filenames from XML output using simple string matching
fn filenames_from_cxml(cxml_string: &str) -> Vec<String> {
    let mut filenames = Vec::new();
    for line in cxml_string.lines() {
        if line.trim().starts_with("<source>") && line.trim().ends_with("</source>") {
            let content = line
                .trim()
                .strip_prefix("<source>")
                .unwrap()
                .strip_suffix("</source>")
                .unwrap();
            filenames.push(content.to_string());
        }
    }
    filenames
}

#[test]
fn test_basic_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("file1.txt"), "Contents of file1").unwrap();
    fs::write(test_dir.join("file2.txt"), "Contents of file2").unwrap();

    let assert = cmd().arg(&test_dir).assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();

    eprintln!("Test dir: {:?}", test_dir);
    eprintln!("STDOUT: '{}'", stdout);
    eprintln!("STDERR: '{}'", stderr);

    // Check for specific content
    assert!(!stdout.is_empty(), "stdout should not be empty");
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("Contents of file1"));
    assert!(stdout.contains("file2.txt"));
    assert!(stdout.contains("Contents of file2"));
    assert!(stdout.contains("---"));
}

#[test]
fn test_include_hidden() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join(".hidden.txt"), "Contents of hidden file").unwrap();

    // Test without --include-hidden
    let output = cmd()
        .arg(&test_dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let expected_hidden = test_dir.join(".hidden.txt").to_string_lossy().to_string();
    assert!(!stdout.contains(&expected_hidden));

    // Test with --include-hidden
    let output = cmd()
        .arg(&test_dir)
        .arg("--include-hidden")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains(&expected_hidden));
    assert!(stdout.contains("Contents of hidden file"));
}

#[test]
fn test_ignore_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir_all(&test_dir).unwrap();
    fs::create_dir_all(test_dir.join("nested_include")).unwrap();
    fs::create_dir_all(test_dir.join("nested_ignore")).unwrap();

    fs::write(test_dir.join(".gitignore"), "ignored.txt").unwrap();
    fs::write(test_dir.join("ignored.txt"), "This file should be ignored").unwrap();
    fs::write(
        test_dir.join("included.txt"),
        "This file should be included",
    )
    .unwrap();
    fs::write(
        test_dir.join("nested_include").join("included2.txt"),
        "This nested file should be included",
    )
    .unwrap();

    fs::write(
        test_dir.join("nested_ignore").join(".gitignore"),
        "nested_ignore.txt",
    )
    .unwrap();
    fs::write(
        test_dir.join("nested_ignore").join("nested_ignore.txt"),
        "This nested file should not be included",
    )
    .unwrap();
    fs::write(
        test_dir.join("nested_ignore").join("actually_include.txt"),
        "This nested file should actually be included",
    )
    .unwrap();

    let output = cmd()
        .arg(&test_dir)
        .arg("-c")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let filenames = filenames_from_cxml(&stdout);

    let expected_included = test_dir
        .join("included.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    let expected_nested_included = test_dir
        .join("nested_include")
        .join("included2.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    let expected_actually_include = test_dir
        .join("nested_ignore")
        .join("actually_include.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    let expected_ignored = test_dir
        .join("ignored.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    assert!(filenames.contains(&expected_included));
    assert!(filenames.contains(&expected_nested_included));
    assert!(filenames.contains(&expected_actually_include));
    assert!(!filenames.contains(&expected_ignored));

    // Test with --ignore-gitignore
    let output = cmd()
        .arg(&test_dir)
        .arg("-c")
        .arg("--ignore-gitignore")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let filenames = filenames_from_cxml(&stdout);

    let expected_included = test_dir
        .join("included.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    let expected_ignored = test_dir
        .join("ignored.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    let expected_nested_included = test_dir
        .join("nested_include")
        .join("included2.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    let expected_nested_ignored = test_dir
        .join("nested_ignore")
        .join("nested_ignore.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    let expected_actually_include = test_dir
        .join("nested_ignore")
        .join("actually_include.txt")
        .to_string_lossy()
        .to_string()
        .to_string();
    assert!(filenames.contains(&expected_included));
    assert!(filenames.contains(&expected_ignored));
    assert!(filenames.contains(&expected_nested_included));
    assert!(filenames.contains(&expected_nested_ignored));
    assert!(filenames.contains(&expected_actually_include));
}

#[test]
fn test_multiple_paths() {
    let temp_dir = TempDir::new().unwrap();

    let test_dir1 = temp_dir.path().join("test_dir1");
    let test_dir2 = temp_dir.path().join("test_dir2");
    fs::create_dir(&test_dir1).unwrap();
    fs::create_dir(&test_dir2).unwrap();

    fs::write(test_dir1.join("file1.txt"), "Contents of file1").unwrap();
    fs::write(test_dir2.join("file2.txt"), "Contents of file2").unwrap();
    fs::write(
        temp_dir.path().join("single_file.txt"),
        "Contents of single file",
    )
    .unwrap();

    let output = cmd()
        .arg(&test_dir1)
        .arg(&test_dir2)
        .arg(temp_dir.path().join("single_file.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let expected_path1 = test_dir1.join("file1.txt").to_string_lossy().to_string();
    let expected_path2 = test_dir2.join("file2.txt").to_string_lossy().to_string();
    let expected_single = temp_dir
        .path()
        .join("single_file.txt")
        .to_string_lossy()
        .to_string();
    assert!(stdout.contains(&expected_path1));
    assert!(stdout.contains("Contents of file1"));
    assert!(stdout.contains(&expected_path2));
    assert!(stdout.contains("Contents of file2"));
    assert!(stdout.contains(&expected_single));
    assert!(stdout.contains("Contents of single file"));
}

#[test]
fn test_ignore_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(
        test_dir.join("file_to_ignore.txt"),
        "This file should be ignored due to ignore patterns",
    )
    .unwrap();
    fs::write(
        test_dir.join("file_to_include.txt"),
        "This file should be included",
    )
    .unwrap();

    let output = cmd()
        .arg(&test_dir)
        .arg("--ignore")
        .arg("*ignore*")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let expected_include = test_dir
        .join("file_to_include.txt")
        .to_string_lossy()
        .to_string();
    assert!(!stdout.contains("file_to_ignore.txt"));
    assert!(!stdout.contains("This file should be ignored due to ignore patterns"));
    assert!(stdout.contains(&expected_include));
    assert!(stdout.contains("This file should be included"));
}

#[test]
fn test_specific_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    let two_dir = test_dir.join("two");
    fs::create_dir_all(&two_dir).unwrap();

    fs::write(test_dir.join("one.txt"), "This is one.txt").unwrap();
    fs::write(test_dir.join("one.py"), "This is one.py").unwrap();
    fs::write(two_dir.join("two.txt"), "This is two/two.txt").unwrap();
    fs::write(two_dir.join("two.py"), "This is two/two.py").unwrap();
    fs::write(test_dir.join("three.md"), "This is three.md").unwrap();

    let output = cmd()
        .arg(&test_dir)
        .arg("-e")
        .arg("py")
        .arg("-e")
        .arg("md")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let expected_one_py = test_dir.join("one.py").to_string_lossy().to_string();
    let expected_two_py = test_dir
        .join("two")
        .join("two.py")
        .to_string_lossy()
        .to_string();
    let expected_three_md = test_dir.join("three.md").to_string_lossy().to_string();
    assert!(!stdout.contains(".txt"));
    assert!(stdout.contains(&expected_one_py));
    assert!(stdout.contains(&expected_two_py));
    assert!(stdout.contains(&expected_three_md));
}

#[test]
fn test_binary_file_warning() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    // Create a binary file
    fs::write(test_dir.join("binary_file.bin"), [0xff, 0xfe, 0xfd]).unwrap();
    fs::write(test_dir.join("text_file.txt"), "This is a text file").unwrap();

    let assert = cmd().arg(&test_dir).assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();

    let expected_text = test_dir.join("text_file.txt").to_string_lossy().to_string();
    assert!(stdout.contains(&expected_text));
    assert!(stdout.contains("This is a text file"));
    assert!(!stdout.contains("binary_file.bin"));
    assert!(stderr.contains("Warning: Skipping binary file"));
    assert!(stderr.contains("binary_file.bin"));
}

#[test]
fn test_xml_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("file1.txt"), "Contents of file1.txt").unwrap();
    fs::write(test_dir.join("file2.txt"), "Contents of file2.txt").unwrap();

    let output = cmd()
        .arg(&test_dir)
        .arg("--cxml")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Check XML structure
    assert!(stdout.contains("<documents>"));
    assert!(stdout.contains("</documents>"));
    let expected_path1 = test_dir.join("file1.txt").to_string_lossy().to_string();
    let expected_path2 = test_dir.join("file2.txt").to_string_lossy().to_string();
    assert!(stdout.contains(r#"<document index="1">"#));
    assert!(stdout.contains(r#"<document index="2">"#));
    assert!(stdout.contains(&format!("<source>{}</source>", expected_path1)));
    assert!(stdout.contains(&format!("<source>{}</source>", expected_path2)));
    assert!(stdout.contains("<document_content>"));
    assert!(stdout.contains("</document_content>"));
    assert!(stdout.contains("Contents of file1.txt"));
    assert!(stdout.contains("Contents of file2.txt"));
}

#[test]
fn test_output_option() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("file1.txt"), "Contents of file1.txt").unwrap();
    fs::write(test_dir.join("file2.txt"), "Contents of file2.txt").unwrap();

    let output_file = temp_dir.path().join("output.txt");

    let assert = cmd()
        .arg(&test_dir)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output = assert.get_output();
    // Should have no stdout output when using -o
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(stdout.is_empty());

    // Check the output file was created and has correct content
    let file_content = fs::read_to_string(&output_file).unwrap();
    let expected_path1 = test_dir.join("file1.txt").to_string_lossy().to_string();
    let expected_path2 = test_dir.join("file2.txt").to_string_lossy().to_string();
    assert!(file_content.contains(&expected_path1));
    assert!(file_content.contains("Contents of file1.txt"));
    assert!(file_content.contains(&expected_path2));
    assert!(file_content.contains("Contents of file2.txt"));
    assert!(file_content.contains("---"));
}

#[test]
fn test_line_numbers() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    let test_content = "First line\nSecond line\nThird line\nFourth line\n";
    fs::write(test_dir.join("multiline.txt"), test_content).unwrap();

    // Test without line numbers
    let output = cmd()
        .arg(&test_dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let expected_path = test_dir.join("multiline.txt").to_string_lossy().to_string();
    assert!(!stdout.contains("1  First line"));
    assert!(stdout.contains(test_content));
    assert!(stdout.contains(&expected_path));

    // Test with line numbers
    let output = cmd()
        .arg(&test_dir)
        .arg("-n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let expected_path = test_dir.join("multiline.txt").to_string_lossy().to_string();
    assert!(stdout.contains(&expected_path));
    assert!(stdout.contains("1  First line"));
    assert!(stdout.contains("2  Second line"));
    assert!(stdout.contains("3  Third line"));
    assert!(stdout.contains("4  Fourth line"));

    // Test with --line-numbers (long form)
    let output = cmd()
        .arg(&test_dir)
        .arg("--line-numbers")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let expected_path = test_dir.join("multiline.txt").to_string_lossy().to_string();
    assert!(stdout.contains(&expected_path));
    assert!(stdout.contains("1  First line"));
    assert!(stdout.contains("2  Second line"));
    assert!(stdout.contains("3  Third line"));
    assert!(stdout.contains("4  Fourth line"));
}

#[test]
fn test_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("python.py"), "This is python").unwrap();
    fs::write(
        test_dir.join("python_with_quad_backticks.py"),
        "This is python with ```` in it already",
    )
    .unwrap();
    fs::write(test_dir.join("code.js"), "This is javascript").unwrap();
    fs::write(
        test_dir.join("code.unknown"),
        "This is an unknown file type",
    )
    .unwrap();

    let output = cmd()
        .arg(&test_dir)
        .arg("-m")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Check that each file is formatted correctly
    let expected_js = test_dir.join("code.js").to_string_lossy().to_string();
    let expected_unknown = test_dir.join("code.unknown").to_string_lossy().to_string();
    let expected_python = test_dir.join("python.py").to_string_lossy().to_string();
    let expected_quad_backticks = test_dir
        .join("python_with_quad_backticks.py")
        .to_string_lossy()
        .to_string();

    assert!(stdout.contains(&format!(
        "{}\n```javascript\nThis is javascript\n```",
        expected_js
    )));
    assert!(stdout.contains(&format!(
        "{}\n```\nThis is an unknown file type\n```",
        expected_unknown
    )));
    assert!(stdout.contains(&format!(
        "{}\n```python\nThis is python\n```",
        expected_python
    )));

    // Check that the file with backticks uses more backticks
    assert!(stdout.contains(&format!(
        "{}\n`````python\nThis is python with ```` in it already\n`````",
        expected_quad_backticks
    )));
}
