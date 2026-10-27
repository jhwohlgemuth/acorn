# Synthetic Graph byte fixture

The content is an invented eight-byte binary payload, not JSONL. A successful Graph file service returns the decoded bytes unchanged. The intake adapter rejects it as a JSONL source. Metadata and content responses identify the same drive item and revision; a changed ETag or size must fail consistency verification.
