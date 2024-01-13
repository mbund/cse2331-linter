use std::{
    collections::HashSet,
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    vec,
};

use regex::Regex;
use tree_sitter::{Node, Parser, Query, QueryCursor, Range};

#[derive(Debug)]
struct Lint<'a> {
    message: String,
    text: String,
    range: Range,
    file: &'a Path,
    sublints: Option<Vec<Lint<'a>>>,
}

impl Lint<'_> {
    fn print(&self, parent: &Path) -> String {
        format!(
            "{}:{}:{} {} `{}`",
            parent.join(self.file).to_str().unwrap(),
            self.range.start_point.row + 1,
            self.range.start_point.column + 1,
            self.message,
            self.text
        )
    }
}

fn lint_real_source<'a>(file: &'a Path, source: &str, lints: &mut Vec<Lint<'a>>) {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(source, None).unwrap();
    let root_node = tree.root_node();

    let mut cursor = root_node.walk();
    for node in root_node.children(&mut cursor) {
        // top level declarations are global variables, and disallowed
        if node.kind() == "declaration" {
            let declarator = node.child_by_field_name("declarator").unwrap();
            if declarator.kind() == "init_declarator" || declarator.kind() == "identifier" {
                lints.push(Lint {
                    text: source[node.range().start_byte..node.range().end_byte].to_string(),
                    message: "Offending global variable".to_string(),
                    range: node.range(),
                    file,
                    sublints: None,
                })
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
                let declarator_range = node.child_by_field_name("declarator").unwrap().range();
                lints.push(Lint {
                    text: source[declarator_range.start_byte..declarator_range.end_byte]
                        .to_string(),
                    message: "Missing comment directly above function".to_string(),
                    range: declarator_range,
                    file,
                    sublints: None,
                })
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum IdentifierCase {
    LowerSnake,
    Camel,
}

#[derive(Debug)]
struct Identifier<'a> {
    file: &'a Path,
    range: Range,
    case: IdentifierCase,
}

fn lint_identifiers<'a>(
    file: &'a Path,
    source: &str,
    lints: &mut Vec<Lint<'a>>,
    identifiers: &mut Vec<Identifier<'a>>,
) {
    let query = Query::new(
        tree_sitter_c::language(),
        r#"
        (declaration (identifier) @identifier)
        (declaration (init_declarator (identifier) @identifier))
        (parameter_list (parameter_declaration (identifier) @identifier))
        (preproc_def) @preproc
        (preproc_function_def) @preproc
        "#,
    )
    .unwrap();

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(&source, None).unwrap();

    let mut query_cursor = QueryCursor::new();
    let all_matches = query_cursor.matches(&query, tree.root_node(), source.as_bytes());

    let screaming_snake_case_regex = Regex::new(r"^[A-Z0-9_]+$").unwrap();
    let lower_snake_case_regex = Regex::new(r"^[a-z0-9_]+_[a-z0-9_]+$").unwrap();
    let camel_case_regex = Regex::new(r"^[a-z]+(?:[A-Z][a-z0-9]*)+$").unwrap();

    for m in all_matches {
        for capture in m.captures {
            match capture.node.kind() {
                "preproc_def" | "preproc_function_def" => {
                    let identifier = capture.node.child_by_field_name("name").unwrap();
                    let range = identifier.range();
                    let text = &source[range.start_byte..range.end_byte];
                    if !screaming_snake_case_regex.is_match(text) {
                        lints.push(Lint {
                            text: source
                                [identifier.range().start_byte..identifier.range().end_byte]
                                .to_string(),
                            message: "Macro is not SCREAMING_SNAKE_CASE".to_string(),
                            range,
                            file,
                            sublints: None,
                        })
                    }
                }
                "identifier" => {
                    let range = capture.node.range();
                    let text = &source[range.start_byte..range.end_byte];
                    if lower_snake_case_regex.is_match(text) {
                        identifiers.push(Identifier {
                            case: IdentifierCase::LowerSnake,
                            file,
                            range,
                        });
                    } else if camel_case_regex.is_match(text) {
                        identifiers.push(Identifier {
                            case: IdentifierCase::Camel,
                            file,
                            range,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

fn lint_preproccessed_debug() {}

fn lint_preproccessed_nondebug<'a>(file: &'a Path, source: &str, lints: &mut Vec<Lint<'a>>) {
    let source = preprocess(source, false);

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(&source, None).unwrap();
    let root_node = tree.root_node();

    let mut cursor = root_node.walk();
    for node in root_node.children(&mut cursor) {
        if node.kind() == "function_definition" {
            let body_node = node.child_by_field_name("body").unwrap();
            let mut sublints: Vec<Lint<'a>> = vec![];
            let linecount = count_lines_compound_statement(file, &source, body_node, &mut sublints);
            if linecount > 10 {
                let declarator_range = node.child_by_field_name("declarator").unwrap().range();
                lints.push(Lint {
                    text: source[declarator_range.start_byte..declarator_range.end_byte]
                        .to_string(),
                    message: format!("Function has more than 10 lines ({})", linecount),
                    range: declarator_range,
                    file,
                    sublints: Some(sublints),
                })
            }
        }
    }
}

fn count_lines_statement<'a>(
    file: &'a Path,
    source: &str,
    node: Node,
    sublints: &mut Vec<Lint<'a>>,
) -> usize {
    let mut linecount = 0;
    match node.kind() {
        "declaration" => {
            let declarator = node.child_by_field_name("declarator");
            if let Some(d) = declarator {
                if d.kind() == "init_declarator" {
                    let range = d.range();
                    let value = range.start_point.row - range.start_point.row + 1;
                    linecount += value;
                    sublints.push(Lint {
                        file,
                        range,
                        message: "Definition".to_string(),
                        text: source[range.start_byte..range.end_byte].to_string(),
                        sublints: None,
                    });
                }
            }
        }
        "if_statement" => {
            linecount += count_lines_if_statement(file, source, node, sublints);
        }
        "while_statement" => {
            let condition = node.child_by_field_name("condition").unwrap();
            let condition_range = condition.range();
            let value = condition_range.end_point.row - condition_range.start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range: condition_range,
                message: "While".to_string(),
                text: source[condition_range.start_byte..condition_range.end_byte].to_string(),
                sublints: None,
            });

            let body = node.child_by_field_name("body").unwrap();
            linecount += count_lines_statement(file, source, body, sublints);
        }
        "do_statement" => {
            let body = node.child_by_field_name("body").unwrap();
            linecount += count_lines_statement(file, source, body, sublints);

            let condition = node.child_by_field_name("condition").unwrap();
            let condition_range = condition.range();
            let value = condition_range.end_point.row - condition_range.start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range: condition_range,
                message: "Do".to_string(),
                text: source[condition_range.start_byte..condition_range.end_byte].to_string(),
                sublints: None,
            });
        }
        "for_statement" => {
            let num_children = node.child_count();
            let first_node = node.child(0).unwrap();
            let penultimate_node = node.child(num_children - 2).unwrap();
            let body = node.child(num_children - 1).unwrap();

            let range = first_node.range();
            let value =
                penultimate_node.range().end_point.row - first_node.range().start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range,
                message: "For".to_string(),
                text: source[range.start_byte..range.end_byte].to_string(),
                sublints: None,
            });

            linecount += count_lines_statement(file, source, body, sublints);
        }
        "switch_statement" => {
            let condition = node.child_by_field_name("condition").unwrap();
            let condition_range = condition.range();
            let value = condition_range.end_point.row - condition_range.start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range: condition_range,
                message: "Switch".to_string(),
                text: source[condition_range.start_byte..condition_range.end_byte].to_string(),
                sublints: None,
            });

            let body = node.child_by_field_name("body").unwrap();
            linecount += count_lines_statement(file, source, body, sublints);
        }
        "expression_statement" => {
            let expression = node.child(0).unwrap();
            let expression_range = expression.range();
            let value = expression_range.start_point.row - expression_range.start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range: expression_range,
                message: "Expression".to_string(),
                text: source[expression_range.start_byte..expression_range.end_byte].to_string(),
                sublints: None,
            });
        }
        "case_statement" => {
            let mut count = |node: Node| {
                let mut cursor = node.walk();
                for node in node.children(&mut cursor) {
                    if node.kind() != "break_statement" {
                        linecount += count_lines_statement(file, source, node, sublints);
                    }
                }
            };

            let expression = node.child(node.child_count() - 1).unwrap();
            if expression.kind() == "compound_statement" {
                count(expression);
            } else {
                count(node);
            }
        }
        "break_statement" => {
            let range = node.range();
            let value = range.start_point.row - range.start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range,
                message: "Break".to_string(),
                text: source[range.start_byte..range.end_byte].to_string(),
                sublints: None,
            });
        }
        "continue_statement" => {
            let range = node.range();
            let value = range.start_point.row - range.start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range,
                message: "Continue".to_string(),
                text: source[range.start_byte..range.end_byte].to_string(),
                sublints: None,
            });
        }
        "else_clause" => {
            linecount += count_lines_statement(file, source, node.child(1).unwrap(), sublints);
        }
        "return_statement" => {
            let identifier = node.child(1).unwrap();
            let identifier_range = identifier.range();
            let value = identifier_range.start_point.row - identifier_range.start_point.row + 1;
            linecount += value;
            sublints.push(Lint {
                file,
                range: identifier_range,
                message: "Break".to_string(),
                text: source[identifier_range.start_byte..identifier_range.end_byte].to_string(),
                sublints: None,
            });
        }
        "compound_statement" => {
            linecount += count_lines_compound_statement(file, source, node, sublints);
        }
        _ => {}
    }
    return linecount;
}

fn count_lines_compound_statement<'a>(
    file: &'a Path,
    source: &str,
    node: Node,
    sublints: &mut Vec<Lint<'a>>,
) -> usize {
    let mut linecount = 0;

    let mut cursor = node.walk();
    for node in node.children(&mut cursor) {
        linecount += count_lines_statement(file, source, node, sublints);
    }

    return linecount;
}

fn count_lines_if_statement<'a>(
    file: &'a Path,
    source: &str,
    node: Node,
    sublints: &mut Vec<Lint<'a>>,
) -> usize {
    let mut linecount = 0;

    let condition = node.child_by_field_name("condition").unwrap();
    let condition_range = condition.range();
    let value = condition_range.end_point.row - condition_range.start_point.row + 1;
    linecount += value;
    sublints.push(Lint {
        file,
        range: condition_range,
        message: "If".to_string(),
        text: source[condition_range.start_byte..condition_range.end_byte].to_string(),
        sublints: None,
    });

    let consequence = node.child_by_field_name("consequence").unwrap();
    linecount += count_lines_statement(file, source, consequence, sublints);

    if let Some(alt) = node.child_by_field_name("alternative") {
        linecount += count_lines_statement(file, source, alt, sublints);
    }

    return linecount;
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

fn preprocess(source: &str, debug: bool) -> String {
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
    // println!("{}", reconstructed);

    return reconstructed;
}

fn main() {
    let filename = "c-example/main.c";
    let path = PathBuf::from(filename);
    let parent = path.parent().unwrap();
    env::set_current_dir(parent).unwrap();
    let local_path = PathBuf::from(path.file_name().unwrap());

    let mut identifiers: Vec<Identifier> = vec![];
    let mut lints: Vec<Lint> = vec![];

    let fileset = discover_files(local_path);
    let mut files: Vec<PathBuf> = fileset.into_iter().collect();
    files.sort();
    for file in files.iter() {
        let source = fs::read_to_string(file).unwrap();
        lint_real_source(file, &source, &mut lints);
        lint_preproccessed_nondebug(file, &source, &mut lints);
        lint_identifiers(file, &source, &mut lints, &mut identifiers);
    }

    lints.sort_by_key(|lint| lint.range.start_point.row);
    lints.iter().for_each(|lint| {
        println!("{}", lint.print(parent));
        for (i, sublint) in lint.sublints.iter().flatten().enumerate() {
            println!("  {}) {}", i + 1, sublint.print(parent));
        }
    });

    let snake_case_identifiers = identifiers
        .iter()
        .filter(|i| i.case == IdentifierCase::LowerSnake)
        .collect::<Vec<&Identifier>>();

    let camel_case_identifiers = identifiers
        .iter()
        .filter(|i| i.case == IdentifierCase::Camel)
        .collect::<Vec<&Identifier>>();

    // if snake_case_identifiers.len() > 0 && camel_case_identifiers.len() > 0 {
    //     for identifier in identifiers.iter() {
    //         let source = fs::read_to_string(&identifier.file).unwrap();
    //         let text = &source[identifier.range.start_byte..identifier.range.end_byte];

    //         let t = if identifier.case == IdentifierCase::LowerSnake {
    //             "snake_case"
    //         } else {
    //             "camelCase"
    //         };
    //         println!(
    //             "{}:{} Inconsistent identifier case {t}: `{text}`",
    //             identifier.file.to_str().unwrap(),
    //             identifier.range.start_point.row + 1
    //         );
    //     }
    // }

    // println!("{:#?}", identifiers);
}
