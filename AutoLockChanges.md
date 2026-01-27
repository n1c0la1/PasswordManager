# AutoLock Implementation

## Changes Made

### `src/session.rs`

- Added `last_activity` timestamp to the `Session` struct.
- Implemented `check_timeout` and `update_activity` methods.
- Initialized `last_activity` when starting a session.

### `src/main.rs`

- Modified `current_session` to be thread-safe using `Arc<Mutex<Option<Session>>>`.
- Spawned a background thread that checks for inactivity every second.
- Updates session activity timestamp before and after each user command.
- Automatically closes the session and prints a logout message if inactive for 5 minutes  
 (for testing purposes, timeout is set to 10s right now.).

### `src/cli.rs`

- Fixed a bug where opening a vault after an auto-logout would fail with  
  **"COULD NOT CLOSE VAULT"**.
- This was caused by `handle_command_open` attempting to close a session that was
  already marked inactive by the auto-lock thread.
- Adjusted `handle_command_open` to gracefully handle `SessionInactive` during
  the cleanup phase.