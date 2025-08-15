# Change log

## Unreleased

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
