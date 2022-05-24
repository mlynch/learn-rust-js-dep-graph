#![allow(dead_code)]
#[macro_use]
extern crate swc_common;
extern crate swc_ecma_parser;
use std::path::Path;
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    /*FileName, FilePathMapping,*/ SourceMap,
};
use swc_ecma_ast::{Decl, ExportDecl, ImportDecl, Module, ModuleDecl, Program, Stmt};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

use std::collections::HashMap;

struct Graph {
    map: HashMap<String, Vec<String>>,
}

impl Graph {
    fn new() -> Graph {
        Graph {
            map: HashMap::new(),
        }
    }

    fn push_local_dep(&mut self, file_name: String, dep: String) {
        self._push(file_name, dep);
    }
    fn push_library_dep(&mut self, file_name: String, dep: String) {
        self._push(file_name, dep);
    }

    fn _push(&mut self, file_name: String, dep: String) {
        if self.map.contains_key(&file_name) {
            let m = &mut self.map;
            //let m = self.map.get(&file_name).unwrap();
            if let Some(v) = m.get_mut(&file_name) {
                v.push(dep);
            }
        } else {
            let v = vec![dep];
            self.map.insert(file_name.clone(), v);
        }
    }
}

fn parse(
    file_name: &str,
    parser: &mut Parser<Lexer<StringInput>>,
    handler: Handler,
    graph: &mut Graph,
) {
    println!("Parsing {}", file_name);
    let program = parser
        .parse_program()
        .map_err(|mut e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");

    process_program(file_name, &program, graph);
}

fn process_program(file_name: &str, program: &Program, graph: &mut Graph) {
    if program.is_module() {
        println!("Got a module");
        process_module(file_name, program.as_module(), graph);
    }
}

fn process_module(file_name: &str, module: Option<&Module>, graph: &mut Graph) {
    let body = &module.expect("Unable to unwrap").body;

    for item in body {
        if item.is_stmt() {
            visit_stmt(item.as_stmt());
        }
        if item.is_module_decl() {
            visit_module_decl(file_name, item.as_module_decl(), graph);
        }
    }
}

fn visit_module_decl(file_name: &str, module_decl: Option<&ModuleDecl>, graph: &mut Graph) {
    let decl = module_decl.expect("Unable to unwrap module decl");

    match decl {
        ModuleDecl::Import(ImportDecl) => {
            println!("Import decl");
            visit_import_decl(file_name, decl.as_import(), graph);
        }
        ModuleDecl::ExportDecl(ExportDecl) => {
            println!("Export decl");
        }
        _ => {
            println!("Other decl");
        }
    }
}

fn visit_import_decl(file_name: &str, import_decl: Option<&ImportDecl>, graph: &mut Graph) {
    let decl = import_decl.expect("Unable to unwrap import decl");

    println!("Import {}", decl.src.value);

    let val = &decl.src.value;

    if val.chars().next().unwrap() == '.' && val.contains(".js") {
        graph.push_local_dep(String::from(file_name), val.to_string().clone());
    } else {
        graph.push_library_dep(String::from(file_name), val.to_string().clone());
    }

    /*
    for s in decl.specifiers {

    }
    */
}

fn visit_stmt(stmt: Option<&Stmt>) {
    let statement = stmt.expect("Unable to unwrap statement");

    match statement {
        Decl => {
            println!("Got a decl");
        }
        _ => {
            println!("Not a decl");
        }
    }
}

fn parse_file(file_name: &str, graph: &mut Graph) {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // Real usage
    let fm = cm
        .load_file(Path::new("app/index.js"))
        .expect("failed to load index.js");

    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    parse(file_name, &mut parser, handler, graph);
}

fn print_graph(graph: &Graph) {
    println!("Graph has {} items", graph.map.len());

    for file in graph.map.keys() {
        if let Some(deps) = graph.map.get(file) {
            println!("{}: {}", file, deps.len());
            for dep in deps {
                println!("\t{}", dep);
            }
        }
    }
}

fn main() {
    let mut graph = Graph::new();

    println!("Building dependency graph...");

    parse_file("index.js", &mut graph);

    print_graph(&graph);
}
