# Seed Corpus for Fuzzing

This directory contains seed inputs for the JSONPath fuzzers.

## Valid JSONPath Examples
```
$
$.store.book[*]
$..author
$.store.book[?(@.price < 10)]
```

## Malformed JSONPath Examples
```
$[
$..
$['unclosed
```

To use these seeds, copy them to the corpus directory:
```bash
mkdir -p fuzz/corpus/jsonpath_parse
cp corpus_seeds/*.txt fuzz/corpus/jsonpath_parse/
```