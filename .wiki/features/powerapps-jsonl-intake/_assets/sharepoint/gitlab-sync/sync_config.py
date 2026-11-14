"""Validate the reusable sync values before generating a flow definition."""

from uuid import UUID


def validate(config):
    required_text = (
        'approvedExportLibrary', 'approvedGitLabPath', 'gitLabBranch',
        'projectListId', 'sharePointSiteUrl',
    )
    if any(not isinstance(config.get(key), str) or not config[key].strip()
           for key in required_text):
        raise ValueError('Sync configuration has a missing text field.')
    if not config['sharePointSiteUrl'].startswith('https://'):
        raise ValueError('SharePoint site must use HTTPS.')
    if '/' in config['approvedExportLibrary'] or "'" in config['approvedExportLibrary']:
        raise ValueError('Export library must be a single folder name without quotes.')
    if not isinstance(config.get('gitLabProjectId'), int) or config['gitLabProjectId'] <= 0:
        raise ValueError('GitLab project ID must be a positive integer.')
    if '..' in config['gitLabBranch'] or any(ord(c) < 32 for c in config['gitLabBranch']):
        raise ValueError('GitLab branch contains invalid characters.')
    path = config['approvedGitLabPath']
    if not path.startswith('.acorn/approved/') or not path.endswith('.jsonl') or '..' in path:
        raise ValueError('Approved output must use a dedicated .acorn/approved/*.jsonl path.')
    try:
        UUID(config['projectListId'])
    except ValueError as error:
        raise ValueError('SharePoint project list ID must be a UUID.') from error
    for key in ('approvedSourceReady', 'catalogSyncReady', 'gitLabWriteReady'):
        if not isinstance(config.get(key), bool):
            raise ValueError(key + ' must be a Boolean.')
    return config
