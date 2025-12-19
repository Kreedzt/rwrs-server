#!/bin/bash

# Run all tests with coverage
echo "Running tests with coverage..."

# Check if cargo-llvm-cov is installed
if ! cargo llvm-cov --version >/dev/null 2>&1; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Run tests with coverage
cargo llvm-cov --all-features --lib --html

# Show coverage report path
echo ""
echo "Coverage report generated at: target/llvm-cov/html/index.html"

# Open the coverage report if on macOS
if command -v open >/dev/null 2>&1; then
    echo ""
    echo "Opening coverage report..."
    open target/llvm-cov/html/index.html
fi