use std::{
    collections::HashSet,
    env, fs,
    io::{Read, Stderr, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use regex::Regex;
use tree_sitter::{Parser, Range};

fn print_error(error: &'static str, file: &Path, source: &str, range: Range) {
    let text = &source[range.start_byte..range.end_byte];
    let line = range.start_point.row;
    let col = range.start_point.column;
    println!(
        "{}:{}:{col} {error}: `{text}`",
        file.to_str().unwrap(),
        line + 1
    );
}

fn lint_real_source(file: &Path) {
    let source = fs::read_to_string(&file).unwrap();

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(&source, None).unwrap();
    let root_node = tree.root_node();

    let mut cursor = root_node.walk();
    for node in root_node.children(&mut cursor) {
        // top level declarations are global variables, and disallowed
        if node.kind() == "declaration" {
            let declarator = node.child_by_field_name("declarator").unwrap();
            if declarator.kind() == "init_declarator" || declarator.kind() == "identifier" {
                print_error("Offending global variable", file, &source, node.range())
            }
        }

        // function declarations must have comments above them
        if node.kind() == "function_definition" {
            let prev_sibling = node
                .prev_sibling()
                .expect("Failed to find function declaration's previous node");
            if !(prev_sibling.kind() == "comment"
                && node.range().start_point.row - 1 == prev_sibling.range().end_point.row)
            {
                print_error(
                    "Missing comment directly above function",
                    file,
                    &source,
                    node.child_by_field_name("declarator").unwrap().range(),
                )
            }
        }
    }
}

fn lint_preproccessed_debug() {}

fn lint_preproccessed_nondebug(file: &Path) {
    let source = preprocess(file, false);

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(&source, None).unwrap();
    let root_node = tree.root_node();

    // println!("{:#?}", root_node.to_sexp());
}

fn discover_files(path: PathBuf) -> HashSet<PathBuf> {
    let mut fileset = HashSet::new();
    fileset.insert(path.clone());

    let parent = path.parent().unwrap();

    let source = fs::read_to_string(path.clone()).unwrap();
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading C grammar");
    let tree = parser.parse(&source, None).unwrap();
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();
    for node in root_node.children(&mut cursor) {
        if node.kind() == "preproc_include" {
            let path_node = node.child_by_field_name("path").unwrap();
            if path_node.kind() == "string_literal" {
                let range = path_node.range();
                let include_path = &source[range.start_byte + 1..range.end_byte - 1];
                if !fileset.contains(&PathBuf::from(include_path)) {
                    let newfiles = discover_files(parent.join(include_path));
                    fileset.extend(newfiles);
                }
            }
        }
    }

    return fileset;
}

fn preprocess(file: &Path, debug: bool) -> String {
    let source = fs::read_to_string(file).unwrap();

    let args: Vec<&'static str> = if debug {
        ["-E", "-", "-D", "DEBUG"].to_vec()
    } else {
        ["-E", "-"].to_vec()
    };

    let process = Command::new("gcc")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute GCC preprocessor");

    process
        .stdin
        .expect("Failed to open stdin")
        .write_all(&source.as_bytes())
        .expect("Failed to write C code to stdin");

    let mut output = String::new();
    process.stdout.unwrap().read_to_string(&mut output).unwrap();

    let regex = Regex::new(r#" (?<line>\d+) "(?<path>[^#"]+)"( \d+)*(?<src>[^#]+)#?"#).unwrap();
    let mut reconstructed = String::new();
    regex
        .captures_iter(&output)
        .filter(|c| c.name("path").unwrap().as_str() == "<stdin>")
        .for_each(|c| {
            let line = c.name("line").unwrap().as_str().parse::<usize>().unwrap();
            let src = c.name("src").unwrap().as_str();
            reconstructed.extend((reconstructed.lines().count()..line).map(|_| "\n"));
            reconstructed += src;
        });
    reconstructed = reconstructed.chars().skip(2).collect();

    return reconstructed;
}

fn main() {
    let filename = "c-example/main.c";
    let path = PathBuf::from(filename);
    env::set_current_dir(path.parent().unwrap()).unwrap();
    let local_path = PathBuf::from(path.file_name().unwrap());

    let fileset = discover_files(local_path);
    let mut files: Vec<PathBuf> = fileset.into_iter().collect();
    files.sort();
    for file in files {
        lint_real_source(&file);
        lint_preproccessed_nondebug(&file);
    }
}
