

## Command documentation

### `clear`
Clears the terminal
**Description:** Clears the terminal by calling the function clear_terminal and prints the intro animation again with the eponimous function.

### `quit`
Quits the programm
**Description:** Quits the programm, by saving the session and exiting the loop in main.rs.

**Hint**
Functions double checks with the user before quitting. This can be skipped by adding -f (force).

### Helper Functions

#### `clear_terminal`
Prints \x1b[2J\x1b[1;1H
\x1b[2J clears the entire screen
\x1b[1;1H moves the cursor to row 1, column 1, so the next output starts at the top

#### `copy_to_clipboard`
Copies a string to the system clipboard using the arboard crate and schedules auto-clear.

**Description:** Writes the given text to the clipboard, prints a message, and calls `clear_clipboard_after` to erase it after 30 seconds.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `content` | `&str` | **Yes** | The text that should be copied. |

**Hint:**

Used by the password generator to reduce manual copy errors.

**Example:**

```rust
copy_to_clipboard("my-password")?;
```


#### `clear_clipboard_after`
Clears clipboard content after a delay.

**Description:** Spawns a background thread, waits for the given number of seconds, and replaces the clipboard content with an empty string.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `duration` | `u64` | **Yes** | Delay in seconds before clearing. |

**Hint:**

This is best-effort: if clipboard access fails, it silently skips.

**Example:**

```rust
clear_clipboard_after(30);
```


#### `url_matches`
Compares two URLs by their domain.

**Description:** Extracts the host portion of both inputs (ignoring a leading `www.`) and returns `true` if they match exactly.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `entry_url` | `&str` | **Yes** | URL stored in the entry. |
| `target_url` | `&str` | **Yes** | URL to compare against. |

**Hint:**

This allows matching `https://github.com/login` with `github.com`.

**Example:**

```rust
assert!(url_matches("https://github.com/login", "github.com"));
```


#### `extract_domain`
Extracts the domain from a URL or hostname.

**Description:** Ensures a scheme is present, parses the URL, and returns the host without a leading `www.`. If parsing fails, it returns the input as-is.

**Parameter:**

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `url` | `&str` | **Yes** | The URL or hostname to normalize. |

**Hint:**

Used internally by `url_matches` to normalize input.

**Example:**

```rust
assert_eq!(extract_domain("https://www.example.com/login"), "example.com");
```


