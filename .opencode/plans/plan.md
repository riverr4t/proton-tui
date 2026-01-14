# Plan to Cache Server List

To improve startup time and reduce API calls, I will implement a caching mechanism for the server list.

## 1. Create Caching Module (`src/server_cache.rs`)
I will create a new module `src/server_cache.rs` that handles loading and saving the server list to disk.
- **Location**: The cache file will be stored in the user's cache directory (e.g., `~/.cache/proton-tui/servers_cache.json`) or config directory if cache is unavailable.
- **Expiration**: The cache will be valid for **15 minutes** by default.
- **Structure**:
  ```rust
  struct ServerCache {
      timestamp: u64,
      servers: Vec<LogicalServer>,
  }
  ```

## 2. Integrate Caching in `src/main.rs`
I will modify `src/main.rs` to attempt loading servers from the cache before querying the API.

- **Logic Flow**:
  1.  **Startup**: Check if a valid cache file exists.
  2.  **Cache Hit**: If valid (exists and < 15 mins old), load servers from cache.
  3.  **Cache Miss**: If invalid or missing:
      - Show "Loading servers..." screen.
      - Fetch from Proton VPN API.
      - Save the result to cache.
  4.  **Sort**: Proceed with existing sorting and app initialization.

## 3. Implementation Details
- Add `mod server_cache;` to `src/main.rs`.
- Use the `dirs` crate (already a dependency) to locate the cache directory.
- Handle errors gracefully (fall back to API if cache fails).

This approach ensures the app works offline (if cache is valid) or starts much faster on subsequent runs within the cache window.
