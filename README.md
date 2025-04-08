# 🧠 ai_gettext_translator

**ai_gettext_translator** is a command-line tool that uses OpenAI’s models to automatically translate `gettext` messages in source code and `.po` files.

✨ It preserves `%{placeholders}`, supports plural forms, and logs with beautiful emoji & timestamps.

## 💡 What is it?

This tool automates the translation process of your `gettext` strings using OpenAI's LLMs. It supports:

- 🔠 **Inline translation**: Scan `.ex` files and translate `gettext("...")` strings (in case you've been writing them in different language).
- 🌍 **.po translation**: Automatically translate `.po` files with LLms.
- 🛡️ Placeholders like `%{name}` are preserved.
- 🧪 Dry-run and 🔁 force modes for full control.
- 📜 Beautiful, timestamped, logging of changes.

## ✨ Getting Started

### 1. Install Rust & Cargo

If you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install the tool

```bash
cargo install ai_gettext_translator
```

### 3. Set your OpenAI API key

You can either export it:

```bash
export OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxx
```

Or pass it directly to any command using `--api-key`.

### 4. Translate your `.po` files (example)

Assuming this structure:

```
locales/
├── es/
│   └── default.po
└── it/
    └── default.po
```

Run:

```bash
ai_gettext_translator translator ./locales --lang "es,it"
```

Want to re-translate already filled entries?

```bash
ai_gettext_translator translator ./locales --lang "es,it" --force
```

(Note: You can also use `--dry-run` to preview what would be translated, but without modifying any files.)

## 🧪 Commands

### 🔠 `inline`

Scans `.ex` files for `gettext("...")` strings and translates them inline to English. This is very useful if you've written your strings in different languages (or in another language).

```bash
ai_gettext_translator inline <folder> [OPTIONS]
```

#### Options:

| Flag        | Description                             |
| ----------- | --------------------------------------- |
| `--dry-run` | Preview changes without modifying files |
| `--api-key` | Use a specific OpenAI API key           |

### 🌍 `translator`

Translates `.po` files found in subfolders named by ISO language codes (e.g. `es/`, `it/`).

```bash
ai_gettext_translator translator <folder> --lang <langs> [OPTIONS]
```

#### Options:

| Flag        | Description                                                  |
| ----------- | ------------------------------------------------------------ |
| `--lang`    | Comma-separated list of target language codes (e.g. `es,it`) |
| `--dry-run` | Show what would be translated, but don’t modify files        |
| `--force`   | Re-translate entries that already have translations          |
| `--api-key` | Use a specific OpenAI API key                                |

## 🤝 Collaborate

This project is open source and contributions are welcome!

- 🐞 Found a bug? [Open an issue](https://github.com/jgcardelus/ai_gettext_translator/issues)
- 🌱 Want to contribute? Fork the repo and send a PR!
- 🗨️ Have ideas or feedback? Send them our way.

Made with ❤️ by [jgcardelus](https://github.com/jgcardelus).
