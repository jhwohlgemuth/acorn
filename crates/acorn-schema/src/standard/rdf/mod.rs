//! RDF concrete-syntax codecs shared by metadata standards.
use super::dublin_core::{
    DublinCoreDescription, DublinCoreDocument, DublinCoreLiteral, DublinCoreResource, DublinCoreValue, DCAM_NAMESPACE, DCTERMS_NAMESPACE,
    ELEMENTS_NAMESPACE, TYPE_NAMESPACE,
};
use crate::codec::CodecDiagnostic;
use crate::util::constants::app::{RDF_TYPE, XSD_STRING};
use crate::validation::rules;
use acorn_core::prelude::{format, vec, String, ToString, Vec};
use acorn_core::util::{escape_ntriples_literal, MimeType};
use alloc::collections::BTreeMap;
use core::fmt;
use oxixml_model::Triple as OxiTriple;
use oxixml_rdfxml::{RdfXmlParser as OxiRdfXmlParser, RdfXmlSerializer as OxiRdfXmlSerializer};
use serde_json::{Map, Value};
use sophia::api::prelude::*;
use sophia::api::source::StreamError::{SinkError, SourceError};
use sophia::api::term::TermKind;
use sophia::c14n::rdfc10;
use sophia::inmem::dataset::FastDataset;
use sophia::inmem::graph::FastGraph;
use sophia::turtle::{
    parser::{nq::NQuadsParser, nt::NTriplesParser, turtle::TurtleParser},
    serializer::{
        nt::NtSerializer,
        turtle::{TurtleConfig, TurtleSerializer},
    },
};

#[derive(Clone, Debug, Default)]
struct Context {
    terms: BTreeMap<String, String>,
    vocab: Option<String>,
}
/// One RDF statement in N-Triples term syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NTriple {
    /// Subject term, already written in N-Triples syntax.
    subject: String,
    /// Predicate term, already written in N-Triples syntax.
    predicate: String,
    /// Object term, already written in N-Triples syntax.
    object: String,
}
impl DublinCoreDocument {
    fn encode_json_ld(&self) -> Result<String, String> {
        let nodes = self
            .descriptions
            .iter()
            .map(|description| {
                let mut node = Map::new();
                node.insert("@id".to_string(), Value::String(description.subject.resource_id()));
                if !description.types.is_empty() {
                    node.insert(
                        "@type".to_string(),
                        Value::Array(description.types.iter().cloned().map(Value::String).collect()),
                    );
                }
                description.properties.iter().for_each(|(predicate, values)| {
                    node.insert(
                        predicate.clone(),
                        Value::Array(values.iter().map(DublinCoreValue::json_ld_value).collect()),
                    );
                });
                Value::Object(node)
            })
            .collect();
        serde_json::to_string_pretty(&Value::Array(nodes)).map_err(|error| error.to_string())
    }
    fn encode_rdf_xml(&self) -> Result<String, String> {
        canonical_ntriples_from_document(self).and_then(|triples| {
            OxiRdfXmlSerializer::new()
                .with_prefix("dc", ELEMENTS_NAMESPACE)
                .and_then(|serializer| serializer.with_prefix("dcam", DCAM_NAMESPACE))
                .and_then(|serializer| serializer.with_prefix("dcmitype", TYPE_NAMESPACE))
                .and_then(|serializer| serializer.with_prefix("dcterms", DCTERMS_NAMESPACE))
                .map_err(|error| error.to_string())
                .and_then(|serializer| {
                    let mut serializer = serializer.for_writer(Vec::new());
                    triples
                        .lines()
                        .filter(|line| !line.is_empty())
                        .map(|line| line.parse::<OxiTriple>().map_err(|error| error.to_string()))
                        .try_for_each(|triple| triple.and_then(|triple| serializer.serialize_triple(&triple).map_err(|error| error.to_string())))
                        .and_then(|()| serializer.finish().map_err(|error| error.to_string()))
                        .and_then(|bytes| String::from_utf8(bytes).map_err(|error| error.to_string()))
                })
        })
    }
}
impl DublinCoreResource {
    fn resource_id(&self) -> String {
        match self {
            | Self::BlankNode(value) => format!("_:{value}"),
            | Self::Iri(value) => value.clone(),
        }
    }
}
impl DublinCoreValue {
    fn json_ld_value(&self) -> Value {
        let mut object = Map::new();
        match self {
            | Self::Resource(value) => {
                object.insert("@id".to_string(), Value::String(value.resource_id()));
            }
            | Self::Literal(value) => {
                object.insert("@value".to_string(), Value::String(value.value.clone()));
                if let Some(language) = &value.language {
                    object.insert("@language".to_string(), Value::String(language.clone()));
                }
                if let Some(datatype) = &value.datatype {
                    object.insert("@type".to_string(), Value::String(datatype.clone()));
                }
            }
        }
        Value::Object(object)
    }
}
impl fmt::Display for NTriple {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {} .", self.subject, self.predicate, self.object)
    }
}
impl From<&DublinCoreResource> for String {
    fn from(resource: &DublinCoreResource) -> Self {
        match resource {
            | DublinCoreResource::BlankNode(value) => format!("_:{value}"),
            | DublinCoreResource::Iri(value) => format!("<{value}>"),
        }
    }
}
impl From<&DublinCoreValue> for String {
    fn from(value: &DublinCoreValue) -> Self {
        match value {
            | DublinCoreValue::Resource(resource) => String::from(resource),
            | DublinCoreValue::Literal(literal) => {
                let lexical = escape_ntriples_literal(&literal.value);
                match (&literal.language, &literal.datatype) {
                    | (Some(language), _) => format!("\"{lexical}\"@{language}"),
                    | (_, Some(datatype)) => format!("\"{lexical}\"^^<{datatype}>"),
                    | _ => format!("\"{lexical}\""),
                }
            }
        }
    }
}
impl From<DublinCoreDocument> for Vec<NTriple> {
    fn from(document: DublinCoreDocument) -> Self {
        document
            .normalized()
            .descriptions
            .into_iter()
            .flat_map(|description| {
                let subject = String::from(&description.subject);
                let types = description.types.into_iter().map({
                    let subject = subject.clone();
                    move |kind| NTriple {
                        object: format!("<{kind}>"),
                        predicate: format!("<{RDF_TYPE}>"),
                        subject: subject.clone(),
                    }
                });
                let properties = description.properties.into_iter().flat_map({
                    let subject = subject.clone();
                    move |(predicate, values)| {
                        let subject = subject.clone();
                        values.into_iter().map(move |value| NTriple {
                            object: String::from(&value),
                            predicate: format!("<{predicate}>"),
                            subject: subject.clone(),
                        })
                    }
                });
                types.chain(properties)
            })
            .collect()
    }
}
fn absolute_iri(value: &str) -> Result<String, String> {
    rules::url(value)
        .map(|()| value.to_string())
        .map_err(|_| format!("Provide an absolute IRI instead of '{value}'"))
}
fn add_triple<S: Term, P: Term, O: Term>(
    mut descriptions: BTreeMap<DublinCoreResource, DublinCoreDescription>,
    subject: S,
    predicate: P,
    object: O,
) -> Result<BTreeMap<DublinCoreResource, DublinCoreDescription>, String> {
    resource_from_term(subject).and_then(|subject| {
        predicate
            .iri()
            .map(|value| value.to_string())
            .ok_or_else(|| "RDF predicates must be absolute IRIs".to_string())
            .and_then(|predicate| {
                let description = descriptions.entry(subject.clone()).or_insert_with(|| DublinCoreDescription {
                    properties: BTreeMap::new(),
                    subject,
                    types: Vec::new(),
                });
                if predicate == RDF_TYPE {
                    resource_from_term(object).and_then(|resource| {
                        resource.as_iri().map_or_else(
                            || Err("RDF type values must be absolute class IRIs".to_string()),
                            |value| {
                                description.types.push(value.to_string());
                                Ok(())
                            },
                        )
                    })
                } else {
                    value_from_term(object).map(|value| description.properties.entry(predicate).or_default().push(value))
                }
            })
            .map(|()| descriptions)
    })
}
fn canonical_document(graph: FastGraph) -> Result<DublinCoreDocument, String> {
    canonical_ntriples(&graph)
        .and_then(|triples| NTriplesParser {}.parse_str(&triples).collect_triples::<FastGraph>().map_err(stream_error))
        .and_then(document_from_graph)
}
fn canonical_ntriples(graph: &FastGraph) -> Result<String, String> {
    let mut serializer = NtSerializer::new_stringifier();
    serializer
        .serialize_triples(graph.triples())
        .map_err(stream_error)
        .map(|_| ())
        .map(|()| serializer.to_string())
        .and_then(|triples| NQuadsParser {}.parse_str(&triples).collect_quads::<FastDataset>().map_err(stream_error))
        .and_then(|dataset| {
            let mut output = Vec::new();
            rdfc10::normalize(&dataset, &mut output)
                .map_err(|error| error.to_string())
                .and_then(|()| String::from_utf8(output).map_err(|error| error.to_string()))
        })
}
fn canonical_ntriples_from_document(document: &DublinCoreDocument) -> Result<String, String> {
    graph_from_document(document).and_then(|graph| canonical_ntriples(&graph))
}
fn canonicalize_document(document: &DublinCoreDocument) -> Result<DublinCoreDocument, String> {
    graph_from_document(document).and_then(canonical_document)
}
pub(super) fn decode(content: &str, format: &MimeType) -> Result<DublinCoreDocument, Vec<CodecDiagnostic>> {
    let decoded = match format {
        | MimeType::LdJson => decode_json_ld(content),
        | MimeType::RdfXml => decode_rdf_xml(content),
        | MimeType::Turtle => TurtleParser { base: None }
            .parse_str(content)
            .collect_triples::<FastGraph>()
            .map_err(stream_error)
            .and_then(canonical_document),
        | _ => Err(format!("Unsupported Dublin Core RDF format '{format}'")),
    };
    decoded.map_err(|message| vec![CodecDiagnostic::deserialize(message)])
}
fn decode_json_ld(content: &str) -> Result<DublinCoreDocument, String> {
    serde_json::from_str(content)
        .map_err(|error| error.to_string())
        .and_then(|value| json_ld_document(&value))
        .and_then(|document| canonicalize_document(&document))
}
fn decode_rdf_xml(content: &str) -> Result<DublinCoreDocument, String> {
    OxiRdfXmlParser::new()
        .for_slice(content)
        .map(|triple| triple.map(|triple| format!("{triple} .\n")).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map(|triples| triples.concat())
        .and_then(|triples| NTriplesParser {}.parse_str(&triples).collect_triples::<FastGraph>().map_err(stream_error))
        .and_then(canonical_document)
}
fn document_from_graph(graph: FastGraph) -> Result<DublinCoreDocument, String> {
    graph
        .triples()
        .try_fold(BTreeMap::<DublinCoreResource, DublinCoreDescription>::new(), |descriptions, triple| {
            triple
                .map_err(|error| error.to_string())
                .and_then(|triple| add_triple(descriptions, triple.s(), triple.p(), triple.o()))
        })
        .map(|descriptions| DublinCoreDocument::new(descriptions.into_values().collect()))
}
pub(super) fn encode(document: &DublinCoreDocument, format: &MimeType) -> Result<String, CodecDiagnostic> {
    canonicalize_document(document)
        .and_then(|document| match format {
            | MimeType::LdJson => document.encode_json_ld(),
            | MimeType::RdfXml => document.encode_rdf_xml(),
            | MimeType::Turtle => canonical_ntriples_from_document(&document).and_then(|triples| {
                let mut serializer = TurtleSerializer::new_stringifier_with_config(TurtleConfig::new().with_pretty(true));
                serializer
                    .serialize_triples(NTriplesParser {}.parse_str(&triples))
                    .map_err(stream_error)
                    .map(|_| ())
                    .map(|()| serializer.to_string())
            }),
            | _ => Err(format!("Unsupported Dublin Core RDF format '{format}'")),
        })
        .map_err(CodecDiagnostic::serialize)
}
fn expand_iri(value: &str, context: &Context) -> Result<String, String> {
    expand_iri_depth(value, context, 0)
}
fn expand_iri_depth(value: &str, context: &Context, depth: usize) -> Result<String, String> {
    if depth > context.terms.len() {
        Err(format!("Cyclic JSON-LD context definition for '{value}'"))
    } else if let Some(mapped) = context.terms.get(value) {
        expand_iri_depth(mapped, context, depth.saturating_add(1))
    } else if let Some((prefix, suffix)) = value.split_once(':') {
        context
            .terms
            .get(prefix)
            .map_or_else(|| absolute_iri(value), |namespace| absolute_iri(&format!("{namespace}{suffix}")))
    } else {
        context.vocab.as_ref().map_or_else(
            || Err(format!("Unresolved relative JSON-LD IRI '{value}'")),
            |vocab| absolute_iri(&format!("{vocab}{value}")),
        )
    }
}
fn graph_from_document(document: &DublinCoreDocument) -> Result<FastGraph, String> {
    let triples = Vec::<NTriple>::from(document.clone())
        .into_iter()
        .map(|triple| format!("{triple}\n"))
        .collect::<String>();
    NTriplesParser {}.parse_str(&triples).collect_triples::<FastGraph>().map_err(stream_error)
}
fn json_ld_document(value: &Value) -> Result<DublinCoreDocument, String> {
    match value {
        | Value::Array(nodes) => parse_nodes(nodes, &Context::default()),
        | Value::Object(object) => parse_context(object.get("@context"), &Context::default()).and_then(|context| match object.get("@graph") {
            | Some(Value::Array(nodes)) => parse_nodes(nodes, &context),
            | Some(node @ Value::Object(_)) => parse_nodes(core::slice::from_ref(node), &context),
            | Some(_) => Err("JSON-LD @graph must be an object or array".to_string()),
            | None => parse_nodes(core::slice::from_ref(value), &context),
        }),
        | _ => Err("JSON-LD must contain a node object or array".to_string()),
    }
}
fn lexical_value(value: &Value) -> String {
    match value {
        | Value::String(value) => value.clone(),
        | value => value.to_string(),
    }
}
fn parse_context(value: Option<&Value>, parent: &Context) -> Result<Context, String> {
    match value {
        | None | Some(Value::Null) => Ok(parent.clone()),
        | Some(Value::String(_)) => Err("Remote JSON-LD contexts are not supported".to_string()),
        | Some(Value::Array(values)) => values
            .iter()
            .try_fold(parent.clone(), |context, value| parse_context(Some(value), &context)),
        | Some(Value::Object(values)) => values.iter().try_fold(parent.clone(), |mut context, (term, definition)| {
            if term == "@vocab" {
                definition.as_str().map_or_else(
                    || Err("JSON-LD @vocab must be an absolute IRI".to_string()),
                    |value| {
                        context.vocab = Some(value.to_string());
                        Ok(context)
                    },
                )
            } else {
                let mapping = definition
                    .as_str()
                    .or_else(|| definition.as_object().and_then(|value| value.get("@id")).and_then(Value::as_str));
                mapping.map_or_else(
                    || Err(format!("Unsupported JSON-LD context definition for '{term}'")),
                    |mapping| {
                        context.terms.insert(term.clone(), mapping.to_string());
                        Ok(context)
                    },
                )
            }
        }),
        | Some(_) => Err("JSON-LD @context must be an inline object or array".to_string()),
    }
}
fn parse_node(value: &Value, index: usize, parent: &Context) -> Result<DublinCoreDescription, String> {
    value.as_object().map_or_else(
        || Err("Each JSON-LD graph node must be an object".to_string()),
        |object| {
            parse_context(object.get("@context"), parent).and_then(|context| {
                let subject = object
                    .get("@id")
                    .and_then(Value::as_str)
                    .map_or_else(|| Ok(DublinCoreResource::blank_node(format!("generated{index}"))), parse_resource);
                subject.and_then(|subject| {
                    parse_types(object.get("@type"), &context).and_then(|types| {
                        object
                            .iter()
                            .filter(|(key, _)| !key.starts_with('@'))
                            .try_fold(BTreeMap::new(), |mut properties, (key, value)| {
                                expand_iri(key, &context).and_then(|predicate| {
                                    parse_values(value, &context).map(|values| {
                                        properties.entry(predicate).or_insert_with(Vec::new).extend(values);
                                        properties
                                    })
                                })
                            })
                            .map(|properties| DublinCoreDescription { properties, subject, types })
                    })
                })
            })
        },
    )
}
fn parse_nodes(nodes: &[Value], parent: &Context) -> Result<DublinCoreDocument, String> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| parse_node(node, index, parent))
        .collect::<Result<Vec<_>, _>>()
        .map(DublinCoreDocument::new)
}
fn parse_resource(value: &str) -> Result<DublinCoreResource, String> {
    value.strip_prefix("_:").map_or_else(
        || absolute_iri(value).map(DublinCoreResource::iri),
        |value| Ok(DublinCoreResource::blank_node(value)),
    )
}
fn parse_types(value: Option<&Value>, context: &Context) -> Result<Vec<String>, String> {
    match value {
        | None => Ok(Vec::new()),
        | Some(Value::String(value)) => expand_iri(value, context).map(|value| vec![value]),
        | Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| "JSON-LD @type values must be strings".to_string())
                    .and_then(|value| expand_iri(value, context))
            })
            .collect(),
        | _ => Err("JSON-LD @type must be a string or array".to_string()),
    }
}
fn parse_value(value: &Value, context: &Context) -> Result<DublinCoreValue, String> {
    match value {
        | Value::Object(object) if object.contains_key("@id") => object
            .get("@id")
            .and_then(Value::as_str)
            .ok_or_else(|| "JSON-LD @id must be a string".to_string())
            .and_then(parse_resource)
            .map(DublinCoreValue::Resource),
        | Value::Object(object) if object.contains_key("@value") => {
            let lexical = object.get("@value").map(lexical_value).unwrap_or_default();
            let language = object.get("@language").and_then(Value::as_str).map(ToString::to_string);
            object
                .get("@type")
                .and_then(Value::as_str)
                .map(|value| expand_iri(value, context))
                .transpose()
                .map(|datatype| {
                    DublinCoreValue::Literal(DublinCoreLiteral {
                        datatype,
                        language,
                        value: lexical,
                    })
                })
        }
        | Value::Object(_) => Err("JSON-LD values must contain @id or @value".to_string()),
        | value => Ok(DublinCoreValue::Literal(DublinCoreLiteral {
            datatype: None,
            language: None,
            value: lexical_value(value),
        })),
    }
}
fn parse_values(value: &Value, context: &Context) -> Result<Vec<DublinCoreValue>, String> {
    match value {
        | Value::Array(values) => values.iter().map(|value| parse_value(value, context)).collect(),
        | value => parse_value(value, context).map(|value| vec![value]),
    }
}
fn resource_from_term<T: Term>(term: T) -> Result<DublinCoreResource, String> {
    match term.kind() {
        | TermKind::BlankNode => term
            .bnode_id()
            .map(|value| DublinCoreResource::blank_node(value.to_string()))
            .ok_or_else(|| "Invalid RDF blank node".to_string()),
        | TermKind::Iri => term
            .iri()
            .map(|value| DublinCoreResource::iri(value.to_string()))
            .ok_or_else(|| "Invalid RDF IRI".to_string()),
        | TermKind::Triple => Err("RDF-star quoted triples are not supported".to_string()),
        | _ => Err("RDF resources must be IRIs or blank nodes".to_string()),
    }
}
fn stream_error<S, T>(error: sophia::api::source::StreamError<S, T>) -> String
where
    S: core::error::Error,
    T: core::error::Error,
{
    match error {
        | SinkError(error) => error.to_string(),
        | SourceError(error) => error.to_string(),
    }
}
fn value_from_term<T: Term>(term: T) -> Result<DublinCoreValue, String> {
    match term.kind() {
        | TermKind::BlankNode | TermKind::Iri => resource_from_term(term).map(DublinCoreValue::Resource),
        | TermKind::Literal => term.lexical_form().map_or_else(
            || Err("Invalid RDF literal".to_string()),
            |value| {
                let language = term.language_tag().map(|value| value.to_string());
                let datatype = language.is_none().then(|| term.datatype().map(|value| value.to_string())).flatten();
                Ok(DublinCoreValue::Literal(DublinCoreLiteral {
                    datatype: datatype.filter(|value| value != XSD_STRING),
                    language,
                    value: value.to_string(),
                }))
            },
        ),
        | TermKind::Triple => Err("RDF-star quoted triples are not supported".to_string()),
        | TermKind::Variable => Err("RDF variables are not supported".to_string()),
    }
}
