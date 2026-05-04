# ADR 005: Headless AI Translation Architecture

**Date**: 2025-10-31
**Status**: **Superseded** (2026-05-04) — describes the original Go pathfinder pipeline's Anthropic-API-driven headless translation. The Go code (`main.go`, `cmd/`, `pkg/translate/`, `internal/`) has since been removed. The replacement runs entirely on Cloudflare Workers AI (SEA-LION 27B, Thai-fine-tuned) via a public `POST /translate` endpoint on `quick-worker.gedw99.workers.dev`, with the Rust CLI (`quick/cli/`) as the local client. No external API keys involved. See `quick/CLAUDE.md` for current architecture. This ADR is preserved as historical context.
**Context**: Translation system needs to run autonomously without human intervention

---

## Problem

The current translation workflow requires manual human translation:
1. Run `translate sync` → generates task files
2. **MANUAL**: Human fills in `target_text` fields
3. Run `translate apply` → injects translations

For production use, we need **headless operation** where Claude AI automatically translates without human interaction.

---

## Decision

Implement a **headless AI translation** system using Anthropic's Claude API that can run fully automated.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│ Headless Translation Pipeline                       │
└─────────────────────────────────────────────────────┘

User runs ONE command:
  $ mon-tool translate full --language=th

Internally executes three phases:

┌──────────────────────┐
│ PHASE 1: Extract     │
│ (translate sync)     │
│                      │
│ • Scan EN files      │
│ • Generate tasks/    │
│   translate-th.json  │
└──────────────────────┘
         ↓
┌──────────────────────┐
│ PHASE 2: Translate   │
│ (translate auto)     │
│                      │
│ • Load task file     │
│ • Call Claude API    │
│ • Fill target_text   │
│ • Save updated task  │
└──────────────────────┘
         ↓
┌──────────────────────┐
│ PHASE 3: Apply       │
│ (translate apply)    │
│                      │
│ • Load filled task   │
│ • Inject into files  │
│ • Delete task        │
└──────────────────────┘
         ↓
┌──────────────────────┐
│ Events Logged        │
│                      │
│ • All operations     │
│ • AI usage/cost      │
│ • Translation stats  │
└──────────────────────┘
```

---

## Implementation

### New Packages

**pkg/translate/ai/**
- `types.go` - AI translation interfaces and types
- `claude.go` - Claude API implementation
- Future: `openai.go`, `google.go`, etc.

### Key Components

#### 1. Translator Interface
```go
type Translator interface {
    Translate(req *TranslationRequest) (*TranslationResponse, error)
    Name() string
}
```

#### 2. Translation Request
```go
type TranslationRequest struct {
    SourceLanguage string
    TargetLanguage string
    Domain         string // "architecture", "medical", etc.
    Terminology    map[string]string // "envelope" → "แนวเปลือกอาคาร"
    Notes          []string
    Items          []TranslationItem
}
```

#### 3. Translation Item
```go
type TranslationItem struct {
    ID          string // For tracking
    Context     string // "heading", "paragraph", "label"
    SourceText  string
    TargetText  string // Filled by AI
}
```

#### 4. Translation Response
```go
type TranslationResponse struct {
    Success        bool
    ItemsProcessed int
    Translations   []TranslationItem
    Usage          Usage // Tokens, cost tracking
}
```

---

## API Integration

### Claude API Call

```go
translator := ai.NewClaudeTranslator(apiKey, "claude-3-5-sonnet-20241022")

task, response, err := translate.AutoTranslate(rootDir, taskFile, translator)
if err != nil {
    // Handle error
}

fmt.Printf("Translated %d items\n", response.ItemsProcessed)
fmt.Printf("Cost: $%.4f\n", response.Usage.EstimatedCost)
```

### Prompt Structure

The AI receives:
```
You are a professional translator specializing in architectural drawings.

Translate from English to Thai.

TERMINOLOGY (use these exact translations):
- envelope → แนวเปลือกอาคาร
- wall-exterior → ผนังภายนอก
- roof → หลังคา

TRANSLATION NOTES:
- Use formal/technical Thai for construction documents
- Translate semantic meaning, not word-for-word

TEXTS TO TRANSLATE:
Return JSON array:
[
  {"id": "0", "target_text": "TRANSLATION"},
  {"id": "1", "target_text": "TRANSLATION"},
  ...
]

Items:
ID: 0
Context: heading
Source: # Test Drawings

ID: 1
Context: paragraph
Source: This is a test documentation folder...
```

---

## Usage

### Individual Commands

```bash
# Extract text (manual workflow)
$ mon-tool translate sync

# Auto-translate with AI (headless)
$ export ANTHROPIC_API_KEY=sk-ant-...
$ mon-tool translate auto tasks/translate-th.json

# Apply translations
$ mon-tool translate apply tasks/translate-th.json
```

### Fully Automated Pipeline

```bash
# One command does everything
$ mon-tool translate full --language=th

Output:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🤖 Automated Translation Pipeline
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Phase 1: Extracting text...
✓ Generated tasks/translate-th.json (24 extractions)

Phase 2: Calling Claude AI...
✓ Translated 24 items
📊 Usage: 1,245 input tokens, 890 output tokens
💰 Cost: $0.0171

Phase 3: Applying translations...
✓ Applied 24 translations to 2 files
🗑️ Deleted task file

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Translation complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Event Sourcing Integration

All AI operations emit events:

```json
{
  "type": "AITranslationRequested",
  "timestamp": "2025-10-31T09:30:15Z",
  "session_id": "a1b2c3d4",
  "task_file": "tasks/translate-th.json",
  "items_count": 24,
  "model": "claude-3-5-sonnet-20241022"
}

{
  "type": "AITranslationCompleted",
  "timestamp": "2025-10-31T09:30:22Z",
  "session_id": "a1b2c3d4",
  "items_translated": 24,
  "input_tokens": 1245,
  "output_tokens": 890,
  "cost_usd": 0.0171,
  "duration_seconds": 7
}
```

View history:
```bash
$ mon-tool translate events
[09:30:15] 🤖 AI Translation requested: 24 items (claude-3-5-sonnet)
[09:30:22] ✅ AI Translation completed: 24 items ($0.0171, 7s)
```

---

## Configuration

### API Key

Set via environment variable:
```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

Or pass directly:
```bash
$ mon-tool translate auto tasks/translate-th.json --api-key=sk-ant-...
```

### Model Selection

Default: `claude-3-5-sonnet-20241022` (best quality/cost balance)

Override:
```bash
$ mon-tool translate auto tasks/translate-th.json --model=claude-3-opus-20240229
```

---

## Cost Management

### Estimation

Before running:
```bash
$ mon-tool translate estimate tasks/translate-th.json
Estimated cost: $0.015 - $0.025
Based on: 24 items, ~1200 input tokens
```

### Tracking

After running:
```bash
$ mon-tool translate stats
Total translations: 147
Total cost: $0.89
Average cost per item: $0.006
```

---

## Error Handling

### Retry Logic

If API call fails:
1. Log error event
2. Keep task file with partial translations
3. Retry with exponential backoff (3 attempts)
4. If all retries fail, return error with partial results

### Validation

After AI translation:
1. Verify all IDs matched
2. Check no translations are empty
3. Validate character encodings
4. Log any issues as events

---

## Benefits

1. **Fully Automated** - No human interaction needed
2. **Traceable** - Every operation logged as event
3. **Cost Transparent** - Know exactly what each translation costs
4. **Extensible** - Easy to add new AI providers (OpenAI, Google, etc.)
5. **CQRS Compatible** - Fits existing command/query architecture
6. **Production Ready** - Error handling, retries, validation

---

## Alternatives Considered

### Option A: Manual Translation
**Rejected** - Not scalable, too slow for production

### Option B: Machine Translation (Google Translate)
**Rejected** - Lower quality, no architectural terminology support

### Option C: Local AI Model
**Considered** - Would avoid API costs but requires GPU infrastructure

---

## Future Enhancements

1. **Batch Processing** - Process multiple languages in parallel
2. **Caching** - Cache common translations to reduce costs
3. **Quality Scoring** - AI rates its own translation confidence
4. **Human Review** - Flag low-confidence translations for human check
5. **Custom Fine-tuning** - Train on architectural terminology
6. **Cost Budgets** - Set max spend per translation run

---

## Migration Path

1. ✅ **Phase 1**: Manual translation (current)
2. ✅ **Phase 2**: CQRS + Event Sourcing (current)
3. 🚧 **Phase 3**: Headless AI (this ADR)
4. 🔮 **Phase 4**: Advanced features (caching, quality scoring)

---

## Success Criteria

- [ ] `translate auto` command works with real API key
- [ ] Cost tracking accurate within 5%
- [ ] Translation quality matches human baseline (95%+ accuracy)
- [ ] Full pipeline (`translate full`) completes in <30 seconds
- [ ] All events logged correctly
- [ ] Error handling tested (API failures, network issues)

---

**This enables the system to run COMPLETELY HEADLESS - perfect for CI/CD pipelines, automated builds, or scheduled translations!**
