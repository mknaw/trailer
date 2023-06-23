use colored::Colorize;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::ops::Range;
use tree_sitter::{Language, Parser, TreeCursor};

// Main formatting / printing code here
// Kind of silly in that rely on sqlformat to pretty print and then go back and colorize
// based on treesitter parser. But I didn't feel like writing a pretty printer myself. :<

extern "C" {
    fn tree_sitter_sql() -> Language;
}

type ColorMap = Vec<(Range<usize>, &'static str)>;

/// Print a string with colors.
fn print_colored(s: &str, color_map: ColorMap) {
    let mut last_end = 0;
    for (range, color) in color_map {
        // Print the non-colored section
        if last_end < range.start {
            print!("{}", &s[last_end..range.start]);
        }

        // Print the colored section
        print!("{}", s[range.clone()].color(color));

        last_end = range.end;
    }

    // Print the rest of the string, if any
    if last_end < s.len() {
        print!("{}", &s[last_end..]);
    }
}

/// Handle the incremental text addeed to the log file.
pub fn process(reader: &mut BufReader<File>, parser: &mut Parser) -> io::Result<()> {
    let mut line = String::new();
    loop {
        match reader.read_line(&mut line)? {
            0 => return Ok(()),
            _ => {
                pretty_print(&line, parser);
                line.clear();
            }
        }
    }
}

/// Pretty print the SQL query.
fn pretty_print(raw_sql: &str, parser: &mut Parser) {
    let sql = sqlformat::format(
        &raw_sql[52..], // TODO should actually regex out the irrelevant stuff
        &sqlformat::QueryParams::None,
        sqlformat::FormatOptions::default(),
    );

    let tree = parser.parse(&sql, None).expect("Error parsing code");
    let mut cmap = Vec::new();
    walk_tree(&mut tree.walk(), &sql, &mut cmap);
    print_colored(&sql, cmap);
    print_delimiter();
}

/// Print a horizontal bar.
fn print_delimiter() {
    let width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    println!("{}{}", "\n", "-".repeat(width).red());
}

/// Return a SQL `Parser`.
pub fn init_parser() -> Parser {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_sql() };
    parser
        .set_language(language)
        .expect("Error setting language");
    parser
}

/// Walk the parser tree and map range to colors.
fn walk_tree(cursor: &mut TreeCursor, sql: &str, cmap: &mut ColorMap) {
    let node = cursor.node();

    // TODO here one can get much fancier.
    if node.kind().starts_with("keyword") {
        cmap.push((node.byte_range(), "green"));
    }
    if cursor.goto_first_child() {
        walk_tree(cursor, sql, cmap);
        cursor.goto_parent();
    }
    while cursor.goto_next_sibling() {
        walk_tree(cursor, sql, cmap);
    }
}
