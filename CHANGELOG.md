# Change log

## Unreleased

- Redraw the running Sync bar after output for another bar clears its
  line.
- Don't print a blank line when a hidden Sync bar finishes.
- Only end the line after a Sync redraw when the bar is no longer
  current.
- Stop Sync bars from redrawing over an active prompt.
- Don't clear current bar when a stale Sync bar finishes.

### Breaking changes

- Sync drops bar refs when they succeeds or fails, so using reference
  afterwards now panics.

## 2.0.0

### Breaking changes

- `prompt` returns `Result<String, retrogress::Error>` instead of
  `String`.
- Handle Ctrl+C that may reach prompts.

## 1.5.0

- Handle Ctrl+U and Alt+Backspace in inputs.
- Use edition 2024.

## 1.4.0

- Add simple back buffer for reducing size of parallel renders.
- Implement stdlib traits on structs, enums for general usability:
  - Display on ProgressMessage.
  - Debug, From dyn Progress on ProgressBar.
  - Clone, Copy, Debug, Display on parallel Status.
  - Debug on Sync, Renderer, Parallel.

## 1.3.0

- Truncate progress output to terminal width.

## 1.2.0

- Proactively release locks prior to IO.

## 1.1.0

- Add required `print_inline` to `Progress` trait.

## 1.0.0

- Add required `prompt`, `clear_prompt`, and `set_prompt_input` to
  `Progress` trait.
- Introduce `retrogress::Parallel` progress bar.
- Remove dependency on indicatif.

## 0.2.1

- Always enable ansi colors.
- Disable tick when hidden.

## 0.2.0

- add `hide` and `show` to `Progress`.

## 0.1.0

- `retrogress::Sync` for cases when a single progres bar is running at
  one time.
- `retrogress::ProgressBar` to wrap `Box<dyn Progress>`.
