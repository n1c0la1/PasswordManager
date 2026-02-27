### AI/LLM Usage Disclosure

* **Tool:** Gemini.
**Purpose:**
  * Used for structuring the Markdown documentation, refining the command tables for readability.
  * Used for finding suitable crates.

* **Tool** Github Copilot.
**Purpose:**
  * Creation and Refactoring of PowerShell Scripts to allow Multi-Platform-Support.
  * Explanation of Syntax of Shell.
  * Code-Review to find potential panics.
  * Find Regex to format Terminal Outputs (clear-terminal).

*  **Tool:** Claude.
**Purpose:**
   * Generation of the following Integration Tests:
      * `fn test_create_open_close_vault() `
      * `fn test_vault_persistence() `
      * `fn test_edit_entry() `
      * `fn test_delete_entry() `
      * `fn test_duplicate_entry_names_rejected()`
      * `fn test_wrong_password_rejected() `
      * `fn test_tampering_detection() `
   * Long-Context Code-Review to find potential risks and recommend improvements.
   * Used for generating text in Testing section in documentation. 
   * Threat model and DFD review to find potential mistakes. 

* **Tool:** ChatGPT.
**Purpose:**
  * Time Planning?
  * Strategic Roadmap & Milestone Planning
  * Generation of the following unit tests (however, later adapted by us):
    * `fn test_create_new_vault() `
    * `fn test_start_session()`
    * `fn test_save_and_reopen()`
    * `fn test_end_session()`
  * Used for finding suitable crates.
  * Review existing code snippets and suggest refactoring or improvements.
  * Provide design suggestions for session integration, reviewed and adapted by the team
  * Generation of `fn session_state()` function in session.rs


All AI-generated outputs were reviewed, modified where necessary, and tested to ensure correctness and suitability for the project