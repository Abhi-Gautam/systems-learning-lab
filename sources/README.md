# sources/

Reference indexes for `/design-today`. Two files:

- `papers.md` — canonical academic papers. The substance behind Deep-dive mechanism walks.
- `blogs.md` — engineering blogs. The production numbers and war stories behind "Where this shows up in production" sections.

## How sources are used by /design-today

Each problem in `problems/{hld,lld}.json` has a `sources` array pinning the canonical references for that problem. When the skill composes a note, it:

1. Reads the problem's `sources` array (papers + blogs + book chapters).
2. In Deep-dive subsection 2 ("Walk it concretely") — cites the paper section that introduces the mechanism.
3. In Deep-dive subsection 6 ("Where this shows up in production") — cites blogs with named systems + named numbers.
4. In the "Common follow-ups" section — pulls failure modes from blog post-mortems.

## Why we don't copy paper PDFs into the repo

- Most canonical papers are in the LLM's training data; citation by name + section is enough.
- Blogs change URLs over time; we link the URL but the composer cites the *insight*, not the URL.
- Keeping the repo lightweight matters for clone time + Obsidian sync.

If a paper is essential and you want it locally, drop the PDF into `references/papers/` (untracked by git) and add a note in `papers.md` saying "local copy at references/papers/X.pdf".

## Adding a source

- For papers: edit `papers.md`. Group under the right category. Include `**Powers**:` (which problems cite it), `**Key sections**:`, `**One-line**:`.
- For blogs: edit `blogs.md`. Same shape. Add `**Key numbers**:` if the blog publishes specific scale numbers worth quoting.
