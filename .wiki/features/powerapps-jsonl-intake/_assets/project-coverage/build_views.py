"""Build SharePoint-native coverage cards and their view-configuration scope."""
import json
from build_dashboard import COVERAGE, MONTHS, ROOT, SITE, http, list_uri, package


def element(tag, text=None, style=None, children=None, attributes=None):
    result = {'elmType': tag}
    if text is not None:
        result['txtContent'] = text
    if style:
        result['style'] = style
    if children:
        result['children'] = children
    if attributes:
        result['attributes'] = attributes
    return result


def month_formatter():
    stale = '=if([$IsCurrentMonth] && Number(@now)-Number([$RefreshedAt]) > 900000,\'Refresh overdue — last successful refresh: \',\'Refreshed: \') + toLocaleString([$RefreshedAt])'
    body = element('div', style={'display': 'flex', 'flex-direction': 'column', 'align-items': 'stretch', 'text-align': 'left', 'padding': '22px', 'margin': '8px 0', 'border-radius': '12px', 'background-color': '#F2F7F5', 'border-left': '5px solid #00785B', 'width': '100%', 'box-sizing': 'border-box'}, children=[
        element('div', '[$Title]', {'font-size': '24px', 'font-weight': '700', 'color': '#004C54'}),
        element('div', "=if([$ProjectsExpected] == 0, 'No expected projects configured', if([$ProjectsSubmitted] == [$ProjectsExpected], 'All projects covered', toString([$ProjectsExpected]-[$ProjectsSubmitted]) + ' projects awaiting highlights'))", {'font-size': '18px', 'font-weight': '600', 'margin-top': '8px', 'color': '#183C43'}),
        element('div', "=toString([$ProjectsSubmitted]) + ' of ' + toString([$ProjectsExpected]) + ' projects submitted'", {'margin-top': '5px'}),
        element('div', style={'width': '100%', 'height': '12px', 'background-color': '#DCE5E2', 'border-radius': '6px', 'margin-top': '12px'}, children=[element('div', style={'height': '12px', 'background-color': '#00785B', 'border-radius': '6px', 'width': "=if([$ProjectsExpected] == 0, '0%', toString(100*[$ProjectsSubmitted]/[$ProjectsExpected]) + '%')"})]),
        element('div', "=toString([$HighlightCount]) + ' highlights · ' + toString([$GeneralCount]) + ' report-wide updates included'", {'margin-top': '12px', 'color': '#405D63'}),
        element('a', 'View projects for this month →', {'display': 'inline-block', 'margin-top': '12px', 'font-weight': '600', 'color': '#00674F'}, attributes={'href': "='"+SITE+"/Lists/Sponsor%20Highlights%20Coverage/AllItems.aspx?FilterField1=MonthKey&FilterValue1=' + [$Title]", 'target': '_self'}),
        submission_links(),
        element('div', stale, {'font-size': '12px', 'margin-top': '12px', 'color': "=if([$IsCurrentMonth] && Number(@now)-Number([$RefreshedAt]) > 900000, '#9A3412', '#526A70')"}),
        element('div', '[$RosterBasis]', {'font-size': '12px', 'color': '#526A70', 'margin-top': '4px'})])
    return {'$schema': 'https://developer.microsoft.com/json-schemas/sp/v2/row-formatting.schema.json', 'hideSelection': True, 'hideColumnHeader': True, 'rowFormatter': body}


def project_formatter():
    return {'$schema': 'https://developer.microsoft.com/json-schemas/sp/v2/row-formatting.schema.json', 'hideSelection': True, 'hideColumnHeader': True, 'rowFormatter': element('div', style={'display': 'flex', 'flex-wrap': 'wrap', 'align-items': 'center', 'justify-content': 'space-between', 'padding': '16px 20px', 'margin': '5px 0', 'border-radius': '10px', 'background-color': "=if([$HighlightCount] > 0, '#EDF6F0', '#FFF7E7')", 'width': '100%', 'box-sizing': 'border-box'}, children=[
        element('div', style={'display': 'flex', 'flex-direction': 'column', 'min-width': '180px', 'flex-grow': '1'}, children=[element('div', '[$Title]', {'font-size': '18px', 'font-weight': '600', 'color': '#183C43'}), element('div', '[$MonthKey]', {'font-size': '12px', 'color': '#526A70', 'margin-top': '4px'})]),
        element('div', "=if([$HighlightCount] > 0, '✓ Submitted · ' + toString([$HighlightCount]), 'Awaiting highlight')", {'font-size': '15px', 'font-weight': '600', 'color': "=if([$HighlightCount] > 0, '#00662C', '#805A0B')"}),
        submission_links(),
        element('div', "='Updated: ' + toLocaleString([$RefreshedAt])", {'width': '100%', 'font-size': '12px', 'margin-top': '7px', 'color': '#526A70'})])}


def submission_links():
    # Only percent, record and ID separators are escaped; decode percent last.
    receipt_id = "substring([$receipt],0,indexOf([$receipt],'|'))"
    summary = "substring([$receipt],indexOf([$receipt],'|')+1,indexOf([$receipt]+'|END','|END'))"
    tooltip = "=replaceAll(replaceAll(replaceAll("+summary+",'%7C','|'),'%3B',';'),'%25','%')"
    link = element('a', "='Submission #' + "+receipt_id, {'color': '#00674F', 'font-size': '13px', 'font-weight': '600', 'padding': '5px 9px', 'margin': '3px 6px 3px 0', 'border': '1px solid #AEC8BF', 'border-radius': '6px', 'background-color': '#FFFFFF', 'text-decoration': 'none'}, attributes={'href': "='"+SITE+"/Lists/Sponsor%20Highlights%20Intake%20Pilot/DispForm.aspx?ID=' + "+receipt_id, 'target': '_blank', 'title': tooltip})
    link['forEach'] = "receipt in split([$ReceiptLinks],';')"
    return element('div', style={'display': "=if([$ReceiptLinks] == '', 'none', 'flex')", 'flex-direction': 'column', 'width': '100%', 'margin-top': '12px'}, children=[
        element('div', 'View submissions · reviewers', {'font-size': '12px', 'font-weight': '600', 'color': '#526A70', 'margin-bottom': '4px'}),
        element('div', style={'display': 'flex', 'flex-wrap': 'wrap', 'max-height': '140px', 'overflow-y': 'auto'}, children=[link]),
        element('div', 'Hover for the original summary. Opening the full receipt requires reviewer access.', {'font-size': '11px', 'color': '#526A70', 'margin-top': '4px'})])


def configure():
    actions = {}
    previous = None
    for prefix, title, fields, formatter, sort in [
        ('Months', MONTHS, ['Title', 'ProjectsExpected', 'ProjectsSubmitted', 'HighlightCount', 'GeneralCount', 'IsCurrentMonth', 'RefreshedAt', 'RosterBasis', 'ReceiptItemIds', 'ReceiptLinks'], month_formatter(), '<OrderBy><FieldRef Name="Title" Ascending="FALSE"/></OrderBy>'),
        ('Projects', COVERAGE, ['Title', 'MonthKey', 'ProjectId', 'HighlightCount', 'FirstReceived', 'LastReceived', 'RefreshedAt', 'ReceiptItemIds', 'ReceiptLinks'], project_formatter(), '<OrderBy><FieldRef Name="MonthKey" Ascending="FALSE"/><FieldRef Name="Title"/></OrderBy>')]:
        name = prefix+'_view'
        actions[name] = http('POST', list_uri(title)+'/DefaultView', {'CustomFormatter': json.dumps(formatter).replace('&', r'\u0026').replace('<', r'\u003c').replace('>', r'\u003e'), 'ViewQuery': sort, 'RowLimit': 100, 'Paged': True}, previous, {'IF-MATCH': '*', 'X-HTTP-Method': 'MERGE'})
        previous = name
        name = prefix+'_clear_fields'
        actions[name] = http('POST', list_uri(title)+'/DefaultView/ViewFields/RemoveAllViewFields', dependency=previous)
        previous = name
        for field in fields:
            name = prefix+'_field_'+field
            actions[name] = http('POST', list_uri(title)+f"/DefaultView/ViewFields/AddViewField('{field}')", dependency=previous)
            previous = name
    return package('Configure_coverage_views', actions)


if __name__ == '__main__':
    for name, value in [('months-format.json', month_formatter()), ('projects-format.json', project_formatter()), ('views.clipboard.json', configure())]:
        (ROOT/name).write_text(json.dumps(value, indent=2)+'\n')
