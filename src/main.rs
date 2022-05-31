#![allow(dead_code)]
extern crate swc_common;
extern crate swc_ecma_parser;
use std::collections::HashMap;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    /*FileName, FilePathMapping,*/ SourceMap,
};
use swc_ecma_ast::{ImportDecl, Module, ModuleDecl, Program, TsCallSignatureDecl};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

use crate::graph::Graph;

mod cli;
mod graph;

struct Context<'a> {
    graph: &'a mut Graph,
    entry_file: &'a str,
    root_dir: &'a str,
}


fn parse(
    ctx: &mut Context,
    file_name: &str,
    parser: &mut Parser<Lexer<StringInput>>,
    handler: Handler
) {
    let program = parser
        .parse_program()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");

    process_program(ctx, file_name, &program);
}

fn process_program(ctx: &mut Context, file_name: &str, program: &Program) {
    if program.is_module() {
        process_module(ctx, file_name, program.as_module());
    }
}

fn process_module(ctx: &mut Context, file_name: &str, module: Option<&Module>) {
    let body = &module.expect("Unable to unwrap").body;

    for item in body {
        /*
        if item.is_stmt() {
            visit_stmt(item.as_stmt());
        }
        */
        if item.is_module_decl() {
            visit_module_decl(ctx, file_name, item.as_module_decl());
        }
    }
}

fn visit_module_decl(ctx: &mut Context, file_name: &str, module_decl: Option<&ModuleDecl>) {
    let decl = module_decl.expect("Unable to unwrap module decl");

    match decl {
        ModuleDecl::Import(_import_decl) => {
            visit_import_decl(ctx, file_name, decl.as_import());
        }
        _ => {
        }
    }
}

fn visit_import_decl(ctx: &mut Context, file_name: &str, import_decl: Option<&ImportDecl>) {
    let decl = import_decl.expect("Unable to unwrap import decl");

    let val = &decl.src.value;

    let root_dir = ctx.root_dir;

    println!("Visiting import {} {} {}", file_name, val, root_dir);

    if Path::exists(&Path::new(&root_dir).join(val.to_string())) {
        // This isn't perfect but assume the index file is tsx
        println!("local ref exists {}", val);
        let dir_path = &Path::new(&root_dir).join(val.to_string());
        let mut index_path = dir_path.join("index.tsx");

        if !Path::exists(index_path.as_path()) {
            index_path = dir_path.join("index.ts");
        }
        if !Path::exists(index_path.as_path()) {
            index_path = dir_path.join("index.js");
        }
        if !Path::exists(index_path.as_path()) {
            index_path = dir_path.join("index.jsx");
        }

        if !Path::exists(index_path.as_path()) {
            return;
        }

        println!("index path {:?}", index_path.to_str());
        ctx.graph.push_local_dep(String::from(index_path.to_str().unwrap()), val.to_string().clone());
        parse_file(ctx, &index_path.to_str().unwrap());
    }

    if Path::exists(&Path::new(&root_dir).join("..").join(val.to_string())) {
        // This isn't perfect but assume the index file is tsx
        println!("local ref dir in root exists {}", val);
        let dir_path = &Path::new(&root_dir).join("..").join(val.to_string());
        let mut index_path = dir_path.join("index.tsx");

        if !Path::exists(index_path.as_path()) {
            index_path = dir_path.join("index.ts");
        }
        if !Path::exists(index_path.as_path()) {
            index_path = dir_path.join("index.js");
        }
        if !Path::exists(index_path.as_path()) {
            index_path = dir_path.join("index.jsx");
        }
        if !Path::exists(index_path.as_path()) {
            return;
        }

        println!("index path {:?}", index_path.to_str());
        ctx.graph.push_local_dep(String::from(file_name), val.to_string().clone());
        parse_file(ctx, &index_path.to_str().unwrap());
    }

    let js_path = &Path::new(&root_dir).join(format!("{}.js", val.to_string()));
    if Path::exists(js_path) {
        println!("local ref file js in root exists {}", val);
        ctx.graph.push_local_dep(String::from(file_name), val.to_string().clone());
        parse_file(ctx, js_path.to_str().unwrap());
    }

    let ts_path = &Path::new(&root_dir).join(format!("{}.ts", val.to_string()));
    if Path::exists(ts_path) {
        println!("local ref file ts in root exists {}", val);
        ctx.graph.push_local_dep(String::from(file_name), val.to_string().clone());
        parse_file(ctx, ts_path.to_str().unwrap());
    }

    let tsx_path = &Path::new(&root_dir).join(format!("{}.tsx", val.to_string()));
    if Path::exists(tsx_path) {
        println!("local ref file tsx in root exists {}", val);
        ctx.graph.push_local_dep(String::from(file_name), val.to_string().clone());
        parse_file(ctx, tsx_path.to_str().unwrap());
    }

    if val.chars().next().unwrap() == '.' && val.contains(".js") {
        ctx.graph.push_local_dep(String::from(file_name), val.to_string().clone());
        parse_file(ctx, val);
    } else {
        ctx.graph.push_library_dep(String::from(file_name), val.to_string().clone());
    }
}

/*
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
*/

fn parse_file(ctx: &mut Context, file_name: &str) {
    if ctx.graph.seen(file_name.to_string()) {
        println!("ALREADY SEEN {}", file_name);
        return
    }

    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let pb = PathBuf::from(file_name);
    let full_path = canonicalize(pb).expect("Unable to convert path");

    println!("PARSE FILE {} {}", file_name, full_path.to_str().unwrap());

    let root_dir = Path::new(file_name).parent().unwrap();

    // Real usage
    let fm = cm
        //.load_file(Path::new(&format!("app/{}", file_name)))
        .load_file(full_path.as_path())
        .expect(&format!("failed to load {}", file_name));

    let lexer = Lexer::new(
        // We want to parse ecmascript
        //Syntax::Es(Default::default()),
        Syntax::Typescript(TsConfig {
            tsx: true,
            ..Default::default()
        }),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    parse(ctx, file_name, &mut parser, handler);
}

fn print_graph(graph: &Graph) {
    for file in graph.map.keys() {
        if let Some(deps) = graph.map.get(file) {
            println!("{}: {}", file, deps.len());
            for dep in deps {
                println!("\t{}", dep);
            }
        }
    }
}

fn print_graph_stats(graph: &Graph) {
    let mut count_map: HashMap<&String, i32> = HashMap::new();

    for vec in graph.map.values() {
        for dep in vec {
            if let Some(count) = count_map.get(dep) {
                count_map.insert(dep, count + 1);
            } else {
                count_map.insert(dep, 1);
            }
        }
    }

    let mut deps: Vec<&&String> = count_map.keys().collect();

    deps.sort_by(|a, b| {
        if let (Some(av), Some(bv)) = (count_map.get(*a), count_map.get(*b)) {
            return bv.cmp(av);
        }
        return std::cmp::Ordering::Equal;
    });

    for dep in deps {
        if let Some(count) = count_map.get(dep) {
            println!("{}: {}", dep, count);
        }
    }
}

fn main() {
    let args = cli::get_args();

    let mut graph = Graph::new();

    let root_dir = Path::new(args.entry.as_str()).parent().unwrap().parent().unwrap();

    let mut ctx = Context {
        entry_file: args.entry.as_str(),
        root_dir: root_dir.to_str().unwrap(),
        graph: &mut graph
    };

    println!("Building dependency graph, starting at {}...", ctx.entry_file);

    parse_file(&mut ctx, args.entry.as_str());

    print_graph(&graph);

    print_graph_stats(&graph);
}
