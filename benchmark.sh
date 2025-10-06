
#!/bin/bash
set -e

# ----------------------
# Config
# ----------------------
SRC_ORIGINAL="/Users/sochetra.nov/Documents/workspace/personal/react-native-app"  # Original folder
TEST_FOLDER="/Users/sochetra.nov/Documents/workspace/personal/react-native-app-copy"           # Temp folder for deletion test
FAST_RM_BINARY="./target/release/fast-rm"
FAST_CP_BINARY="./target/release/fast-copy" 

# ----------------------
# Build Rust release
# ----------------------
echo ""
echo "🚀 Building Rust release..."
cargo build --release

# ----------------------
# Test fast-rm
# ----------------------
echo "📂 Preparing test folder for fast-rm..."
rm -rf "$TEST_FOLDER"
"$FAST_CP_BINARY" "$SRC_ORIGINAL" "$TEST_FOLDER"
echo "\n"

echo "🗑 Running fast-rm..."
START_RUST=$(date +%s.%N)
"$FAST_RM_BINARY" "$TEST_FOLDER"
END_RUST=$(date +%s.%N)
DURATION_RUST=$(echo "$END_RUST - $START_RUST" | bc)
echo "⏱ fast-rm took: $DURATION_RUST seconds"
echo "\n"

# ----------------------
# Test system rm -rf
# ----------------------
echo "📂 Restoring test folder for system rm..."
"$FAST_CP_BINARY" "$SRC_ORIGINAL" "$TEST_FOLDER"
echo "\n"

echo "🗑 Running system rm -rf..."
START_RM=$(date +%s.%N)
rm -rf "$TEST_FOLDER"
END_RM=$(date +%s.%N)
DURATION_RM=$(echo "$END_RM - $START_RM" | bc)
echo "⏱ rm -rf took: $DURATION_RM seconds"
echo "\n"

# ----------------------
# Compare speed
# ----------------------
if (( $(echo "$DURATION_RUST < $DURATION_RM" | bc -l) )); then
    SPEEDUP=$(echo "$DURATION_RM / $DURATION_RUST" | bc -l)
    echo "⚡ fast-rm is approximately ${SPEEDUP}x faster than system rm"
else
    SPEEDUP=$(echo "$DURATION_RUST / $DURATION_RM" | bc -l)
    echo "⚡ system rm is approximately ${SPEEDUP}x faster than fast-rm"
fi
echo "\n"
