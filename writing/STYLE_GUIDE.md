# VeriLecture writing style

This is the project-level instruction for every human-facing sentence. It applies to the website, app UI, installer, README, documentation, release notes, pull requests, commits, issue templates, comments, and docstrings.

## Completion rule

Before finishing a task, review every newly added or changed human-facing text with a Humanizer-style pass and run the applicable Vale check. Check only the changed prose by default; do not rescan the whole repository unless the task requires it.

```text
Code:    Lint → Test → Build
Writing: Humanizer → Vale
```

The workflow is intentionally light. Humanizer is a writing approach, not a new runtime dependency. Vale is the only formal prose-lint dependency. Other anti-AI-writing projects may inform edits, but are not required project dependencies.

## House style

- Explain the product; do not praise it.
- State concrete facts before benefits. Prefer a noun and a verb over adjectives.
- Keep one idea per section. Use the fewest words that preserve the meaning.
- Write for scanning. Read important copy aloud once before accepting it.
- Do not invent claims, numbers, guarantees, or user feelings.
- Avoid filler, marketing jargon, repeated summaries, and symmetrical three-part lists.
- Treat phrases such as “赋能”, “生态”, “闭环”, “打造”, “构建”, “重新定义”, “开启”, “探索”, “全新”, “无缝”, and “智能驱动” as review prompts, not automatic bans. Keep one only when it is precise and necessary.

## Contexts

- **Website:** clear in five seconds; calm, restrained, and specific. Buttons name an action: “Download for Windows”, “Read the privacy notes”.
- **UI:** short and explicit. Do not repeat information already shown by the interface.
- **Errors:** say what happened and what the user can do next.
- **README/docs:** technical, direct, and easy to scan. Separate current facts from pending validation.
- **Release notes:** describe the change a user actually gets.
- **Commits/PRs:** describe the actual change; do not write a product pitch.
