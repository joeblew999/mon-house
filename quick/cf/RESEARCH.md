# `cf/` — research notes from `quick/.src/cloudflare-agents/examples/`

Survey of all 37 examples in the upstream `cloudflare-agents` repo, ranked
by relevance to **this** project (spec copilot + future spec editor +
builder quote workflow). Updated after each material API shift in the SDK
or a new pass through `examples/`.

Each entry: what it does, what code we'd lift, and rough estimate of how
much work it is to graft onto our `cf/` Worker.

---

## Already in use

- **`ai-chat`** — base `AIChatAgent` + `useAgentChat` hook.
  - Status: shipped (`cf/src/chat.ts` + `cf/src/client.tsx`).
  - Note: their example imports their own `@cloudflare/kumo` design system
    (Button / InputArea / Surface / Empty). We intentionally use Tailwind
    primitives instead so we control the shell.

- **`playground` Layout/Sidebar** — side-pane shell with Tailwind
  responsive flex (`flex flex-col md:flex-row`).
  - Status: shipped (the side-pane chat layout in `cf/src/client.tsx`).
  - The mobile fold-out hamburger pattern is theirs; we do the simpler
    "stack on mobile, side-by-side on md+" version because we don't need
    a route-driven sidebar yet.

---

## Strong candidates — high value, modest effort

### `auth-agent` — GitHub OAuth gate (≈ 2-3 hrs)

**What:** `/auth/login` → GitHub OAuth → cookie session → `/auth/me` →
`getAgentByName(env.ChatAgent, user.login)` so each authenticated user
gets their own DO instance (= their own chat history, isolated).

**Why for us:** the open-internet chat is currently rate-limited but not
auth-gated. CF Access is one option (free for first 50 users); this
example is a self-contained alternative that lives in the Worker code.
Either kills the "stranger burns the AI budget" risk that COSTS.md flags
as the only outstanding worry.

**Lift:** copy `examples/auth-agent/src/auth.ts` (~150 lines, pure utility),
add `/auth/*` routes to our `index.ts`, gate `/agents/*` and `/translate`
with `getGitHubUserFromRequest()`. Two new wrangler secrets:
`GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`.

**Trade-off:** every visitor would need to log in with GitHub. Joe + Mon
already have GitHub accounts, no friction. Random visitors on the published
URL get bounced — exactly the goal.

### `workspace-chat` — chat agent with a persistent virtual filesystem (≈ 1 day)

**What:** `WorkspaceChatAgent extends AIChatAgent` plus 13 tools the LLM
can call: `readFile`, `writeFile`, `listDirectory`, `glob`, `mkdir`,
`deleteFile`, `gitInit`, `gitStatus`, `gitAdd`, `gitCommit`, `gitLog`,
`gitDiff`, `runStateCode`. Backed by `Workspace` from `@cloudflare/shell`
(SQLite + R2 hybrid storage inside the DO).

**Why for us:** **this is the killer feature.** With this, the spec copilot
isn't a context-blind chat — it can:
- Read `BATHROOM-COMPACT.md` ("what's still TBC in this spec?").
- Edit a section in place ("rewrite section 3 more concisely").
- Diff against a base ("what changed in the parts list this week?").
- Even commit ("save these revisions and translate the changed sections").

It's exactly the "AI editor with grounded context" UX that ADR 008's SPA
editor wants.

**Lift:** the agent class + tool definitions copy across cleanly. The
storage layer needs thought — do we point Workspace at our existing
`specs/*.md` (hard, our files live in git on disk) or use it as a
scratchpad ("paste a section, ask for revisions, copy back")? Probably
scratchpad first, full FS sync later.

**Trade-off:** big enough to deserve its own ADR. But the value is huge.

### `structured-input` — interactive forms inside chat (≈ 4 hrs)

**What:** the LLM has tools like `askMultipleChoice`, `askYesNo`,
`askFreeText`, `askRating`. The chat UI renders them as inline form
widgets. The user fills them in; the result becomes the tool's output;
the LLM continues.

**Why for us:** **directly unblocks ADR 006 (Builder Quote System).** A
builder pastes a spec and the copilot walks them through quote-line items
with one form per item: "labour cost for the bathroom retile", "tile cost
per sqm", etc. Structured output = clean JSON the quote system can store.

**Lift:** copy the four basic tools verbatim. Add quote-domain ones later
(`askMaterialChoice`, `askPricePerUnit` with currency ＋ qty).

### `multi-ai-chat` — sub-agent routing (≈ 4-6 hrs)

**What:** an `Inbox` top-level Agent owns a list of chat sessions; each
chat is a `Chat` sub-agent (`subAgent(Chat, id)`) with its own DO storage.
Client connects to `/agents/inbox/{user}` for the sidebar, then
`/agents/inbox/{user}/sub/chat/{chatId}` for one conversation.

**Why for us:** "one chat per spec" or "one chat per quote." Today the
copilot is a single shared session for everyone. With sub-agent routing
we'd give each spec (BATHROOM, KITCHEN, etc.) its own chat history.
Switching specs = switching chats. Mon's chat history stays separate
from Joe's.

**Lift:** restructure `ChatAgent` as a sub-agent of an `Inbox` parent.
Add `createChat` / `deleteChat` callable methods. Frontend: chat picker
in the sidebar.

**Pairs naturally with `auth-agent`** — auth gives you the user id, then
each user has their own Inbox.

### `agents-as-tools` — give the chat agent tools that call other agents (≈ 4 hrs)

**What:** an agent can register other agents as tools. The LLM decides
when to delegate. Useful for splitting concerns: a "translator agent",
a "quote-writer agent", etc.

**Why for us:** the chat copilot could expose a `translateSection` tool
that hits our existing `/translate` endpoint, or a `compileToPDF` tool
once typst-WASM lands. Lets the conversation flow include real actions,
not just text.

**Lift:** define each helper as an agent or as a plain `tool()`, register
it in `streamText({ tools })`. Same pattern as `workspace-chat`'s file
tools.

---

## Lower priority — interesting but not pulling weight today

### `resumable-stream-chat`
Auto-resume on disconnect. **Already free** with `AIChatAgent`. No lift.

### `mcp` / `mcp-client` / `mcp-worker-authenticated`
MCP server / client. Useful if we want to expose Quick as an MCP tool
to other LLMs (Claude Desktop, etc.) so external agents can call our
spec workflow. Future ADR territory.

### `voice-agent` / `voice-input`
Voice-to-text + full voice agent. Pretty cool — "tell the copilot to
revise this section" without typing. Skip for now; audio in a builder
context isn't a clear win.

### `email-agent`
Send emails from the Worker. Could be: "email Mon when a section is
flagged for human review." Mild value, low effort if we want it.

### `push-notifications`
Browser push when CI finishes building a PDF. Cute. Low priority.

### `tictactoe` / `playground` (demos area)
Just demos. Not for our use case.

### `codemode` / `codemode-mcp` / `dynamic-workers`
LLM writes and executes Worker code at runtime. Powerful but enormous
blast radius. Only consider if the SPA editor genuinely needs it.

### `github-webhook`
React to GitHub events. Could trigger "rebuild PDFs when main moves."
Already covered by GitHub Actions today.

### `x402` / `x402-mcp`
Paid HTTP / paid MCP tools. Not relevant unless we monetise something.

---

## Recommended order of adoption

If we keep moving on the SPA editor track:

1. **`auth-agent`** first — closes the "open AI to the internet" hole.
   Foundation for everything else (`multi-ai-chat` needs user id).
2. **`workspace-chat` agent + tools (subset)** — give the copilot
   `readFile` / `writeFile` over a virtual filesystem so it can actually
   manipulate the active spec. Value compounds with everything below.
3. **`structured-input`** — unlocks the ADR 006 builder quote workflow.
4. **`multi-ai-chat` sub-agent routing** — once each user has multiple
   specs to chat about.
5. **`agents-as-tools`** — when the copilot needs to invoke
   `translate` / `compile` as actions, not just talk about them.

Defer everything else until a concrete user need shows up.

---

## How to keep this current

- Every time `cf/package.json` bumps `agents` or `@cloudflare/ai-chat`,
  re-survey `examples/` (1-2 new examples per minor bump on average).
- When we copy a pattern from an example, link the source path in the
  commit message body so a future archaeologist knows the lineage.
- When an example we cited gets renamed or deleted, update this file.
