#!/bin/bash
# Verify benchmark datasets are properly downloaded

set -e

BENCHMARK_DIR="benchmarks/data"

echo "Verifying benchmark datasets..."
echo ""

# Function to check file exists and size
check_file() {
    local file=$1
    local min_size=$2
    
    if [ -f "$file" ]; then
        local size=$(wc -c < "$file")
        if [ $size -gt $min_size ]; then
            echo "✓ $file ($(numfmt --to=iec-i --suffix=B $size))"
            return 0
        else
            echo "✗ $file (too small: $(numfmt --to=iec-i --suffix=B $size))"
            return 1
        fi
    else
        echo "✗ $file (missing)"
        return 1
    fi
}

# FB15k-237
echo "Checking FB15k-237..."
check_file "$BENCHMARK_DIR/fb15k237/train.txt" 1000000
check_file "$BENCHMARK_DIR/fb15k237/valid.txt" 100000
check_file "$BENCHMARK_DIR/fb15k237/test.txt" 100000
echo ""

# WN18RR
echo "Checking WN18RR..."
check_file "$BENCHMARK_DIR/wn18rr/train.txt" 500000
check_file "$BENCHMARK_DIR/wn18rr/test.txt" 100000
echo ""

# STS
echo "Checking STS Benchmark..."
check_file "$BENCHMARK_DIR/sts/stsbenchmark/sts-test.csv" 100000
echo ""

# SICK
echo "Checking SICK..."
check_file "$BENCHMARK_DIR/sick/SICK_test.txt" 50000
echo ""

# SQuAD
echo "Checking SQuAD 2.0..."
check_file "$BENCHMARK_DIR/squad/dev-v2.0.json" 1000000
echo ""

# CommonsenseQA
echo "Checking CommonsenseQA..."
check_file "$BENCHMARK_DIR/commonsenseqa/dev.jsonl" 100000
echo ""

# bAbI
echo "Checking bAbI..."
check_file "$BENCHMARK_DIR/babi/tasks_1-20_v1-2/en-10k/qa1_single-supporting-fact_test.txt" 1000
echo ""

# CLUTRR
echo "Checking CLUTRR..."
if [ -f "$BENCHMARK_DIR/clutrr/test.csv" ]; then
    check_file "$BENCHMARK_DIR/clutrr/test.csv" 10000
else
    echo "⚠ CLUTRR (optional - skipped)"
fi
echo ""

# Silesia
echo "Checking Silesia Corpus..."
check_file "$BENCHMARK_DIR/silesia/dickens" 1000000
echo ""

echo "✅ Dataset verification complete!"
echo ""
echo "Ready to run benchmarks:"
echo "  cargo test --release --package benchmarks"
