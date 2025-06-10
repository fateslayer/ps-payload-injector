#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Building PS Payload Injector for multiple platforms...${NC}"

# Create dist directory
mkdir -p dist
rm -rf dist/*

echo -e "${YELLOW}📦 Building for Linux (x86_64)...${NC}"
cargo build --release --target x86_64-unknown-linux-gnu
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Linux build successful${NC}"
    cp target/x86_64-unknown-linux-gnu/release/ps-payload-injector dist/ps-payload-injector-linux
else
    echo -e "${RED}❌ Linux build failed${NC}"
    exit 1
fi

echo -e "${YELLOW}🪟 Building for Windows (x86_64)...${NC}"
cargo build --release --target x86_64-pc-windows-gnu
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Windows build successful${NC}"
    cp target/x86_64-pc-windows-gnu/release/ps-payload-injector.exe dist/ps-payload-injector-windows.exe
else
    echo -e "${RED}❌ Windows build failed${NC}"
    exit 1
fi

echo -e "${GREEN}🎉 Build complete! Executables are in the 'dist' folder:${NC}"
ls -la dist/

echo -e "${BLUE}📁 File sizes:${NC}"
du -h dist/* 