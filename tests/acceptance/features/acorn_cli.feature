@acorn @cli
Feature: Human acceptance of the ACORN command line
  ACORN users need understandable and repeatable evidence that supported workflows
  behave correctly without risking their normal files, databases, or accounts.

  This file specifies behavior. Matching IDs and copy-paste PowerShell commands are
  in docs/src/cli-reference/acceptance-test-plan.md. These scenarios become directly
  executable as Gherkin only after a Cucumber runner and step definitions are added.
  The intake handbook also supplies a Python rehearsal covering INTAKE-01 through
  INTAKE-09; visual, interactive, and live-service checks remain explicit.

  Background:
    Given the tester is in the ACORN repository root
    And the exact checked-out ACORN source has been built
    And the disposable acceptance workspace has been initialized
    And ACORN uses only fixtures and outputs in that workspace

  Rule: Installation and command discovery are understandable

    @baseline @CLI-01
    Scenario: Report the installed version
      When the tester runs ACORN with "--version"
      Then the command succeeds
      And output matches "acorn-cli <semantic-version>"

    @baseline @CLI-02
    Scenario: Advertise every supported command family
      When the tester runs ACORN with "--help"
      Then the command succeeds
      And help lists the following command families:
        | command  |
        | check    |
        | create   |
        | doctor   |
        | download |
        | export   |
        | format   |
        | gather   |
        | import   |
        | intake   |
        | link     |
        | report   |
        | swhid    |
        | skill    |
        | serve    |
        | sync     |
        | tui      |

    @baseline @CLI-03
    Scenario Outline: Explain a supported command
      When the tester requests help for "<command>"
      Then the command succeeds
      And usage information is displayed

      Examples:
        | command        |
        | check          |
        | create runner  |
        | create bot     |
        | doctor         |
        | download       |
        | download model |
        | export         |
        | format         |
        | gather         |
        | import         |
        | import model   |
        | import spec    |
        | intake stage   |
        | intake inspect |
        | intake apply   |
        | intake run     |
        | link           |
        | report         |
        | swhid          |
        | skill          |
        | serve bot      |
        | sync           |
        | tui            |

    @baseline @CLI-04
    Scenario Outline: Accept a documented alias
      When the tester runs the safe acceptance command using "<alias>"
      Then the command succeeds

      Examples:
        | alias   |
        | lint    |
        | fmt     |
        | harvest |

  Rule: Check identifies structural and content problems

    @baseline @CHECK-01
    Scenario: Check valid RAD without network services
      Given a valid RAD JSON fixture
      When the tester checks it offline with prose and website checks disabled
      Then the command succeeds
      And the schema error count is zero

    @baseline @CHECK-02
    Scenario: Report multiple malformed RAD fields
      Given RAD containing several malformed fields
      When the tester checks it with no-fail behavior
      Then checking does not abort at the first field
      And multiple deserialization or schema findings are reported

    @baseline @CHECK-03
    Scenario Outline: Check a supported encoding or standard
      Given the "<fixture>" fixture
      When the tester checks it as "<standard>" without network services
      Then the input is parsed
      And a check report is produced

      Examples:
        | fixture                   | standard |
        | valid-rad/index.jsonc      | rads     |
        | rad.yaml                   | rads     |
        | rad.md                     | rads     |
        | CITATION.cff               | cff      |
        | raid.json                  | raid     |
        | schema/datacite.json       | datacite |
        | schema/dcat.json           | dcat     |
        | schema/invenio-single.json | invenio  |
        | schema/huwise.json         | huwise   |

    @baseline @CHECK-03B
    Scenario Outline: Check a Dublin Core RDF syntax
      Given equivalent Dublin Core metadata encoded as "<format>"
      When the tester checks it as "dcmi" without network services
      Then the input is parsed
      And a check report is produced

      Examples:
        | format  |
        | JSON-LD |
        | RDF/XML |
        | Turtle  |

    @baseline @CHECK-04
    Scenario: Check plain text from standard input
      Given plain text is piped to ACORN
      When no filesystem source is selected
      Then the command succeeds
      And the report identifies "stdin.txt"

    @baseline @CHECK-05
    Scenario: Check extracted PDF text
      Given a text-bearing PDF fixture
      When the tester checks it using the document standard
      Then document checks are produced without a crash

    @baseline @CHECK-06
    Scenario Outline: Select a readability metric
      When the tester checks valid RAD tersely using "<metric>"
      Then the command succeeds

      Examples:
        | metric |
        | fkgl   |
        | ari    |
        | cli    |

    @baseline @negative @CHECK-07
    Scenario: Reject remote check input while offline
      When the tester checks an HTTPS URL in offline mode
      Then the command fails
      And the error explains that remote input is unavailable offline

    @baseline @clipboard @CHECK-08
    Scenario: Check clipboard text without losing prior data
      Given the current clipboard value has been saved
      When the tester checks known clipboard text
      Then the command succeeds
      And the previous clipboard value is restored

  Rule: Format and link offer safe previews before writing

    @baseline @FORMAT-01
    Scenario: Preview formatting without changing the source
      Given RAD containing noncanonical values
      When the tester runs format with dry-run
      Then a normalization diff is displayed
      And the source checksum is unchanged

    @baseline @FORMAT-02
    Scenario: Apply deterministic normalization
      Given disposable RAD containing noncanonical values
      When the tester applies format
      Then "rs" becomes "rust"
      And a Department of Energy alias becomes its canonical name

    @baseline @negative @FORMAT-03
    Scenario: Explain unsupported offline formatting
      When the tester requests an offline format dry run
      Then the command fails
      And the error says offline format is not implemented

    @baseline @FORMAT-04
    Scenario: Format and compare equivalent Dublin Core syntaxes
      Given equivalent Dublin Core metadata encoded as Turtle and JSON-LD
      When the tester previews Turtle formatting and compares both files semantically
      Then formatting leaves the source unchanged
      And the semantic diff is empty

    @baseline @LINK-01
    Scenario: Preview linked data without changing the source
      Given disposable RAD
      When the tester runs link with dry-run
      Then a linked-data diff is displayed
      And the source checksum is unchanged

    @baseline @LINK-02
    Scenario: Create JSON-LD
      Given disposable RAD
      When the tester applies link
      Then "index.jsonld" is created
      And it contains Schema.org context

  Rule: Gather discovers and optionally persists identifiers

    @baseline @GATHER-01
    Scenario: Normalize identifiers found in text
      Given text containing a DOI, ORCID, ROR, and arXiv identifier
      When the tester gathers raw identifiers offline
      Then every identifier is present in normalized form

    @baseline @GATHER-02
    Scenario Outline: Render a structured discovery report
      When the tester gathers known identifiers as "<format>"
      Then a non-empty "<format>" report is created

      Examples:
        | format   |
        | json     |
        | yaml     |
        | markdown |

    @baseline @GATHER-03
    Scenario: Extract identifiers from PDF
      When the tester gathers identifiers from the PDF fixture
      Then DOI "10.1145/73560.73576" is present

    @baseline @GATHER-04
    Scenario: Limit recursive directory gathering
      Given identifiers at three directory depths
      When the tester gathers with maximum depth 1
      Then a direct-child identifier is present
      And a grandchild identifier is absent

    @baseline @database @GATHER-05
    Scenario: Persist discoveries in isolated SQLite
      Given an unused workspace database path
      When the tester gathers a DOI using SQLite
      Then the command succeeds
      And the database file is created

  Rule: Export creates usable publication artifacts

    @baseline @EXPORT-01 @EXPORT-02
    Scenario Outline: Export RAD to a requested directory
      Given valid RAD in "<source>" format
      When the tester exports it as "<target>" to a chosen directory
      Then the command succeeds
      And the artifact exists in the chosen directory

      Examples:
        | source | target   |
        | json   | yaml     |
        | json   | markdown |

    @baseline @EXPORT-03
    Scenario: Create and validate CITATION.cff
      Given valid RAD JSON
      When the tester exports it as CFF
      Then "CITATION.cff" is created beside the source
      And ACORN can check it

    @baseline @EXPORT-04
    Scenario Outline: Export a defined JSON Schema
      When the tester exports the "<standard>" schema
      Then a non-empty valid JSON Schema is created

      Examples:
        | standard |
        | acre     |
        | rads     |
        | cff      |
        | datacite |
        | dcat     |
        | dcmi     |
        | invenio  |
        | huwise   |
        | raid     |

    @baseline @EXPORT-05 @EXPORT-05B
    Scenario Outline: Crosswalk supported metadata standards
      Given valid "<from>" metadata
      When the tester exports it as "<to>" JSON
      Then a non-empty converted artifact is created

      Examples:
        | from     | to       |
        | datacite | dcat     |
        | datacite | invenio  |
        | datacite | huwise   |
        | dcat     | datacite |
        | dcat     | invenio  |
        | dcat     | huwise   |
        | invenio  | datacite |
        | invenio  | dcat     |
        | invenio  | huwise   |
        | huwise   | datacite |
        | huwise   | dcat     |
        | huwise   | invenio  |

    @baseline @EXPORT-05A
    Scenario: Crosswalk DataCite to Dublin Core Turtle
      Given valid DataCite metadata
      When the tester exports it as Dublin Core Turtle
      Then a non-empty Turtle artifact is created
      And ACORN can check it as Dublin Core

    @baseline @EXPORT-05C
    Scenario: Import Dublin Core with explicit RAD identity context
      Given valid Dublin Core metadata and a caller-supplied contact and local identifier
      When the tester exports it as RAD JSON
      Then a non-empty RAD artifact is created
      And the caller-supplied local identifier is retained

    @optional @browser @visual @EXPORT-06
    Scenario: Create a visually usable PDF
      Given Chrome or Chromium is installed
      When the tester exports highlight RAD as PDF
      Then a non-empty PDF is created
      And text and images are readable without clipping or distortion

    @baseline @visual @EXPORT-07
    Scenario: Create PowerPoint from a reference template
      Given a valid template and highlight RAD
      When the tester exports with ASPECT labels and scores
      Then a non-empty PowerPoint is created
      And PowerPoint opens it without requesting repair

    @baseline @EXPORT-08
    Scenario Outline: Create a verified BagIt archive
      When the tester exports research content as BagIt "<format>"
      Then built-in verification succeeds
      And the archive exists

      Examples:
        | format |
        | zip    |
        | 7z     |
        | tar    |
        | tar.gz |

    @baseline @EXPORT-09
    Scenario: Preview export without writing
      When the tester runs export with dry-run
      Then the command succeeds
      And no artifact is created

    @baseline @EXPORT-10
    Scenario: Combine multiple RAD records
      Given two valid RAD records
      When the tester exports combined JSON
      Then one JSON artifact contains at least two records

  Rule: Import transfers content safely and describes APIs

    @baseline @IMPORT-01
    Scenario: Import local research content
      When the tester imports a local bucket offline
      Then selected content appears beneath the output directory

    @baseline @IMPORT-02
    Scenario: Filter and flatten imported content
      When the tester imports only "index.json" with flatten enabled
      Then "index.json" is a direct child of the output directory
      And unrelated images are absent

    @baseline @negative @IMPORT-03
    Scenario: Protect existing imported content
      Given a destination contains an earlier import
      When the tester imports again without clobber
      Then the command fails without replacement
      When the tester repeats with clobber
      Then the command succeeds

    @baseline @IMPORT-04
    Scenario: Generate an endpoint resource from OpenAPI
      Given OpenAPI operation "listProjects"
      When the tester previews and writes an endpoint resource
      Then both commands succeed
      And resource "list-projects" is present

    @baseline @IMPORT-05
    Scenario: Re-import a verified BagIt archive
      Given the ZIP from EXPORT-08 exists
      When the tester imports it offline
      Then its manifest checksum is verified
      And its payload is restored

    @baseline @IMPORT-06
    Scenario Outline: Read a configuration encoding
      Given local bucket configuration encoded as "<format>"
      When the tester imports with that configuration
      Then selected content is transferred

      Examples:
        | format |
        | JSON   |
        | JSONC  |
        | YAML   |

  Rule: Software identifiers are deterministic and verifiable

    @baseline @SWHID-01
    Scenario: Hash file content deterministically
      When the tester hashes the same file twice
      Then both content SWHIDs are identical and valid

    @baseline @SWHID-02
    Scenario: Verify a calculated SWHID
      Given the calculated SWHID for a fixture
      When the tester verifies that fixture
      Then the command succeeds

    @baseline @git @SWHID-03
    Scenario Outline: Hash a supported filesystem or Git object
      Given a disposable repository with an annotated tag
      When the tester calculates a "<kind>" SWHID
      Then a valid SWHID is printed

      Examples:
        | kind      |
        | directory |
        | git-blob  |
        | git-tree  |
        | revision  |
        | release   |
        | snapshot  |

  Rule: Diagnostics and local-model configuration are reviewable

    @baseline @DOCTOR-01
    Scenario: Diagnose the host
      When the tester runs all doctor checks
      Then system, memory, network, GPU, and software findings are understandable

    @baseline @DOCTOR-02
    Scenario: Produce a diagnostic report
      When the tester requests a doctor report
      Then the report is valid JSON

    @baseline @SYNC-01
    Scenario: Preview model synchronization
      When the tester previews VS Code synchronization
      Then a configuration diff is displayed
      And no target file is created

    @baseline @SYNC-02
    Scenario Outline: Write isolated application configuration
      When the tester synchronizes "<application>" to a workspace path
      Then only that workspace configuration is written

      Examples:
        | application |
        | VS Code     |
        | OpenCode    |
        | llama-swap  |
        | Goose       |

    @baseline @system-state @SKILL-01
    Scenario: Extract embedded agent instructions
      When the tester runs the skill command
      Then output identifies an extracted "SKILL.md"
      And clipboard failure does not remove the file

  Rule: Interactive behaviors are manually observable

    @manual @interactive @WATCH-01
    Scenario: Repeat a check after a watched file changes
      Given a check runs in polling watch mode
      When the watched file timestamp changes
      Then one new check occurs
      And q or Control-C stops watch mode

    @manual @interactive @TUI-01
    Scenario: Open, resize, navigate, and exit the TUI
      When the tester starts the TUI with isolated storage
      Then resizing redraws without corruption
      And the displayed quit key exits cleanly

    @baseline @git @GIT-01
    Scenario: Select files changed by a Git commit
      Given a disposable repository has two RAD commits
      When the tester checks files changed in HEAD
      Then the changed RAD file is reported

  Rule: Network workflows remain optional and explicit

    @network @NET-01
    Scenario: Check prose and live links
      When network-enabled checks run
      Then schema, prose, readability, quality, and website checks execute

    @network @NET-02
    Scenario: Resolve a DOI citation
      When DOI "10.1038/nature12373" is resolved as APA
      Then the citation contains publication details

    @network @NET-03
    Scenario: Search DOE CODE
      When projects matching "ACORN" and laboratory "ORNL" are requested
      Then valid JSON results are returned

    @network @NET-04
    Scenario: Preview GGUF metadata
      When model import is dry-run for a public GGUF repository
      Then filenames, quantizations, revisions, and sizes are reported
      And weights are not downloaded

    @network @NET-05
    Scenario: Preview model download selection
      When Q4_K_M download is dry-run
      Then selected-file metadata is reported
      And weights are not downloaded

    @network @NET-06
    Scenario: Download a filtered public bucket
      When only research index files are requested
      Then matching files are transferred without unrelated large content

    @network @NET-07
    Scenario: Preview provider enrichment
      When OpenAlex and CiteAs enrichment is dry-run
      Then existing values remain intact
      And additions appear only in the preview

    @network @database @NET-08
    Scenario: Canonicalize before strict ingestion
      When download uses canonicalize and strict
      Then aliases are rewritten before strict ingestion

  Rule: Infrastructure changes require owner supervision

    @owner @infrastructure @INFRA-01
    Scenario: Inspect infrastructure interfaces without side effects
      When help is requested for runner and bot commands
      Then every help command succeeds
      And no external resource is created

    @owner @infrastructure @INFRA-02
    Scenario: Register an authorized GitLab runner
      Given approved credentials, group, runtime, and cleanup plan
      When the owner creates the acceptance runner
      Then it registers and starts
      And the owner removes it afterward

    @owner @infrastructure @INFRA-03
    Scenario: Create an authorized detached GitLab bot
      Given approved credentials, project, runtime, and cleanup plan
      When the owner creates the acceptance bot
      Then it starts without exposing secrets
      And the owner removes it afterward

    @owner @infrastructure @INFRA-04
    Scenario: Run an authorized in-process bot
      Given approved credentials and a test project
      When the owner starts the polling bot
      Then one test note event is handled once
      And Control-C stops the service cleanly

  Rule: Incomplete interfaces fail clearly

    @gap @negative @GAP-01
    Scenario: Explain that create MCP is incomplete
      When the tester runs "create mcp"
      Then the command fails without changing external state
      And the error says it is not implemented

    @gap @negative @GAP-02
    Scenario: Explain that serve MCP is incomplete
      When the tester runs "serve mcp"
      Then the command fails without starting a service
      And the error says it is not implemented

    @gap @negative @GAP-03
    Scenario: Explain that doctor autofix is incomplete
      When the tester runs doctor with fix enabled
      Then the command fails without changing the host
      And the error says autofix is not implemented

  Rule: Downloaded form receipts can produce a reviewed local demonstration

    @baseline @demo @INTAKE-01
    Scenario: Stage and inspect a collected project JSONL snapshot
      Given three synthetic receipts target three configured local project indexes
      When the tester stages the file as a batch and inspects its batch ID
      Then all three routes and proposed changes are visible
      And the downloaded bytes and all project indexes remain unchanged

    @baseline @demo @negative @INTAKE-02
    Scenario: Require approval before applying a reviewed batch
      When apply runs with redirected output and without explicit approval
      Then it fails without changing any index
      When the reviewed synthetic batch is applied with explicit approval
      Then each project gains exactly one entry
      And quoted, Unicode, and multiline item text is preserved

    @baseline @demo @INTAKE-03
    Scenario: Replaying a receipt or batch does not duplicate content
      Given the three-receipt batch has been applied
      When the tester repeats apply and stages the exact receipts again
      Then all receipts are recognized as replayed
      And subsequent apply leaves index bytes unchanged

    @baseline @demo @INTAKE-04
    Scenario: Correct a submission while retaining its immutable history
      Given the original batch has been applied
      When a new receipt corrects the exact original in a later batch
      Then inspection identifies the root and predecessor
      And apply replaces the logical entry without changing its identifier
      And the complete four-receipt history replays without index changes
      And applying that history in a fresh workspace produces identical indexes
      And original downloaded receipts remain unchanged

    @baseline @demo @negative @INTAKE-05
    Scenario Outline: Reject an invalid collected history without partial application
      Given valid project receipts and "<defect>" in an isolated batch
      When the tester stages and tries to apply the batch
      Then staging marks it non-applicable and apply fails
      And every project index remains unchanged

      Examples:
        | defect                          |
        | missing correction ancestry     |
        | changed authorized owner        |
        | changed registered project      |
        | changed report placement        |
        | competing correction successors |
        | changed bytes under one UUID    |
        | malformed envelope              |

    @baseline @demo @negative @INTAKE-06
    Scenario Outline: Recheck the reviewed batch before writing
      Given a staged batch whose "<input>" changes before approval
      When the tester applies the batch
      Then apply fails and preserves all current index bytes

      Examples:
        | input                  |
        | downloaded source      |
        | destination index      |
        | authorized membership  |

    @baseline @demo @gap @INTAKE-07
    Scenario: Do not silently drop a standalone report-wide update
      Given a mixed batch includes a general destination without a project index
      When the tester stages and tries to apply it
      Then the unsupported general route blocks the whole batch
      And no row is silently discarded or remapped to a project
      And a demo requiring standalone general reporting remains blocked

    @baseline @demo @INTAKE-08
    Scenario: Graduate optional metadata only after reviewing its preview
      Given an ingested synthetic personnel entry in a separate workspace
      When intake run previews with dry-run and force
      Then it proposes a contributor without writing
      When the reviewed promotion is forced with its expected digest
      Then the contributor is added and the source entry is retained archived
      And an exact original-byte backup exists
      And reusing the stale digest fails without another change

    @demo @pdf @manual @INTAKE-09
    Scenario: Produce a readable combined PDF from corrected project highlights
      Given the corrected project indexes and an explicit reporting window
      When report runs offline with dry-run and combine using local defaults
      Then a combined PDF contains the title page, three projects, and legend
      And the corrected highlight appears once without its superseded text
      And the other two highlights appear once and out-of-window entries are absent
      And every rendered page is readable without clipped or overlapping content
      And no index, remote inbox, publication, callback, or notification is changed

    @demo @network @manual @INTAKE-10
    Scenario: Verify the live SharePoint handoff separately from the CLI rehearsal
      Given an authorized standard-license contributor and test project
      And project highlight is the default with project controls gated by placement
      When the contributor submits and corrects a TEST ONLY highlight
      Then receipt feedback is visible after both saves
      And My submissions occupies a separate row below the banner
      And only their own history is visible with project and placement fixed during editing
      And the contributor and operator receive Teams confirmations
      And routine submission sends no milestone email
      And a fresh collected export retains both linked receipts and previous rows
      When an operator reviews and imports the exact selected history locally
      Then the report contains the corrected highlight once
      And the GitLab connector branch remains disabled

    @demo @interactive @manual @INTAKE-11
    Scenario: Confirm or cancel intake and graduation in a real terminal
      Given the runner's staged but unapplied confirmation workspace
      When the tester presses default No, Escape, or Ctrl+C during apply
      Then no index changes
      When the tester selects Yes for the reviewed batch
      Then the expected entries are applied once
      When the tester declines a graduation candidate or its final write
      Then the index and backup inventory remain unchanged
      When the tester approves the candidate and final write
      Then graduation retains an archived source and exact-byte backup

  Rule: Global controls and cleanup are safe

    @baseline @GLOBAL-01
    Scenario: Adjust output verbosity
      When equivalent commands run in quiet and verbose modes
      Then quiet output is smaller than verbose output

    @baseline @GLOBAL-02
    Scenario: Limit processing to one thread
      When gather runs with one worker thread
      Then the command succeeds

    @baseline @negative @GLOBAL-03
    Scenario: Reject conflicting database controls
      When database-path and no-local-database are both supplied
      Then parsing fails and explains the conflict

    @optional @database @GLOBAL-04
    Scenario: Request the DuckDB backend
      When gather uses an isolated DuckDB path
      Then a DuckDB build creates the database
      But a SQLite-only build clearly reports unavailable support

    @baseline @database @destructive-isolated @GLOBAL-05
    Scenario: Reset only the disposable SQLite database
      Given its path resolves beneath the acceptance workspace
      When the tester resets it and gathers one DOI
      Then no database outside the workspace changes

    @manual @cleanup @CLEANUP-01
    Scenario: Review retained artifacts
      When the tester lists the workspace recursively
      Then failure evidence can be retained

    @manual @cleanup @destructive-isolated @CLEANUP-02
    Scenario: Delete only the current acceptance workspace
      Given the target resolves beneath "target/qa-acceptance"
      When cleanup is confirmed after evidence is saved
      Then only that timestamped workspace is deleted
