"""Readable shell examples for the workflow PDFs; no commands are executed."""
import html
import re
import shlex
import textwrap

from reportlab.lib.colors import HexColor
from reportlab.lib.styles import ParagraphStyle
from reportlab.pdfbase.pdfmetrics import stringWidth
from reportlab.platypus import Paragraph


def shell_block(text, width, size=12, minimum=10):
    """Prefer one line per command; wrap only when readable text cannot fit."""
    logical = re.sub(r'[ \t]*\\\n[ \t]*', ' ', text).strip('\n').splitlines()
    longest = max((stringWidth(line, 'Courier', size) for line in logical), default=1)
    font_size = max(minimum, min(size, size * (width - 20) / max(longest, 1)))
    limit = int((width - 20) / stringWidth('M', 'Courier', font_size))
    def fit(line):
        if len(line) <= limit:
            return [line]
        if line.startswith('#'):
            return ['# ' + part for part in textwrap.wrap(line[1:].strip(), limit - 2)]
        lexer = shlex.shlex(line, posix=False)
        lexer.whitespace_split = True
        lexer.commenters = ''
        rows, current = [], ''
        for word in lexer:
            candidate = current + (' ' if current.strip() else '') + word
            if len(candidate) > limit - 2 and current.strip():
                rows.append(current + ' \\')
                current = '  ' + word
            else:
                current = candidate
        return rows + [current]
    lines = [line for original in logical for line in fit(original)]
    if any(stringWidth(line, 'Courier', font_size) > width - 16 for line in lines):
        raise ValueError('A shell token is too wide; shorten its path or use a named variable.')
    style = ParagraphStyle('Shell', fontName='Courier', fontSize=font_size, leading=font_size * 1.55,
                           textColor=HexColor('#243B45'), backColor=HexColor('#F0F5F6'),
                           borderColor=HexColor('#D5E3E4'), borderWidth=0.6, borderPadding=8,
                           spaceBefore=8, spaceAfter=16)
    return Paragraph('<br/>'.join(shell_markup(line) for line in lines), style)


def shell_markup(line):
    """Color shell example tokens while preserving the displayed command text."""
    parts = re.findall(r'''\#[^\n]*|"(?:\\.|[^"\\])*"|'[^']*'|\s+|[^\s]+''', line)
    commands = {'acorn', 'apply', 'cd', 'inspect', 'intake', 'python3', 'report', 'run', 'stage'}
    def styled(token):
        if token.startswith('#'):
            color, bold = '#526570', False
        elif token in commands:
            color, bold = '#00664D', True
        elif token.startswith('-'):
            color, bold = '#174EA6', True
        elif token.startswith(('"', "'", '$', '<')) or token == 'BATCH_ID' or '=' in token:
            color, bold = '#7A3E86', False
        elif '/' in token or token.endswith(('.json', '.jsonl')):
            color, bold = '#8A4518', False
        else:
            color, bold = '#243B45', False
        escaped = html.escape(token).replace(' ', '&#160;')
        return f'<font color="{color}" name="Courier-Bold">{escaped}</font>' if bold else f'<font color="{color}">{escaped}</font>'
    return ''.join(styled(token) for token in parts)
