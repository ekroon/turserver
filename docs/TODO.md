# Turserver

## Project Overview

Turserver is a webserver written in Rust that serves files from Turso embedded database replicas. It leverages the power and efficiency of Rust combined with the distributed capabilities of Turso to create a fast, reliable file-serving solution.

### What is Turso?

[Turso](https://turso.tech/) is a distributed database built on libSQL, a SQLite fork. It enables creating embedded database replicas that can be distributed globally, offering low-latency access regardless of location.

### Project Goals

- Create a lightweight, high-performance file server that uses Turso embedded replicas as the storage backend
- Provide simple APIs for file management (upload, retrieve, update, delete)
- Ensure reliable performance with minimal resource consumption
- Support content type detection and appropriate serving of files

## Architecture

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   Client    │ ───────▶│    Axum     │ ───────▶│    Turso    │
│   Request   │         │   Server    │         │  Database   │
└─────────────┘         └─────────────┘         └─────────────┘
                              │
                              ▼
                        ┌─────────────┐
                        │    Files    │
                        │    Table    │
                        └─────────────┘
```

The server will use Axum, a Rust web framework, to handle HTTP requests. Files will be stored in and served from a Turso database table, with appropriate metadata.

## Implementation Roadmap

### Phase 1: Basic Setup and Core Functionality

- [X] Create basic Axum server
  - Set up routing
  - Implement error handling
  - Configure logging
- [X] Setup CI
  - bin/ci file
  - GitHub action
- [X] Create basic SQLite / Turso setup in code
  - Establish database connection
  - Add test cases
- [X] Enhance Turso setup
  - Handle connection pooling
  - Set up initialization process
- [ ] Turso embedded replica

### Phase 2: Database Schema and File Management

- [ ] Create SQLite migration for a `files` table with:
  - `id` - Unique identifier
  - `path` - Virtual path of the file
  - `content` - Binary content of the file
  - `content_type` - MIME type
  - `last_modified` - Timestamp of last modification
  - `size` - File size in bytes
  - `created_at` - Creation timestamp

### Phase 3: API Implementation

- [ ] Serve `/{path}` from files table in database
  - Implement path resolution logic
  - Set appropriate content-type headers
  - Handle not found errors
- [ ] Implement file upload endpoint
- [ ] Implement file deletion endpoint
- [ ] Add basic authentication/authorization

### Phase 4: Performance Optimization and Advanced Features

- [ ] Implement caching strategy
- [ ] Add compression for text-based content
- [ ] Implement chunked file transfers for large files
- [ ] Add metrics and monitoring

## Development Setup

*To be added as development progresses.*

## Resources

- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Turso Documentation](https://docs.turso.tech/)
- [SQLx for Rust](https://github.com/launchbadge/sqlx)