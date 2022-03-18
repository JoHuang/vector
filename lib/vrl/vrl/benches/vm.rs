use std::collections::BTreeMap;

use compiler::state;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use indoc::indoc;
use vector_common::TimeZone;
use vrl::{Runtime, Value};

struct Source {
    name: &'static str,
    code: &'static str,
}

static SOURCES: [Source; 2] = [
    Source {
        name: "parse_json",
        code: indoc! {r#"
            x = parse_json!(s'{"noog": "nork"}')
            x.noog
        "#},
    },
    Source {
        name: "simple",
        code: indoc! {r#"
            .hostname = "vector"

            if .status == "warning" {
                .thing = upcase(.hostname)
            } else if .status == "notice" {
                .thung = downcase(.hostname)
            } else {
                .nong = upcase(.hostname)
            }

            .matches = { "name": .message, "num": "2" }
            .origin, .err = .hostname + "/" + .matches.name + "/" + .matches.num
        "#},
    },
];

fn benchmark_kind_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrl_compiler/value::kind::display");
    for source in &SOURCES {
        let state = state::Runtime::default();
        let runtime = Runtime::new(state);
        let tz = TimeZone::default();
        let functions = vrl_stdlib::all();
        let mut state = vrl::state::Compiler::new();
        let program = vrl::compile_with_state(source.code, &functions, &mut state).unwrap();
        let vm = runtime
            .compile(functions, &program, Default::default())
            .unwrap();
        let builder = vrl::llvm::Builder::new().unwrap();
        let context = builder.compile(&state, &program).unwrap();
        context.optimize();
        let execute = context.get_jit_function().unwrap();

        {
            println!("yo");
            let mut obj = Value::Object(BTreeMap::default());
            let mut context = core::Context {
                target: &mut obj,
                timezone: &tz,
            };
            let mut result = Ok(Value::Null);
            println!("bla");
            unsafe { execute.call(&mut context, &mut result) };
            println!("derp");
        }

        group.bench_with_input(
            BenchmarkId::new("LLVM", source.name),
            &execute,
            |b, execute| {
                b.iter_with_setup(
                    || Value::Object(BTreeMap::default()),
                    |mut obj| {
                        {
                            let mut context = core::Context {
                                target: &mut obj,
                                timezone: &tz,
                            };
                            let mut result = Ok(Value::Null);
                            unsafe { execute.call(&mut context, &mut result) };
                        }
                        obj // Return the obj so it doesn't get dropped.
                    },
                )
            },
        );

        group.bench_with_input(BenchmarkId::new("VM", source.name), &vm, |b, vm| {
            let state = state::Runtime::default();
            let mut runtime = Runtime::new(state);
            b.iter_with_setup(
                || Value::Object(BTreeMap::default()),
                |mut obj| {
                    let _ = black_box(runtime.run_vm(vm, &mut obj, &tz));
                    runtime.clear();
                    obj // Return the obj so it doesn't get dropped.
                },
            )
        });

        group.bench_with_input(BenchmarkId::new("Ast", source.name), &(), |b, _| {
            let state = state::Runtime::default();
            let mut runtime = Runtime::new(state);
            b.iter_with_setup(
                || Value::Object(BTreeMap::default()),
                |mut obj| {
                    let _ = black_box(runtime.resolve(&mut obj, &program, &tz));
                    runtime.clear();
                    obj
                },
            )
        });
    }
}

criterion_group!(name = vrl_compiler_kind;
                 config = Criterion::default();
                 targets = benchmark_kind_display);
criterion_main!(vrl_compiler_kind);
