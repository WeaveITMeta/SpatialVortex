#!/bin/bash
# Download benchmark datasets

set -e

BENCHMARK_DIR="benchmarks/data"
mkdir -p "$BENCHMARK_DIR"

echo "Downloading benchmark datasets..."

# FB15k-237 Knowledge Graph
echo "📦 Downloading FB15k-237..."
mkdir -p "$BENCHMARK_DIR/fb15k237"
curl -L "https://download.microsoft.com/download/8/7/0/8700516A-AB3D-4850-B4BB-805C515AECE1/FB15K-237.2.zip" -o "$BENCHMARK_DIR/fb15k237.zip"
unzip -o "$BENCHMARK_DIR/fb15k237.zip" -d "$BENCHMARK_DIR/fb15k237"
rm "$BENCHMARK_DIR/fb15k237.zip"
echo "✓ FB15k-237 downloaded"

# WN18RR Knowledge Graph
echo "📦 Downloading WN18RR..."
mkdir -p "$BENCHMARK_DIR/wn18rr"
curl -L "https://github.com/TimDettmers/ConvE/raw/master/WN18RR.tar.gz" -o "$BENCHMARK_DIR/wn18rr.tar.gz"
tar -xzf "$BENCHMARK_DIR/wn18rr.tar.gz" -C "$BENCHMARK_DIR/wn18rr"
rm "$BENCHMARK_DIR/wn18rr.tar.gz"
echo "✓ WN18RR downloaded"

# STS Benchmark
echo "📦 Downloading STS Benchmark..."
mkdir -p "$BENCHMARK_DIR/sts"
curl -L "http://ixa2.si.ehu.es/stswiki/images/4/48/Stsbenchmark.tar.gz" -o "$BENCHMARK_DIR/stsbenchmark.tar.gz"
tar -xzf "$BENCHMARK_DIR/stsbenchmark.tar.gz" -C "$BENCHMARK_DIR/sts"
rm "$BENCHMARK_DIR/stsbenchmark.tar.gz"
echo "✓ STS Benchmark downloaded"

# SICK Dataset
echo "📦 Downloading SICK..."
mkdir -p "$BENCHMARK_DIR/sick"
if curl -L "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_train.txt" -o "$BENCHMARK_DIR/sick/SICK_train.txt" && \
   curl -L "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_trial.txt" -o "$BENCHMARK_DIR/sick/SICK_trial.txt" && \
   curl -L "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_test_annotated.txt" -o "$BENCHMARK_DIR/sick/SICK_test.txt"; then
    echo "✓ SICK downloaded"
else
    echo "⚠ SICK download failed, skipping..."
    echo "# SICK dataset unavailable" > "$BENCHMARK_DIR/sick/README.txt"
fi

# SQuAD 2.0
echo "📦 Downloading SQuAD 2.0..."
mkdir -p "$BENCHMARK_DIR/squad"
curl -L "https://rajpurkar.github.io/SQuAD-explorer/dataset/dev-v2.0.json" -o "$BENCHMARK_DIR/squad/dev-v2.0.json"
echo "✓ SQuAD 2.0 downloaded"

# CommonsenseQA
echo "📦 Downloading CommonsenseQA..."
mkdir -p "$BENCHMARK_DIR/commonsenseqa"
curl -L "https://s3.amazonaws.com/commensenseqa/dev_rand_split.jsonl" -o "$BENCHMARK_DIR/commonsenseqa/dev.jsonl"
echo "✓ CommonsenseQA downloaded"

# bAbI Tasks
echo "📦 Downloading bAbI..."
mkdir -p "$BENCHMARK_DIR/babi"
curl -L "http://www.thespermwhale.com/jaseweston/babi/tasks_1-20_v1-2.tar.gz" -o "$BENCHMARK_DIR/babi.tar.gz"
tar -xzf "$BENCHMARK_DIR/babi.tar.gz" -C "$BENCHMARK_DIR/babi"
rm "$BENCHMARK_DIR/babi.tar.gz"
echo "✓ bAbI downloaded"

# CLUTRR
echo "📦 Downloading CLUTRR..."
mkdir -p "$BENCHMARK_DIR/clutrr"
if curl -L "https://huggingface.co/datasets/CLUTRR/v1/raw/main/data_089907f8/test.csv" -o "$BENCHMARK_DIR/clutrr/test.csv" 2>/dev/null; then
    echo "✓ CLUTRR downloaded"
else
    echo "⚠ CLUTRR skipped (requires generation - see https://github.com/facebookresearch/clutrr)"
    echo "# CLUTRR requires generation" > "$BENCHMARK_DIR/clutrr/README.txt"
fi

# Silesia Corpus (for compression)
echo "📦 Downloading Silesia Corpus..."
mkdir -p "$BENCHMARK_DIR/silesia"
curl -L "http://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip" -o "$BENCHMARK_DIR/silesia.zip"
unzip -o "$BENCHMARK_DIR/silesia.zip" -d "$BENCHMARK_DIR/silesia"
rm "$BENCHMARK_DIR/silesia.zip"
echo "✓ Silesia Corpus downloaded"

echo ""
echo "✅ All benchmark datasets downloaded successfully!"
echo "📁 Data location: $BENCHMARK_DIR"
echo ""
echo "Next steps:"
echo "  1. Run: ./benchmarks/scripts/verify_datasets.sh"
echo "  2. Run: cargo test --release --package benchmarks"
