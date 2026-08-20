# SWHID

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a Nutshell</p>
<p>Calculate or verify Software Hash Identifiers for local content with <code>acorn swhid &lt;PATH&gt;</code>.</p>
</blockquote>

Calculate or verify [Software Hash Identifiers](https://www.swhid.org/) for local files, directories, and Git objects.

```console
acorn swhid README.md --raw
acorn swhid ./src --kind directory
acorn swhid . --kind revision --reference HEAD
acorn swhid . --kind git-tree --reference HEAD
acorn swhid . --kind release --reference v1.0.0
acorn swhid . --kind snapshot
```

The default `auto` kind hashes a file as `cnt` or traverses a directory as `dir`. Git kinds use only objects already present in the repository: `git-blob`, `git-tree`, `revision`, `release`, and `snapshot`. Releases require annotated tags.

Use `--verify` to compare the calculated core object with an expected core or qualified SWHID:

```console
acorn swhid README.md --verify swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a
```

Calculation is read-only. It does not clone, fetch, materialize Git LFS, recurse into submodules, or hydrate partial clones. A filesystem directory represents current working-tree content; a Git tree represents committed state.
