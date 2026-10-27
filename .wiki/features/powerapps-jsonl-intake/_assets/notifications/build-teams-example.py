#!/usr/bin/env python3
"""Rebuild the guides and fictional SharePoint owner/submitter Teams illustration together."""
from pathlib import Path
import runpy

if __name__ == '__main__':
    runpy.run_path(str(Path(__file__).resolve().parents[1] / 'build-workflow-guides.py'), run_name='__main__')
