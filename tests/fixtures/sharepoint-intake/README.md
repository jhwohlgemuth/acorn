# Synthetic SharePoint source fixtures

`source-cases.json` defines direct document URL matching. The accepted URL is mapped by canonical host, site, library and item path. Every rejected URL fails before credential lookup or network access. `receipt-empty.json` fixes the version-1 receipt shape and empty-file digest. Identifiers and hosts are invented; these fixtures make no assertion about ORNL tenant access.
