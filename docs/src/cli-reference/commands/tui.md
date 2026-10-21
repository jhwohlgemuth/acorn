# 🖥️ TUI

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Open ACORN's interactive terminal workspace with <code>acorn tui</code>.</p>
</blockquote>

```sh
acorn tui

# Equivalent global shortcut
acorn --interactive
```

The TUI uses the configured local database and respects global database and offline options:

```sh
acorn --database-path ./acorn.db --offline tui
```

Type `/` on the dashboard to see available commands. `/check`, `/doctor`, `/gather` (also `/harvest`), and `/theme` open their screens. `/exit` leaves the TUI; `/quit` remains an alias. The dashboard filters this list as you type.

Use command mode instead when scripting or when the installed build does not include TUI support.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Browse every automation path in the <a href="./">command reference</a>.</p>
