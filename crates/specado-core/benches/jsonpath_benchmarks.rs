//! Comprehensive benchmarks for JSONPath engine performance
//!
//! These benchmarks test various aspects of the JSONPath implementation
//! to ensure optimal performance for common use cases.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::{json, Value};
use specado_core::translation::jsonpath::*;

fn create_test_data() -> Value {
    json!({
        "store": {
            "book": [
                {
                    "category": "reference",
                    "author": "Nigel Rees",
                    "title": "Sayings of the Century",
                    "price": 8.95
                },
                {
                    "category": "fiction",
                    "author": "Evelyn Waugh",
                    "title": "Sword of Honour",
                    "price": 12.99
                },
                {
                    "category": "fiction",
                    "author": "Herman Melville",
                    "title": "Moby Dick",
                    "isbn": "0-553-21311-3",
                    "price": 8.99
                },
                {
                    "category": "fiction",
                    "author": "J. R. R. Tolkien",
                    "title": "The Lord of the Rings",
                    "isbn": "0-395-19395-8",
                    "price": 22.99
                }
            ],
            "bicycle": {
                "color": "red",
                "price": 19.95
            }
        },
        "expensive": 10
    })
}

fn create_large_data() -> Value {
    let mut items = Vec::new();
    for i in 0..1000 {
        items.push(json!({
            "id": i,
            "name": format!("Item {}", i),
            "category": if i % 3 == 0 { "A" } else if i % 3 == 1 { "B" } else { "C" },
            "price": (i as f64) * 1.5 + 10.0,
            "in_stock": i % 2 == 0,
            "metadata": {
                "tags": ["tag1", "tag2"],
                "description": format!("Description for item {}", i)
            }
        }));
    }
    
    json!({
        "items": items,
        "total_count": 1000,
        "categories": ["A", "B", "C"]
    })
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    
    let expressions = vec![
        "$.store.book[0].title",
        "$.store.book[*].author",
        "$..author",
        "$.store.book[?(@.price < 10)]",
        "$.store.book[0,1]",
        "$.store.book[1:3]",
        "$.store.*",
    ];
    
    for expr in expressions {
        group.bench_with_input(BenchmarkId::new("parse", expr), expr, |b, expr| {
            b.iter(|| {
                let result = JSONPath::parse(black_box(expr));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_execution_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution_simple");
    let data = create_test_data();
    
    let test_cases = vec![
        ("root", "$"),
        ("simple_property", "$.store"),
        ("nested_property", "$.store.book"),
        ("array_index", "$.store.book[0]"),
        ("deep_property", "$.store.book[0].title"),
    ];
    
    for (name, expr) in test_cases {
        let jsonpath = JSONPath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("execute", name), &jsonpath, |b, path| {
            b.iter(|| {
                let result = path.execute(black_box(&data));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_execution_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution_complex");
    let data = create_test_data();
    
    let test_cases = vec![
        ("wildcard", "$.store.*"),
        ("array_wildcard", "$.store.book[*]"),
        ("recursive_descent", "$..author"),
        ("array_slice", "$.store.book[1:3]"),
        ("union", "$.store.book[0,1]"),
    ];
    
    for (name, expr) in test_cases {
        let jsonpath = JSONPath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("execute", name), &jsonpath, |b, path| {
            b.iter(|| {
                let result = path.execute(black_box(&data));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_filters(c: &mut Criterion) {
    let mut group = c.benchmark_group("filters");
    let data = create_test_data();
    
    let test_cases = vec![
        ("simple_comparison", "$.store.book[?(@.price < 10)]"),
        ("complex_filter", "$.store.book[?(@.price > 8 && @.category == 'fiction')]"),
    ];
    
    for (name, expr) in test_cases {
        let jsonpath = JSONPath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("filter", name), &jsonpath, |b, path| {
            b.iter(|| {
                let result = path.execute(black_box(&data));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_large_dataset(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_dataset");
    let data = create_large_data();
    
    let test_cases = vec![
        ("all_items", "$.items[*]"),
        ("filtered_items", "$.items[?(@.price > 500)]"),
        ("recursive_search", "$..name"),
        ("category_filter", "$.items[?(@.category == 'A')]"),
    ];
    
    for (name, expr) in test_cases {
        let jsonpath = JSONPath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("large", name), &jsonpath, |b, path| {
            b.iter(|| {
                let result = path.execute(black_box(&data));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("caching");
    let data = create_test_data();
    
    // Test the benefit of caching compiled expressions
    let expr = "$.store.book[*].author";
    
    group.bench_function("without_cache", |b| {
        b.iter(|| {
            let jsonpath = JSONPath::parse(black_box(expr)).unwrap();
            let result = jsonpath.execute(black_box(&data));
            black_box(result)
        })
    });
    
    group.bench_function("with_cache", |b| {
        let jsonpath = JSONPath::parse(expr).unwrap();
        b.iter(|| {
            let result = jsonpath.execute(black_box(&data));
            black_box(result)
        })
    });
    
    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    let data = create_test_data();
    
    let expr = "$.store.book[*].title";
    let jsonpath = JSONPath::parse(expr).unwrap();
    
    group.bench_function("execute_owned", |b| {
        b.iter(|| {
            let result = jsonpath.execute_owned(black_box(&data));
            black_box(result)
        })
    });
    
    group.bench_function("execute_references", |b| {
        b.iter(|| {
            let result = jsonpath.execute(black_box(&data));
            black_box(result)
        })
    });
    
    group.finish();
}

fn bench_compilation_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_optimization");
    
    let expressions = vec![
        "$.simple",
        "$.nested.property",
        "$.array[*]",
        "$..recursive",
        "$.complex[?(@.value > 10)]",
    ];
    
    for expr in expressions {
        group.bench_with_input(BenchmarkId::new("compile", expr), expr, |b, expr| {
            b.iter(|| {
                let result = JSONPath::parse(black_box(expr));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    let invalid_expressions = vec![
        "$.invalid[",
        "$[invalid",
        "$.test[?(@.invalid",
        "invalid_start",
    ];
    
    for expr in invalid_expressions {
        group.bench_with_input(BenchmarkId::new("invalid", expr), expr, |b, expr| {
            b.iter(|| {
                let result = JSONPath::parse(black_box(expr));
                black_box(result)
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_parsing,
    bench_execution_simple,
    bench_execution_complex,
    bench_filters,
    bench_large_dataset,
    bench_caching,
    bench_memory_efficiency,
    bench_compilation_optimization,
    bench_error_handling
);

criterion_main!(benches);