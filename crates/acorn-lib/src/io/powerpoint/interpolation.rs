//! Fallible, event-based interpolation of PowerPoint XML parts.
use super::ooxml;
use crate::io::ApiResult;
use acorn_core::util::interpolation::{ReplacementPlan, ReplacementValue};
use color_eyre::eyre::eyre;
use quick_xml::{
    events::{BytesText, Event},
    Reader, Writer,
};

/// Interpolate scalar and list placeholders without rewriting unrelated XML events.
pub fn interpolate_xml(xml: &str, plan: &ReplacementPlan) -> ApiResult<String> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::new());
    let parsed = loop {
        let result = match reader.read_event() {
            | Ok(Event::Start(start)) if start.name().local_name().as_ref() == b"p" => {
                paragraph_events(&mut reader, Event::Start(start.into_owned())).and_then(|events| write_paragraph(&mut writer, events, plan))
            }
            | Ok(Event::Eof) => break Ok(()),
            | Ok(event) => write_events(&mut writer, vec![event.into_owned()]),
            | Err(why) => Err(eyre!("Failed to parse OOXML part — {why}")),
        };
        if let Err(why) = result {
            break Err(why);
        }
    };
    parsed
        .and_then(|()| String::from_utf8(writer.into_inner()).map_err(|why| eyre!("Interpolated OOXML is not UTF-8 — {why}")))
        .and_then(|output| {
            ooxml::validate_xml_well_formed(&output)
                .then_some(output)
                .ok_or_else(|| eyre!("Interpolated OOXML is malformed"))
        })
}
fn list_name(chars: &[char], plan: &ReplacementPlan) -> ApiResult<Option<String>> {
    let mut found: Option<String> = None;
    let mut index = 0_usize;
    loop {
        if index.saturating_add(1) >= chars.len() {
            break Ok(found);
        }
        if chars.get(index) == Some(&'{') && chars.get(index.saturating_add(1)) == Some(&'{') {
            let start = index.saturating_add(2);
            let end = (start..chars.len().saturating_sub(1))
                .find(|&offset| chars.get(offset) == Some(&'}') && chars.get(offset.saturating_add(1)) == Some(&'}'));
            if let Some(end) = end {
                let name: String = chars.get(start..end).unwrap_or_default().iter().collect::<String>().trim().to_owned();
                if matches!(plan.get(&name), Some(ReplacementValue::List(_))) {
                    match &found {
                        | Some(previous) if previous != &name => break Err(eyre!("OOXML paragraph contains multiple list placeholders")),
                        | _ => found = Some(name),
                    }
                }
                index = end.saturating_add(2);
                continue;
            }
        }
        index = index.saturating_add(1);
    }
}
fn paragraph_events(reader: &mut Reader<&[u8]>, start: Event<'static>) -> ApiResult<Vec<Event<'static>>> {
    let mut events = vec![start];
    let mut depth = 1_usize;
    loop {
        match reader.read_event() {
            | Ok(Event::Eof) => break Err(eyre!("OOXML part ended inside a paragraph")),
            | Ok(event) => {
                match &event {
                    | Event::Start(_) => depth = depth.saturating_add(1),
                    | Event::End(_) => depth = depth.saturating_sub(1),
                    | _ => {}
                }
                events.push(event.into_owned());
                if depth == 0 {
                    break Ok(events);
                }
            }
            | Err(why) => break Err(eyre!("Failed to parse OOXML paragraph — {why}")),
        }
    }
}
fn render_paragraph(events: &[Event<'static>], plan: &ReplacementPlan, item: Option<(&str, &str)>) -> ApiResult<Vec<Event<'static>>> {
    text_nodes(events).and_then(|nodes| {
        let owned = nodes
            .iter()
            .enumerate()
            .flat_map(|(node, (_, value))| value.chars().map(move |character| (node, character)))
            .collect::<Vec<_>>();
        let mut output = vec![String::new(); nodes.len()];
        let mut index = 0_usize;
        let replaced: ApiResult<()> = loop {
            let Some((node, character)) = owned.get(index).copied() else {
                break Ok(());
            };
            let token = token_at(&owned, index);
            let replacement = token.as_ref().and_then(|(_, name)| match plan.get(name) {
                | Some(ReplacementValue::Absent) => Some(""),
                | Some(ReplacementValue::Scalar(value)) => Some(value.as_str()),
                | Some(ReplacementValue::List(_)) => item.and_then(|(selected, value)| (selected == name).then_some(value)),
                | None => None,
            });
            match output.get_mut(node) {
                | Some(destination) => match (token, replacement) {
                    | (Some((end, _)), Some(value)) => {
                        destination.push_str(value);
                        index = end.saturating_add(2);
                    }
                    | _ => {
                        destination.push(character);
                        index = index.saturating_add(1);
                    }
                },
                | None => break Err(eyre!("OOXML text node index is out of bounds")),
            }
        };
        replaced.and_then(|()| {
            let mut rendered = events.to_vec();
            nodes
                .iter()
                .zip(output)
                .try_for_each(|((event_index, original), replacement)| {
                    if original == &replacement {
                        Ok(())
                    } else {
                        rendered
                            .get_mut(*event_index)
                            .map(|event| *event = Event::Text(BytesText::new(&replacement).into_owned()))
                            .ok_or_else(|| eyre!("OOXML event index is out of bounds"))
                    }
                })
                .map(|()| rendered)
        })
    })
}
fn text_nodes(events: &[Event<'static>]) -> ApiResult<Vec<(usize, String)>> {
    let mut in_text = false;
    let mut nodes = Vec::new();
    let result = events.iter().enumerate().try_for_each(|(index, event)| match event {
        | Event::Start(start) if start.name().local_name().as_ref() == b"t" => {
            in_text = true;
            Ok(())
        }
        | Event::End(end) if end.name().local_name().as_ref() == b"t" => {
            in_text = false;
            Ok(())
        }
        | Event::Text(text) if in_text => text
            .decode()
            .map_err(|why| eyre!("Failed to decode OOXML text — {why}"))
            .and_then(|decoded| {
                quick_xml::escape::unescape(&decoded)
                    .map(|value| value.into_owned())
                    .map_err(|why| eyre!("Failed to unescape OOXML text — {why}"))
            })
            .map(|value| nodes.push((index, value))),
        | _ => Ok(()),
    });
    result.map(|()| nodes)
}
fn token_at(owned: &[(usize, char)], index: usize) -> Option<(usize, String)> {
    let starts_token = owned.get(index).map(|(_, character)| character) == Some(&'{')
        && owned.get(index.saturating_add(1)).map(|(_, character)| character) == Some(&'{');
    starts_token
        .then(|| {
            let start = index.saturating_add(2);
            (start..owned.len().saturating_sub(1))
                .find(|&offset| {
                    owned.get(offset).map(|(_, character)| character) == Some(&'}')
                        && owned.get(offset.saturating_add(1)).map(|(_, character)| character) == Some(&'}')
                })
                .map(|end| {
                    let name = owned
                        .get(start..end)
                        .unwrap_or_default()
                        .iter()
                        .map(|(_, character)| *character)
                        .collect::<String>();
                    (end, name.trim().to_owned())
                })
        })
        .flatten()
}
fn write_events(writer: &mut Writer<Vec<u8>>, events: Vec<Event<'static>>) -> ApiResult<()> {
    events.into_iter().try_for_each(|event| {
        writer
            .write_event(event)
            .map_err(|why| eyre!("Failed to serialize OOXML interpolation — {why}"))
    })
}
fn write_paragraph(writer: &mut Writer<Vec<u8>>, events: Vec<Event<'static>>, plan: &ReplacementPlan) -> ApiResult<()> {
    text_nodes(&events)
        .map(|nodes| {
            nodes
                .into_iter()
                .flat_map(|(_, value)| value.chars().collect::<Vec<_>>())
                .collect::<Vec<_>>()
        })
        .and_then(|chars| list_name(&chars, plan))
        .and_then(|name| match name {
            | Some(name) => plan
                .list(&name)
                .iter()
                .try_for_each(|value| render_paragraph(&events, plan, Some((&name, value))).and_then(|rendered| write_events(writer, rendered))),
            | None => render_paragraph(&events, plan, None).and_then(|rendered| write_events(writer, rendered)),
        })
}
