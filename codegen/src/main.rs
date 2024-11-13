use std::collections::HashMap;
use std::io;

use swc::config::{IsModule, SourceMapsConfig};
use swc::PrintArgs;
use swc_common::sync::Lrc;
use swc_common::{
    errors::Handler,
    FileName, SourceMap,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::Config;
use swc_ecma_parser::Syntax;

const CODE: &str = r#"
export class Person extends Human {
    name: string;
    age: number;
    brother: Person;

    constructor(name: string, age: number) {
        this.name = name;
        this.age = age;
    }

    async greet(): string {
        return `Hello, my name is ${this.name} and I am ${this.age} years old.`;
    }
}
"#;

fn main() -> anyhow::Result<()> {
    let cm: Lrc<SourceMap> = Default::default();
    let compiler = swc::Compiler::new(cm.clone());
    let handler = Handler::with_emitter_writer(Box::new(io::stderr()), Some(compiler.cm.clone()));

    let source = cm.new_source_file(FileName::Custom("test.ts".into()).into(), CODE.into());

    let program = compiler
        .parse_js(
            source,
            &handler,
            EsVersion::Es5,
            Syntax::Typescript(Default::default()),
            IsModule::Bool(true),
            Some(compiler.comments()),
        )
        .expect("parse_js failed");
    println!("{:?}", program);

    let ast_printed = compiler
        .print(
            // &Program::Script(Script{
            //     span: Default::default(),
            //     body: vec![kg_codegen::sample::test_class()],
            //     // directives: vec![],
            //     shebang: None,
            // }),
            &program,
            PrintArgs {
                source_root: None,
                source_file_name: None,
                output_path: None,
                inline_sources_content: false,
                source_map: SourceMapsConfig::default(),
                source_map_names: &HashMap::default(),
                orig: None,
                comments: None,
                emit_source_map_columns: false,
                preamble: "",
                codegen_config: Config::default().with_target(EsVersion::latest()),
                output: None,
            },
        )
        .expect("Failed to print");

    println!("{}", ast_printed.code);

    Ok(())
}
