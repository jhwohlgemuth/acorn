# 🧩 Skill

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Extract ACORN's bundled agent instructions with <code>acorn skill</code>.</p>
</blockquote>

```sh
acorn skill
```

The command writes the bundled `SKILL.md` into ACORN's platform cache directory, prints its path, and attempts to copy agent instructions to the clipboard. A clipboard warning does not remove the extracted file; use the printed path when clipboard access is unavailable, such as in a headless session.

## What the skill enables

The extracted skill gives compatible AI agents a current, task-oriented guide to ACORN's schemas, commands, dry-run conventions, artifact checks, and offline behavior. It directs agents to verify syntax against the installed CLI or generated command help instead of inventing fields and flags.

`acorn skill` does not launch an agent, invoke a model, or grant access to files and services. It produces instructions that a user can load into a compatible agent under that agent's existing permissions.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Explore library integration on the <a href="../../learn/developer.md">Developer trail</a>.</p>
