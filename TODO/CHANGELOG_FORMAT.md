# Changelog Format

This document outlines the format for changelogs in Moosync.

## Release titles
Release titles should follow the format: `v{version} - {release_name}`

Examples:
- `v2.1.0 - Harmony Update`
- `v2.0.5 - Stability Fixes`
- `v3.0.0 - Major Overhaul`

Release names should be descriptive but concise, ideally music-themed to fit the application's purpose.

## Release versioning
Follow semantic versioning (major.minor.patch):

- **Major version** (X.0.0): Breaking changes, major feature additions, significant UI/UX overhauls
- **Minor version** (X.Y.0): New features, non-breaking API changes, significant improvements
- **Patch version** (X.Y.Z): Bug fixes, security patches, minor improvements

Version bumping guidelines:
- Increment major version when making incompatible API changes or major breaking changes
- Increment minor version when adding functionality in a backwards-compatible manner
- Increment patch version when making backwards-compatible bug fixes

## Release descriptions
Each release should include the following sections in this order:

### Structure:
```markdown
## [v{version}] - {release_name} - {date}

### 🚀 New Features
- Feature descriptions with clear benefit to users
- Use bullet points for readability

### 🔧 Improvements
- Performance improvements
- UI/UX enhancements
- Code optimizations

### 🐛 Bug Fixes
- Clear description of what was broken and how it's fixed
- Reference issue numbers when applicable

### 🔒 Security
- Security-related fixes (if any)
- Follow responsible disclosure principles

### 📚 Documentation
- Documentation updates
- API changes
- Breaking changes with migration notes

### 🏗️ Technical
- Internal changes that don't affect end users
- Dependency updates
- Build system changes

### ⚠️ Breaking Changes
- List all breaking changes
- Provide migration instructions
- Include before/after examples where helpful
```

### Writing Guidelines:
- Use clear, user-friendly language
- Avoid technical jargon when possible
- Include links to relevant documentation
- Reference issue/PR numbers: `(#123)`, `(closes #456)`
- Use emojis for visual categorization
- Keep descriptions concise but informative

### Example Entry:
```markdown
## [v2.1.0] - Harmony Update - 2024-06-24

### 🚀 New Features
- Added real-time lyrics display with synchronized scrolling (#234)
- Introduced mood-based playlist generation (#267)
- Added support for FLAC audio format (#189)

### 🔧 Improvements
- Improved audio playback performance by 30% (#245)
- Enhanced search functionality with fuzzy matching (#256)
- Updated extension API with better error handling (#278)

### 🐛 Bug Fixes
- Fixed playlist shuffle not working correctly (#289)
- Resolved memory leak in audio decoder (#301)
- Fixed extension loading on Windows (#312)

### 📚 Documentation
- Updated extension development guide (#298)
- Added API reference for new lyrics feature (#305)
```
