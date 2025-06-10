#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if version argument is provided
if [ $# -eq 0 ]; then
    echo -e "${RED}❌ Usage: $0 <version>${NC}"
    echo -e "${YELLOW}   Example: $0 v1.0.0${NC}"
    exit 1
fi

VERSION=$1

echo -e "${BLUE}🚀 Starting release process for version ${VERSION}...${NC}"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ GitHub CLI (gh) is not installed${NC}"
    echo -e "${YELLOW}   Install it from: https://cli.github.com/${NC}"
    exit 1
fi

# Check if user is authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ Not authenticated with GitHub CLI${NC}"
    echo -e "${YELLOW}   Run: gh auth login${NC}"
    exit 1
fi

# Build the project
echo -e "${YELLOW}📦 Building project...${NC}"
./scripts/build.sh

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

# Check if dist files exist
if [ ! -f "dist/ps-payload-injector-linux" ] || [ ! -f "dist/ps-payload-injector-windows.exe" ]; then
    echo -e "${RED}❌ Build files not found in dist/ directory${NC}"
    exit 1
fi

echo -e "${YELLOW}🏷️  Creating git tag...${NC}"
# Create and push tag
git tag $VERSION
git push origin $VERSION

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to create or push tag${NC}"
    exit 1
fi

# Ask for custom release notes
echo -e "${YELLOW}📝 Release Notes:${NC}"
echo -e "${BLUE}Would you like to add custom release notes? (y/n)${NC}"
read -r add_custom_notes

CUSTOM_NOTES=""
if [[ $add_custom_notes =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Enter your release notes (press Ctrl+D when finished):${NC}"
    CUSTOM_NOTES=$(cat)
    echo -e "${GREEN}✅ Custom notes added${NC}"
fi

# Generate release notes
RELEASE_NOTES="## PS Payload Injector $VERSION"

if [ ! -z "$CUSTOM_NOTES" ]; then
    RELEASE_NOTES="$RELEASE_NOTES

### 📋 What's New
$CUSTOM_NOTES"
fi

RELEASE_NOTES="$RELEASE_NOTES

### 🎯 Features
- Cross-platform GUI application for network payload injection
- Built with Rust and egui  
- Supports both Linux and Windows

### 📥 Downloads
- **Linux**: \`ps-payload-injector-linux\`
- **Windows**: \`ps-payload-injector-windows.exe\`

### 🚀 Installation

#### Linux
\`\`\`bash
chmod +x ps-payload-injector-linux
./ps-payload-injector-linux
\`\`\`

#### Windows  
Download and double-click \`ps-payload-injector-windows.exe\`

**Note**: Windows may show a security warning. Click \"More info\" → \"Run anyway\"

### 📁 File Sizes
- Linux: $(du -h dist/ps-payload-injector-linux | cut -f1)
- Windows: $(du -h dist/ps-payload-injector-windows.exe | cut -f1)"

echo -e "${YELLOW}📋 Creating GitHub release...${NC}"
# Create the release with files
gh release create $VERSION \
    dist/ps-payload-injector-linux \
    dist/ps-payload-injector-windows.exe \
    --title "Release $VERSION" \
    --notes "$RELEASE_NOTES"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Release $VERSION created successfully!${NC}"
    echo -e "${BLUE}🔗 View at: $(gh repo view --web --json url --jq .url)/releases/tag/$VERSION${NC}"
else
    echo -e "${RED}❌ Failed to create release${NC}"
    exit 1
fi 