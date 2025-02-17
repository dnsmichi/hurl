/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2022 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use crate::ast::*;
use crate::parser::multiline::multiline_string;
use crate::parser::string::backtick_template;

use super::combinators::*;
use super::json::parse as parse_json;
use super::primitives::*;
use super::reader::Reader;
use super::xml;
use super::ParseResult;

pub fn bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    choice(
        &[
            multiline_string_bytes,
            string_bytes,
            json_bytes,
            xml_bytes,
            base64_bytes,
            hex_bytes,
            file_bytes,
        ],
        reader,
    )
}

fn xml_bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    match xml::parse(reader) {
        Err(e) => Err(e),
        Ok(value) => Ok(Bytes::Xml(value)),
    }
}

fn json_bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    match parse_json(reader) {
        Err(e) => Err(e),
        Ok(value) => Ok(Bytes::Json(value)),
    }
}

fn file_bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    file(reader).map(Bytes::File)
}

fn base64_bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    base64(reader).map(Bytes::Base64)
}

fn hex_bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    hex(reader).map(Bytes::Hex)
}

pub fn multiline_string_bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    multiline_string(reader).map(Bytes::MultilineString)
}

fn string_bytes(reader: &mut Reader) -> ParseResult<'static, Bytes> {
    backtick_template(reader).map(Bytes::OnelineString)
}

#[cfg(test)]
mod tests {
    use super::super::error::*;
    use super::*;

    #[test]
    fn test_bytes_json() {
        let mut reader = Reader::init("[1,2,3] ");
        assert_eq!(
            bytes(&mut reader).unwrap(),
            Bytes::Json(JsonValue::List {
                space0: "".to_string(),
                elements: vec![
                    JsonListElement {
                        space0: "".to_string(),
                        value: JsonValue::Number("1".to_string()),
                        space1: "".to_string(),
                    },
                    JsonListElement {
                        space0: "".to_string(),
                        value: JsonValue::Number("2".to_string()),
                        space1: "".to_string(),
                    },
                    JsonListElement {
                        space0: "".to_string(),
                        value: JsonValue::Number("3".to_string()),
                        space1: "".to_string(),
                    },
                ],
            })
        );
        assert_eq!(reader.state.cursor, 7);

        let mut reader = Reader::init("{ } ");
        assert_eq!(
            bytes(&mut reader).unwrap(),
            Bytes::Json(JsonValue::Object {
                space0: " ".to_string(),
                elements: vec![],
            })
        );
        assert_eq!(reader.state.cursor, 3);

        let mut reader = Reader::init("true");
        assert_eq!(
            bytes(&mut reader).unwrap(),
            Bytes::Json(JsonValue::Boolean(true))
        );
        assert_eq!(reader.state.cursor, 4);

        let mut reader = Reader::init("\"\" x");
        assert_eq!(
            bytes(&mut reader).unwrap(),
            Bytes::Json(JsonValue::String(Template {
                delimiter: Some('"'),
                elements: vec![],
                source_info: SourceInfo::new(1, 2, 1, 2),
            }))
        );
        assert_eq!(reader.state.cursor, 2);
    }

    #[test]
    fn test_bytes_xml() {
        let mut reader = Reader::init("<a/>");
        assert_eq!(
            bytes(&mut reader).unwrap(),
            Bytes::Xml(String::from("<a/>"))
        );
    }

    #[test]
    fn test_bytes_json_error() {
        let mut reader = Reader::init("{ x ");
        let error = bytes(&mut reader).err().unwrap();
        assert_eq!(error.pos, Pos { line: 1, column: 3 });
        assert_eq!(
            error.inner,
            ParseError::Expecting {
                value: "\"".to_string()
            }
        );
    }

    #[test]
    fn test_bytes_multilines_error() {
        let mut reader = Reader::init("```\nxxx ");
        let error = bytes(&mut reader).err().unwrap();
        assert_eq!(error.pos, Pos { line: 1, column: 4 });
        assert_eq!(error.inner, ParseError::Multiline);
    }

    #[test]
    fn test_bytes_eof() {
        let mut reader = Reader::init("");
        let error = bytes(&mut reader).err().unwrap();
        assert_eq!(
            error.inner,
            ParseError::Expecting {
                value: String::from("file")
            }
        );
        assert!(error.recoverable);
    }

    #[test]
    fn test_json_bytes() {
        let mut reader = Reader::init("100");
        assert_eq!(
            json_bytes(&mut reader).unwrap(),
            Bytes::Json(JsonValue::Number("100".to_string()))
        );
    }

    #[test]
    fn test_bytes_string() {
        let mut reader = Reader::init("`foo`  ");
        assert_eq!(
            bytes(&mut reader).unwrap(),
            Bytes::OnelineString(Template {
                delimiter: Some('`'),
                elements: vec![TemplateElement::String {
                    value: "foo".to_string(),
                    encoded: "foo".to_string()
                }],
                source_info: SourceInfo::new(1, 1, 1, 6)
            })
        );
        assert_eq!(reader.state.cursor, 5);
    }
}
