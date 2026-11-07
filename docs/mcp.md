# MCP connections

Preferences → MCP connects local MCP servers over stdio. Add a name, executable, and arguments as a JSON array, then save and choose Connect. Saving does not start a server; connections are not started automatically on launch.

For example, a Node server uses executable `node` and arguments `["C:/tools/my-server.mjs"]`. Use an absolute executable path if it is not on PATH. Install the server and its runtime separately, following that server's setup instructions. Prefer a runtime executable and server script rather than a shell launcher. Servers inherit Mycast's environment; per-server environment variables and credential fields are not currently available.

Connected servers list their tools. Your AI model must support tool calling. When it requests a tool, Mycast shows the server, tool name, and arguments in the conversation. **Allow once** runs that call; **Deny** returns a declined result to the model. Subsequent calls each require a separate decision. Tool definitions and returned content go to your selected AI provider, including cloud endpoints. Connect servers you trust: a local server starts a program with your user permissions; approval controls tool calls rather than sandboxing the server program.

Disconnect stops the connection and its child process. Quitting Mycast or disabling AI cancels connections; saved configuration remains. Stop cancels the AI request and pending approval. An already executed tool may have completed external work that cancellation cannot undo.

This version supports local stdio tools. Remote HTTP, OAuth, MCP resources, prompts, and sampling are not implemented. Limits include 12 servers, 64 advertised tools per request, 8 tool rounds, 32 KB tool arguments, 128 KB tool results, and a 60-second tool timeout. The overall AI request retains its 180-second timeout, including approval time. Models and servers with incompatible tool schemas may return provider errors.

Conversations are session-only. Follow-ups replay completed user questions and visible assistant answers, rather than raw tool exchanges or thinking previews. Context is capped at 16 recent exchanges / 256 KB. Older exchanges may remain visible after leaving that context window. New conversation or switching AI providers clears the transcript.
