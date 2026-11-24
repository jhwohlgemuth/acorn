# Browser assets

## Authored browser scripts

`scripts/*.js` holds the 16 browser evaluation bodies above the frozen 240-byte inline threshold. They are compiled into `acorn-web` through `js_file!`, which validates each file and emits `include_str!` so edits trigger a Rust rebuild. The macro passes exactly one JSON value as `__acornArgs`; the files do not interpolate Rust text. These are source files, not separately fetched browser assets. Frontend lint discovery should include `crates/acorn-web/assets/scripts/*.js`.

The files cover map and window-sync channels; workspace deletion, appearance reset, remote bucket and file loading, ROR index and skill loading; diagnostic reveal, choice scrolling, result focus, page navigation, editor sync, workspace restore, and review-fragment synchronization. Each file carries an MIT SPDX notice. Named functions and result types remain in `src/script/mod.rs`.

## PID label colors

Checked November 13, 2026. Labels use the following brand accents, with lighter or darker text and tinted backgrounds for readable contrast in both themes. The small dot retains the source color. Text always identifies the scheme; color does not convey validation status. No logos are bundled.

| Scheme | Accent | Source |
| --- | --- | --- |
| ARK | `#6474ff` | Blue sampled from the [ARK Alliance logo](https://arks.org/assets/images/share/arka_wg_logo.png). |
| arXiv | `#a5d6fe` | Open Blue, a [current primary color](https://info.arxiv.org/brand/colors.html); Campus Red is now a heritage color. |
| DOI | `#fab70c` | Gold used by the [DOI Foundation](https://www.doi.org/). |
| Handle | `#af2d29` | Primary red in the [Handle.Net stylesheet](https://www.handle.net/style/hnr-style.css). |
| ISBN | `#12344f` | Primary blue in the [International ISBN Agency stylesheet](https://www.isbn-international.org/themes/isbn8/css/style.css). |
| ISNI | `#333399` | Purple in the [ISNI logo copy](https://commons.wikimedia.org/wiki/File:International_Standard_Name_Identifier.svg), attributed to ISNI; its official site blocked automated access. |
| ORCID | `#a6ce39` | ORCID green, [brand guidelines](https://info.orcid.org/brand-guidelines/). |
| RAiD | `#368bcc` | Brand blue in the [RAiD website](https://raid.org/) stylesheet. |
| ROR | `#53baa1` | [ROR display guidelines](https://ror.readme.io/docs/display). |
| RRID | `#6599ce` | Blue sampled from the RRID logo on [RRIDs.org](https://www.rrids.org/). |
| SWHID | `#e20026` | Red from the [Software Heritage communication kit](https://www.softwareheritage.org/software-heritage-communication-kit/). |

Patent identifiers span multiple issuing authorities and have no single brand color. They and unrecognized schemes use the neutral fallback `#737b88`.
