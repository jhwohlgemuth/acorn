"""Validate the reusable sync values before generating a flow definition."""

import re
from urllib.parse import urlsplit
from uuid import UUID


def validate(config):
    required_text = (
        'approvedExportLibrary', 'approvedGitLabPath', 'gitLabBranch',
        'projectListId', 'sharePointSiteUrl',
    )
    flags = ('approvedSourceReady', 'catalogSyncReady', 'gitLabWriteReady')
    allowed = set(required_text) | set(flags) | {'gitLabProjectId', 'maxReceiptAgeHours'}
    if not isinstance(config, dict) or set(config) - allowed:
        raise ValueError('Sync configuration must contain only known settings.')
    if any(not isinstance(config.get(key), str) or not config[key].strip()
           or config[key] != config[key].strip()
           or any(ord(c) < 32 or ord(c) == 127 for c in config[key])
           for key in required_text):
        raise ValueError('Sync configuration has a missing text field.')
    site = urlsplit(config['sharePointSiteUrl'])
    if (site.scheme != 'https' or not site.hostname or site.username or site.password
            or site.query or site.fragment or site.port not in (None, 443)
            or any(c in config['sharePointSiteUrl'] for c in "'\"\\% ")
            or any(part in ('.', '..') for part in site.path.split('/'))
            or '//' in site.path or site.path.endswith('/')):
        raise ValueError('SharePoint site must be a canonical HTTPS URL without credentials, escapes, query, or fragment.')
    library = config['approvedExportLibrary']
    if library in ('.', '..') or any(c in library for c in "/\\'\"%?#@{}"):
        raise ValueError('Export library must be one literal folder name.')
    if type(config.get('gitLabProjectId')) is not int or config['gitLabProjectId'] <= 0:
        raise ValueError('GitLab project ID must be a positive integer.')
    branch = config['gitLabBranch']
    if (branch.startswith('@') or '..' in branch or '@{' in branch
            or any(c in branch for c in ' ~^:?*[\\') or branch.endswith('.')
            or any(not part or part.startswith('.') or part.endswith('.lock') for part in branch.split('/'))):
        raise ValueError('GitLab branch contains invalid characters.')
    path = config['approvedGitLabPath']
    if (not re.fullmatch(r'\.acorn/approved/(?:[A-Za-z0-9_-][A-Za-z0-9_.-]*/)*[A-Za-z0-9_-][A-Za-z0-9_.-]*\.jsonl', path)
            or '..' in path):
        raise ValueError('Approved output must use a dedicated .acorn/approved/*.jsonl path.')
    try:
        parsed_id = UUID(config['projectListId'])
        if str(parsed_id) != config['projectListId'].lower() or parsed_id.int == 0:
            raise ValueError('Noncanonical or empty UUID')
    except ValueError as error:
        raise ValueError('SharePoint project list ID must be a UUID.') from error
    for key in flags:
        if not isinstance(config.get(key), bool):
            raise ValueError(key + ' must be a Boolean.')
    age = config.get('maxReceiptAgeHours', 48)
    if type(age) is not int or not 1 <= age <= 8760:
        raise ValueError('Receipt age must be a positive integer no greater than one year in hours.')
    return {**config, 'maxReceiptAgeHours': age}
