File Indexing Pipeline – Overview

This tool scans a directory recursively and indexes all files not excluded by .gitignore.
Each file is classified as source code, documentation, or ignored, based on its extension.
Binary or unreadable files are skipped automatically.
For each relevant file, the tool estimates its token count (useful for AI embedding).
Collected metadata (file path, type, token count) is normalized for further processing or search.
Efficient and extensible: easily adapt detection rules or add new processing steps.
Perfect for code search, documentation, or AI-based tools.
No configuration needed—just point to your project folder!
Ready for integration into any developer workflow.