"""Exercise the generated flow's actual filter and month expressions offline.

This small evaluator covers only functions used by these contract tests. Live
SharePoint binding, REST behavior, formatting, and access still need acceptance.
"""
import base64
import calendar
from datetime import datetime, timezone
import json
import re
import unittest
from zoneinfo import ZoneInfo

from build_dashboard import provision, refresh
from build_views import configure, month_formatter, project_formatter

FLOW = refresh()['operationDefinition']['actions']
WORK = FLOW['Bounded_valid_input_COV']['actions']
MONTH = WORK['Bound_months_COV']['actions']['For_month_COV']['actions']
PROJECT = MONTH['For_project_COV']['actions']


def parse_date(value):
    return datetime.fromisoformat(value.replace('Z', '+00:00'))


def add_months(value, count, unit):
    assert unit == 'Month'
    date = parse_date(value)
    year, month = divmod(date.year*12+date.month-1+count, 12)
    return date.replace(year=year, month=month+1, day=min(date.day, calendar.monthrange(year, month+1)[1])).isoformat()


def evaluate(expression, context):
    """Interpret the subset of WDL used by the coverage filters/composes."""
    if not isinstance(expression, str) or not expression.startswith('@'):
        return expression
    source = expression[1:]
    cursor = 0

    def value():
        nonlocal cursor
        while cursor < len(source) and source[cursor].isspace():
            cursor += 1
        if source[cursor] == "'":
            cursor += 1
            result = ''
            while cursor < len(source):
                if source[cursor:cursor+2] == "''":
                    result += "'"
                    cursor += 2
                elif source[cursor] == "'":
                    cursor += 1
                    break
                else:
                    result += source[cursor]
                    cursor += 1
        else:
            token = re.match(r'-?\d+|[A-Za-z_][A-Za-z_0-9]*', source[cursor:]).group()
            cursor += len(token)
            if token.lstrip('-').isdigit():
                result = int(token)
            elif token in ('null', 'true', 'false'):
                result = {'null': None, 'true': True, 'false': False}[token]
            else:
                assert source[cursor] == '(', (source, cursor)
                cursor += 1
                args = []
                while source[cursor] != ')':
                    args.append(value())
                    if source[cursor] == ',':
                        cursor += 1
                cursor += 1
                functions = {
                    'add': lambda a, b: a+b, 'addToTime': add_months,
                    'and': lambda *xs: all(xs), 'base64ToString': lambda x: base64.b64decode(x).decode(),
                    'body': lambda name: context['body'][name],
                    'coalesce': lambda *xs: next((x for x in xs if x is not None), None),
                    'concat': lambda *xs: ''.join(xs), 'contains': lambda xs, x: x in xs,
                    'convertTimeZone': lambda x, _a, _b: parse_date(x).astimezone(ZoneInfo('America/New_York')).isoformat(),
                    'empty': lambda x: not x, 'equals': lambda a, b: a == b,
                    'first': lambda x: x[0] if x else None,
                    'formatDateTime': lambda x, fmt: parse_date(x).strftime({'yyyy-MM': '%Y-%m', 'MMMM yyyy': '%B %Y'}[fmt]),
                    'if': lambda cond, a, b: a if cond else b, 'int': int,
                    'item': lambda: context['item'], 'items': lambda name: context['items'][name],
                    'join': lambda values, separator: separator.join(values),
                    'json': lambda x: json.loads(x) if x else None,
                    'last': lambda x: x[-1] if x else None, 'length': len,
                    'less': lambda a, b: a < b, 'mul': lambda a, b: a*b,
                    'not': lambda x: not x, 'or': lambda *xs: any(xs),
                    'outputs': lambda name: context['outputs'][name],
                    'replace': lambda text, old, new: text.replace(old, new),
                    'string': lambda x: '' if x is None else str(x), 'sub': lambda a, b: a-b,
                    'substring': lambda x, start, count: x[start:start+count],
                    'utcNow': lambda: context['now'],
                }
                result = functions[token](*args)
        while source[cursor:cursor+2] == '?[':
            cursor += 2
            key = value()
            assert source[cursor] == ']'
            cursor += 1
            result = (result or {}).get(key)
        return result

    result = value()
    assert cursor == len(source), (source, cursor)
    return result


def apply_query(action, rows, context=None):
    context = context or {}
    return [row for row in rows if evaluate(action['inputs']['where'], {**context, 'item': row})]


def evaluate_condition(expression, context):
    """Evaluate structured WDL conditions without emulating connector actions."""
    if not isinstance(expression, dict):
        return evaluate(expression, context)
    operation, operands = next(iter(expression.items()))
    if operation == 'not':
        return not evaluate_condition(operands, context)
    values = [evaluate_condition(value, context) for value in operands]
    return {
        'and': lambda: all(values), 'equals': lambda: values[0] == values[1],
        'greater': lambda: values[0] > values[1],
        'lessOrEquals': lambda: values[0] <= values[1], 'or': lambda: any(values),
    }[operation]()


class CoverageTests(unittest.TestCase):
    def test_actual_project_count_includes_tests_without_approval_gate(self):
        rows = [{'id': 'a', 'project': 'alpha', 'summary': 'TEST ONLY', 'export': 'Pending'}, {'id': 'b', 'project': 'alpha'}, {'id': 'c', 'project': 'general'}]
        matched = apply_query(PROJECT['Project_receipts_COV'], rows, {'items': {'For_project_COV': {'id': 'alpha'}}})
        self.assertEqual(len(matched), 2)
        self.assertEqual(len(apply_query(MONTH['General_COV'], rows)), 1)

    def test_all_compose_expressions_have_balanced_delimiters(self):
        def visit(node):
            if isinstance(node, dict):
                for value in node.values():
                    visit(value)
            elif isinstance(node, list):
                for value in node:
                    visit(value)
            elif isinstance(node, str) and node.startswith('@'):
                text = re.sub(r"'(?:[^']|'')*'", "''", node)
                self.assertEqual(text.count('('), text.count(')'), node)
                self.assertEqual(text.count('['), text.count(']'), node)
        visit(FLOW)

    def test_corrections_do_not_count_as_new_submissions(self):
        rows = [{'id': 'a', 'revision': None}, {'id': 'b', 'revision': {'rootSubmissionId': 'a', 'supersedesSubmissionId': 'a'}}, {'id': 'c', 'revision': {'rootSubmissionId': 'a', 'supersedesSubmissionId': 'b'}}]
        self.assertEqual(apply_query(WORK['Originals_COV'], rows), [rows[0]])

    def test_eastern_month_boundary_and_dst(self):
        context = {'outputs': {'Month_key_COV': '2026-10'}}
        rows = [{'created': '2026-10-01T03:59:59Z'}, {'created': '2026-10-01T04:00:00Z'}, {'created': '2026-11-01T03:59:59Z'}, {'created': '2026-11-01T04:00:00Z'}]
        self.assertEqual(apply_query(MONTH['Month_receipts_COV'], rows, context), rows[1:3])
        context['outputs']['Month_key_COV'] = '2026-12'
        rows = [{'created': '2026-12-01T04:59:59Z'}, {'created': '2026-12-01T05:00:00Z'}]
        self.assertEqual(apply_query(MONTH['Month_receipts_COV'], rows, context), rows[1:])

    def test_existing_roster_ignores_later_catalog_changes(self):
        roster = [{'id': 'retired-project', 'name': 'Original name', 'catalogId': 3}]
        context = {'body': {'Read_month_COV': [{'ExpectedProjects': json.dumps(roster)}], 'Roster_COV': [{'id': 'new-project'}]}}
        self.assertEqual(evaluate(MONTH['Month_roster_COV']['inputs'], context), roster)

    def test_failed_project_refresh_cannot_advance_month_success(self):
        for project_status in ['Succeeded', 'Failed', 'Skipped', 'TimedOut']:
            statuses = {'For_project_COV': project_status}
            for name in ['Month_item_ids_COV', 'Month_links_COV', 'Month_values_COV', 'Upsert_month_COV']:
                dependencies = MONTH[name]['runAfter']
                statuses[name] = 'Succeeded' if all(statuses.get(parent) in allowed for parent, allowed in dependencies.items()) else 'Skipped'
            self.assertEqual(statuses['Upsert_month_COV'], 'Succeeded' if project_status == 'Succeeded' else 'Skipped')

    def test_initial_window_contains_twelve_months_and_year_boundary(self):
        context = {'outputs': {'Current_month_COV': '2026-10'}}
        start = evaluate(WORK['Initial_start_COV']['inputs'], context)
        self.assertEqual(start, '2025-11')
        context['outputs']['Start_month_COV'] = start
        self.assertEqual(evaluate(WORK['Month_count_COV']['inputs'], context), 12)
        context['items'] = {'For_month_COV': 2}
        self.assertEqual(evaluate(MONTH['Month_key_COV']['inputs'], context), '2026-01')

    def test_invalid_receipts_reject_identity_and_parent_mismatches(self):
        original = {'id': 'a', 'payloadId': 'a', 'project': 'alpha', 'payloadProject': 'alpha', 'created': '2026-10-28T12:00:00Z', 'parent': None, 'revision': None}
        correction = {**original, 'id': 'b', 'payloadId': 'b', 'parent': 'a', 'revision': {'supersedesSubmissionId': 'a'}}
        invalid = [{**original, 'payloadId': 'wrong'}, {**original, 'payloadProject': 'wrong'}, {**correction, 'parent': 'wrong'}, {**original, 'created': None}]
        self.assertEqual(apply_query(FLOW['Invalid_receipts_COV'], [original, correction, *invalid]), invalid)
        self.assertNotIn(",''", FLOW['Invalid_receipts_COV']['inputs']['where'])

    def test_month_rollover_uses_eastern_time_and_retires_previous_current_flag(self):
        for now, expected in [('2026-11-01T03:59:59Z', '2026-10'), ('2026-11-01T04:00:00Z', '2026-11')]:
            self.assertEqual(evaluate(WORK['Current_month_COV']['inputs'], {'now': now}), expected)
        changed = MONTH['Upsert_month_COV']['else']['actions']['Month_changed_COV']['expression']
        context = {'outputs': {'Month_key_COV': '2026-10', 'Month_label_COV': 'October 2026', 'Current_month_COV': '2026-11'}, 'body': {'Read_month_COV': [{'MonthLabel': 'October 2026', 'IsCurrentMonth': True, 'HighlightCount': 0, 'ProjectsSubmitted': 0}], 'Month_receipts_COV': [], 'Month_item_ids_COV': [], 'Month_links_COV': [], 'Covered_roster_COV': []}}
        self.assertTrue(evaluate_condition(changed, context))
        self.assertFalse(evaluate(MONTH['Month_values_COV']['inputs']['IsCurrentMonth'], context))
        context['body']['Read_month_COV'][0]['IsCurrentMonth'] = False
        self.assertFalse(evaluate_condition(changed, context))

    def test_month_label_is_full_name_while_keys_remain_sortable(self):
        context = {'outputs': {'Month_key_COV': '2026-10'}}
        self.assertEqual(evaluate(MONTH['Month_label_COV']['inputs'], context), 'October 2026')
        context['outputs']['Month_label_COV'] = 'October 2026'
        month_values = MONTH['Month_values_COV']['inputs']
        project_values = PROJECT['Project_values_COV']['inputs']
        self.assertEqual(evaluate(month_values['Title'], context), '2026-10')
        self.assertEqual(evaluate(month_values['MonthLabel'], context), 'October 2026')
        self.assertEqual(evaluate(project_values['MonthKey'], context), '2026-10')
        self.assertEqual(evaluate(project_values['MonthLabel'], context), 'October 2026')
        self.assertIn('[$MonthLabel]', json.dumps(month_formatter()))
        self.assertIn('[$MonthLabel]', json.dumps(project_formatter()))

    def test_no_receipts_still_yields_zero_coverage(self):
        self.assertEqual(apply_query(PROJECT['Project_receipts_COV'], [], {'items': {'For_project_COV': {'id': 'alpha'}}}), [])
        self.assertEqual(apply_query(MONTH['Covered_roster_COV'], [{'id': 'alpha'}], {'body': {'Submitted_project_ids_COV': []}}), [])

    def test_provision_creates_columns_only_after_successful_empty_lookup(self):
        actions = provision()['operationDefinition']['actions']
        name = 'List_0_ExpectedProjects'
        self.assertIn("/fields?$filter=InternalName eq 'ExpectedProjects'", actions[name]['inputs']['parameters']['parameters/uri'])
        def selected_actions(action, context):
            left, right = action['expression']['equals']
            matched = evaluate(left, context) == evaluate(right, context)
            return action['actions'] if matched else action['else']['actions']
        for status, rows, expected in [(200, [], 'create'), (200, [{'InternalName': 'ExpectedProjects'}], None), (400, None, 'stop'), (403, None, 'stop'), (500, None, 'stop')]:
            context = {'outputs': {name: {'statusCode': status}}, 'body': {name: {'value': rows}}}
            selected = selected_actions(actions[name+'_ensure'], context)
            if name+'_missing' in selected:
                selected = selected_actions(selected[name+'_missing'], context)
            self.assertEqual(list(selected), [name+'_'+expected] if expected else [])

    def test_rest_formatters_round_trip_without_xml_metacharacters(self):
        actions = configure()['operationDefinition']['actions']
        for name, expected in [('Months_view', month_formatter()), ('Projects_view', project_formatter())]:
            formatter = json.loads(actions[name]['inputs']['parameters']['parameters/body'])['CustomFormatter']
            self.assertEqual(json.loads(formatter), expected)
            self.assertFalse(any(char in formatter for char in '&<>'))

    def test_repeated_project_submissions_credit_project_once(self):
        roster = [{'id': 'alpha'}, {'id': 'beta'}]
        covered = apply_query(MONTH['Covered_roster_COV'], roster, {'body': {'Submitted_project_ids_COV': ['alpha', 'alpha', 'general']}})
        self.assertEqual(covered, [roster[0]])

    def test_submission_links_follow_original_month_and_project_filters(self):
        rows = [
            {'itemId': '1', 'project': 'alpha', 'created': '2026-10-01T03:59:59Z', 'revision': None},
            {'itemId': '2', 'project': 'alpha', 'created': '2026-10-01T04:00:00Z', 'revision': None},
            {'itemId': '3', 'project': 'general', 'created': '2026-10-02T12:00:00Z', 'revision': None},
            {'itemId': '4', 'project': 'alpha', 'created': '2026-10-03T12:00:00Z', 'revision': {'supersedesSubmissionId': '2'}},
        ]
        originals = apply_query(WORK['Originals_COV'], rows)
        month = apply_query(MONTH['Month_receipts_COV'], originals, {'outputs': {'Month_key_COV': '2026-10'}})
        project = apply_query(PROJECT['Project_receipts_COV'], month, {'items': {'For_project_COV': {'id': 'alpha'}}})
        for actions, prefix, receipts, expected in [(MONTH, 'Month', month, '2;3'), (PROJECT, 'Project', project, '2'), (PROJECT, 'Project', [], '')]:
            select = actions[prefix+'_item_ids_COV']['inputs']
            ids = [evaluate(select['select'], {'item': row}) for row in receipts]
            result = evaluate(actions[prefix+'_values_COV']['inputs']['ReceiptItemIds'], {'body': {prefix+'_item_ids_COV': ids}})
            self.assertEqual(result, expected)

    def test_submission_link_backfill_and_repair_without_count_change(self):
        condition = PROJECT['Upsert_project_COV']['else']['actions']['Project_changed_COV']['expression']
        row = {'HighlightCount': 1, 'Title': 'Alpha', 'MonthLabel': 'October 2026'}
        context = {'outputs': {'Month_label_COV': 'October 2026'}, 'items': {'For_project_COV': {'name': 'Alpha'}}, 'body': {'Read_project_COV': [row], 'Project_receipts_COV': [{}], 'Project_item_ids_COV': ['7'], 'Project_links_COV': []}}
        self.assertTrue(evaluate_condition(condition, context))
        row['ReceiptItemIds'] = '8'
        self.assertTrue(evaluate_condition(condition, context))
        row['ReceiptItemIds'] = '7'
        self.assertFalse(evaluate_condition(condition, context))

    def test_summary_encoding_preserves_delimiters_unicode_and_markup(self):
        summaries = ['Two; parts | 50% complete', 'Literal %3B %7C %25', '“Quotes” & <b>plain text</b>\nSecond line', 'x'*1200]
        expression = PROJECT['Project_links_COV']['inputs']['select']
        encoded = [evaluate(expression, {'item': {'itemId': str(i+1), 'summary': value}}) for i, value in enumerate(summaries)]
        records = ';'.join(encoded).split(';')
        for i, (record, expected) in enumerate(zip(records, summaries)):
            item_id, summary = record.split('|')
            self.assertEqual(item_id, str(i+1))
            self.assertEqual(summary.replace('%7C', '|').replace('%3B', ';').replace('%25', '%'), expected)
        self.assertEqual(evaluate(expression, {'item': {'itemId': '9'}}), '9|Summary unavailable')

    def test_summary_updates_repair_tooltips_without_changing_counts(self):
        condition = PROJECT['Upsert_project_COV']['else']['actions']['Project_changed_COV']['expression']
        row = {'HighlightCount': 1, 'Title': 'Alpha', 'MonthLabel': 'October 2026', 'ReceiptItemIds': '7', 'ReceiptLinks': '7|Old summary'}
        context = {'outputs': {'Month_label_COV': 'October 2026'}, 'items': {'For_project_COV': {'name': 'Alpha'}}, 'body': {'Read_project_COV': [row], 'Project_receipts_COV': [{}], 'Project_item_ids_COV': ['7'], 'Project_links_COV': ['7|New summary']}}
        self.assertTrue(evaluate_condition(condition, context))
        row['ReceiptLinks'] = '7|New summary'
        self.assertFalse(evaluate_condition(condition, context))

    def test_truncated_reads_and_history_overflow_stop_before_writes(self):
        bodies = {name: {'value': []} for name in ['Read_receipts_COV', 'Read_catalog_COV', 'Read_first_month_COV', 'Read_coverage_COV']}
        bodies['Invalid_receipts_COV'] = []
        guard = FLOW['Bounded_valid_input_COV']['expression']
        self.assertTrue(evaluate_condition(guard, {'body': bodies}))
        for name, limit in [('Read_receipts_COV', 5000), ('Read_catalog_COV', 5000), ('Read_first_month_COV', 120), ('Read_coverage_COV', 5000)]:
            invalid = {**bodies, name: {'value': [None] * (limit+1)}}
            self.assertFalse(evaluate_condition(guard, {'body': invalid}))
        for name in ['Read_first_month_COV', 'Read_coverage_COV']:
            invalid = {**bodies, name: {'value': [], 'odata.nextLink': 'more-results'}}
            self.assertFalse(evaluate_condition(guard, {'body': invalid}))
        for count, accepted in [(0, False), (1, True), (120, True), (121, False)]:
            self.assertEqual(evaluate_condition(WORK['Bound_months_COV']['expression'], {'outputs': {'Month_count_COV': count}}), accepted)


if __name__ == '__main__':
    unittest.main()
