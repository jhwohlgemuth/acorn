"""Execute the generated WDL subset with synthetic connector responses, never HTTP.

This checks expressions and runAfter routing, not Microsoft's hosted engine or
connector response contracts. Unsupported expressions/actions fail the tests.
"""
import base64
from datetime import datetime, timedelta, timezone
import json
import re
from urllib.parse import unquote, urlsplit


class FlowFailure(Exception):
    pass


def evaluate(expression, context):
    if isinstance(expression, list):
        return [evaluate(value, context) for value in expression]
    if isinstance(expression, dict):
        return {key: evaluate(value, context) for key, value in expression.items()}
    if not isinstance(expression, str) or not expression.startswith('@'):
        return expression
    source = expression[1:]
    cursor = 0

    def parse_date(value):
        return datetime.fromisoformat(value.replace('Z', '+00:00'))

    def empty(value):
        if value is not None and not isinstance(value, (str, list, dict)):
            raise TypeError('empty requires a string, array, object, or null')
        return not value

    def value():
        nonlocal cursor
        while cursor < len(source) and source[cursor].isspace():
            cursor += 1
        if source[cursor] == "'":
            cursor += 1
            result = ''
            while cursor < len(source):
                if source[cursor:cursor + 2] == "''":
                    result += "'"
                    cursor += 2
                elif source[cursor] == "'":
                    cursor += 1
                    break
                else:
                    result += source[cursor]
                    cursor += 1
        else:
            match = re.match(r'-?\d+|[A-Za-z_][A-Za-z_0-9]*', source[cursor:])
            assert match, (source, cursor)
            token = match.group()
            cursor += len(token)
            if token.lstrip('-').isdigit():
                result = int(token)
            elif token in ('null', 'true', 'false'):
                result = {'null': None, 'true': True, 'false': False}[token]
            else:
                assert source[cursor] == '(', (source, cursor)
                cursor += 1
                arguments = []
                while source[cursor] != ')':
                    arguments.append(value())
                    if source[cursor] == ',':
                        cursor += 1
                cursor += 1
                functions = {
                    'actions': lambda name: context['actions'].get(name, {}),
                    'addHours': lambda date, hours: (parse_date(date) + timedelta(hours=hours)).isoformat(),
                    'and': lambda *xs: all(xs),
                    'base64': lambda text: base64.b64encode(text.encode()).decode(),
                    'base64ToString': lambda x: base64.b64decode(x, validate=True).decode('utf-8'),
                    'body': lambda name: context['outputs'].get(name, {}).get('body'),
                    'coalesce': lambda *xs: next((x for x in xs if x is not None), None),
                    'concat': lambda *xs: ''.join(xs),
                    'contains': lambda collection, x: x in collection,
                    'createArray': lambda *xs: list(xs),
                    'decodeUriComponent': unquote,
                    'div': lambda a, b: a // b,
                    'empty': empty,
                    'endsWith': lambda text, suffix: text.lower().endswith(suffix.lower()),
                    'equals': lambda a, b: a == b and not (isinstance(a, bool) != isinstance(b, bool)),
                    'first': lambda xs: xs[0] if xs else None,
                    'greater': lambda a, b: a > b,
                    'if': lambda condition, a, b: a if condition else b,
                    'int': int,
                    'item': lambda: context['item'],
                    'items': lambda name: context['items'][name],
                    'json': json.loads,
                    'length': len,
                    'lessOrEquals': lambda a, b: a <= b,
                    'mul': lambda a, b: a * b,
                    'not': lambda x: not x,
                    'or': lambda *xs: any(xs),
                    'outputs': lambda name: context['outputs'].get(name),
                    'replace': lambda text, a, b: text.replace(a, b),
                    'setProperty': lambda obj, key, data: {**obj, key: data},
                    'split': lambda text, separator: text.split(separator),
                    'startsWith': lambda text, prefix: text.lower().startswith(prefix.lower()),
                    'string': lambda x: json.dumps(x, separators=(',', ':')) if isinstance(x, (dict, list)) else str(x) if x is not None else '',
                    'sub': lambda a, b: a - b,
                    'substring': lambda text, start, length: text[start:start + length],
                    'ticks': lambda date: int((parse_date(date) - datetime(1, 1, 1, tzinfo=timezone.utc)).total_seconds() * 10000000),
                    'take': lambda collection, count: collection[:count],
                    'toLower': lambda text: text.lower(),
                    'trim': lambda text: text.strip(),
                    'union': lambda a, b: list(dict.fromkeys(a + b)),
                    'uriPath': lambda url: urlsplit(url).path,
                    'utcNow': lambda: context['now'],
                    'workflow': lambda: {'run': {'name': 'synthetic-run'}},
                }
                assert token in functions, token
                result = functions[token](*arguments)
        while source[cursor:cursor + 2] == '?[':
            cursor += 2
            key = value()
            assert source[cursor] == ']'
            cursor += 1
            result = (result or {}).get(key)
        return result

    result = value()
    assert cursor == len(source), (source, cursor)
    return result


def predicate(expression, context):
    operator, operands = next(iter(expression.items()))
    if operator == 'and':
        return all(predicate(item, context) for item in operands)
    if operator == 'or':
        return any(predicate(item, context) for item in operands)
    if operator == 'not':
        return not predicate(operands, context)
    a, b = [evaluate(item, context) for item in operands]
    operations = {
        'equals': lambda: a == b and not (isinstance(a, bool) != isinstance(b, bool)),
        'greater': lambda: a > b,
        'greaterOrEquals': lambda: a >= b,
        'lessOrEquals': lambda: a <= b,
        'startsWith': lambda: a.startswith(b),
        'endsWith': lambda: a.endswith(b),
    }
    assert operator in operations, operator
    return operations[operator]()


def execute(actions, connector, context=None):
    context = context if context is not None else {
        'actions': {}, 'outputs': {}, 'items': {}, 'now': '2026-11-24T12:00:00Z', 'trace': [],
    }

    def run(graph):
        pending = dict(graph)
        local = {}
        while pending:
            ready = [name for name, action in pending.items() if all(dep in local for dep in action.get('runAfter', {}))]
            assert ready, ('Unresolved dependencies', pending.keys())
            for name in ready:
                action = pending.pop(name)
                status = 'Succeeded'
                output = None
                if not all(local[dep] in statuses for dep, statuses in action.get('runAfter', {}).items()):
                    status = 'Skipped'
                else:
                    try:
                        kind = action['type']
                        if kind == 'Compose':
                            output = evaluate(action['inputs'], context)
                        elif kind == 'If':
                            branch = action['actions'] if predicate(action['expression'], context) else action['else']['actions']
                            status = run(branch)
                        elif kind == 'Scope':
                            status = run(action['actions'])
                        elif kind == 'Terminate':
                            raise FlowFailure(action['inputs']['runError']['code'])
                        elif kind == 'OpenApiConnection':
                            parameters = evaluate(action['inputs']['parameters'], context)
                            status, output = connector(name, parameters, action)
                        elif kind in ('Query', 'Select'):
                            rows = evaluate(action['inputs']['from'], context)
                            output = {'body': []}
                            for row in rows:
                                context['item'] = row
                                if kind == 'Select':
                                    output['body'].append(evaluate(action['inputs']['select'], context))
                                elif evaluate(action['inputs']['where'], context):
                                    output['body'].append(row)
                        elif kind == 'Foreach':
                            statuses = []
                            for row in evaluate(action['foreach'], context):
                                context['items'][name] = row
                                statuses.append(run(action['actions']))
                            status = 'Failed' if 'Failed' in statuses else 'Succeeded'
                        elif kind == 'ParseJson':
                            data = evaluate(action['inputs']['content'], context)
                            schema = action['inputs']['schema']
                            types = {'string': str, 'integer': int}
                            if not isinstance(data, dict) or not all(key in data for key in schema['required']):
                                raise ValueError('Invalid receipt shape')
                            for key, field in schema['properties'].items():
                                if type(data[key]) is not types[field['type']]:
                                    raise ValueError('Invalid receipt field')
                            output = {'body': data}
                        else:
                            raise AssertionError(('Unsupported action', kind))
                    except (ValueError, TypeError, KeyError, UnicodeError) as error:
                        status = 'Failed'
                        output = {'error': str(error)}
                local[name] = status
                context['actions'][name] = {'status': status}
                context['outputs'][name] = output
                context['trace'].append((name, status))
        dependencies = {dep for action in graph.values() for dep in action.get('runAfter', {})}
        leaves = [status for name, status in local.items() if name not in dependencies]
        unhandled = [name for name, status in local.items() if status in ('Failed', 'TimedOut')
                     and not any(name in action.get('runAfter', {}) and local[child] != 'Skipped'
                                 for child, action in graph.items())]
        return 'Failed' if unhandled or any(status in ('Failed', 'TimedOut') for status in leaves) else 'Succeeded'

    try:
        status = run(actions)
    except FlowFailure as error:
        status = 'Failed'
        context['error'] = str(error)
    return status, context
