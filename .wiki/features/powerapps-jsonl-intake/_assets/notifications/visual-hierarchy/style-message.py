"""Apply presentation-only HTML hierarchy without changing workflow expressions."""
import re


def style_message(message):
    """Keep content, expressions and links intact; add semantic and CSS hierarchy."""
    message = message.replace('<p>ACORN // Sponsor Highlights</p>', '<p style="color:#526A70;font-size:12px;letter-spacing:1px"><strong>ACORN // SPONSOR HIGHLIGHTS</strong></p>')
    message = message.replace('<h2>', '<h2 style="color:#00785B;font-size:22px;margin:8px 0 16px">')
    message = re.sub(r'<p><strong>(Project|Destination|Status|Receipt ID|Correction receipt|Replaces receipt)</strong><br>(.*?)</p>', r'<blockquote style="border-left:4px solid #00785B;margin:12px 0;padding:4px 0 4px 16px"><strong>\1</strong><br>\2</blockquote>', message, flags=re.S)
    message = message.replace('<p><em>', '<hr><p style="color:#526A70;font-size:12px"><em>')
    message = message.replace('<p><strong>TEST ONLY</strong></p>', '<p style="color:#8A4B08"><strong>TEST ONLY // Synthetic validation</strong></p>')
    return message
