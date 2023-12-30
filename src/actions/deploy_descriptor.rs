use std::collections::HashMap;

use reqwest::{header::HeaderMap, Method, Url};
use thiserror::Error;

#[allow(dead_code)]
const DEPLOY_DESCRIPTOR_SCHEMA: &str = r#"
{
    "schemaDescription": {
        "name": "Deploy Descriptor",
        "description": "A descriptor for the deploy script",
        "version": "1.0.0"
    },
    "schema": {
        "type": "object",
        "properties": {
            "root_user": {
                "type": "string",
                "description": "The username of the root user on the target"
            },
            "root_pass": {
                "type": "string",
                "description": "The password of the root user on the target"
            },
            "path": {
                "type": "string",
                "description": "The unix path to the deploy directory on the target",
                "default": "~"
            },
            "stop_cmd": {
                "type": "string",
                "description": "The command to stop the robot-code on the target"
            },
            "start_cmd": {
                "type": "string",
                "description": "The command to start the robot-code on the target"
            },
            "dep_lib_path": {
                "type": "string",
                "description": "The path to the deploy libraries relative to the deploy directory on the target",
                "default": "./lib"
            },
            "extra_files_path": {
                "type": "string",
                "description": "The path to the extra files relative to the deploy directory on the target",
                "default": "./deploy"
            },
            "serial_getter": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The url to get the serial number from, will interpolate $ADDR$ with the address of the target"
                    },
                    "method": {
                        "type": "string",
                        "description": "The method to use to get the serial number from the target",
                        "default": "POST"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Additional headers to send with the request",
                        "default": null
                    },
                    "parser": {
                        "type": "object",
                        "properties": {
                            "format": {
                                "type": "json|xml|regex",
                                "description": "The format of the keys"
                            },
                            "paths": {
                                "type": "array",
                                "description": "The keys to the serial number in the response"
                            }
                        }
                    }
                }
            },
            "pinger": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The url to use, will interpolate $ADDR$ with the address of the target"
                    },
                    "method": {
                        "type": "string",
                        "description": "The method to use to ping the target",
                        "default": "GET"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Additional headers to send with the request",
                        "default": null
                    }
                }
            }
        }
    }
}"#;

#[derive(Debug, Error, Clone, Copy)]
pub enum DescriptorParseError {
    #[error("Failed to get attribute {name} from {file_name}")]
    MissingAttribute {
        name: &'static str,
        file_name: &'static str,
    },
    #[error("Failed to parse attribute {name} from {file_name}, should be {hint}")]
    ParseAttribute {
        name: &'static str,
        hint: &'static str,
        file_name: &'static str,
    },
}

#[derive(Debug)]
pub struct Descriptor {
    pub root_user: String,
    pub root_password: String,
    pub path: String,
    pub start_cmd: String,
    pub stop_cmd: String,
    pub dep_lib_path: String,
    pub serial_getter: SerialGetter,
    pub pinger: Pinger,
}

#[derive(Debug)]
pub enum ParserFormat {
    JSON,
    XML,
    REGEX,
}

#[derive(Debug)]
pub struct SerialGetter {
    pub url: Url,
    pub method: Method,
    pub format: ParserFormat,
    pub paths: Vec<String>,
    pub headers: HeaderMap,
}
impl SerialGetter {
    pub async fn call(&self, client: &reqwest::Client) -> Result<String, String> {
        let mut req = client.request(self.method.clone(), self.url.clone());
        for (key, value) in self.headers.iter() {
            req = req.header(key, value);
        }
        let res = req.send().await;
        match res {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    match self.format {
                        ParserFormat::JSON => {
                            let body = res.text().await;
                            match body {
                                Ok(body) => self.extract_json(body),
                                Err(err) => Err(format!("Failed to get body: {}", err)),
                            }
                        }
                        _ => Err(format!("Unsupported format: {:?}", self.format)),
                    }
                } else {
                    Err(format!("Failed to get body: {}", status))
                }
            }
            Err(err) => Err(format!("Failed to get body: {}", err)),
        }
    }

    pub fn extract_json(&self, body: String) -> Result<String, String> {
        let json = serde_json::from_str::<serde_json::Value>(&body);
        let paths = self
            .paths
            .iter()
            .map(|paths| paths.split('.').collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>();
        match json {
            Ok(json) => {
                for path in &paths {
                    let mut json = json.clone();
                    for attr in path {
                        if let Some(value) = json.get(attr) {
                            json = value.clone();
                        } else {
                            break;
                        }
                    }
                    if let serde_json::Value::String(serial) = json {
                        return Ok(serial);
                    }
                }
                Err(format!("Failed to find serial in json"))
            }
            Err(err) => Err(format!("Failed to parse json: {}", err)),
        }
    }
}

#[derive(Debug)]
pub struct Pinger {
    pub url: Url,
    pub method: Method,
    pub headers: HeaderMap,
}
impl Pinger {
    pub async fn call(&self, client: &reqwest::Client) -> Result<(), String> {
        let mut req = client.request(self.method.clone(), self.url.clone());
        for (key, value) in self.headers.iter() {
            req = req.header(key, value);
        }
        let res = req.send().await;
        match res {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    Ok(())
                } else {
                    Err(format!("Failed to ping: {}", status))
                }
            }
            Err(err) => Err(format!("Failed to ping: {}", err)),
        }
    }
}

macro_rules! get_attr {
    ($jval:expr, $attr:expr, $hint:expr, $file_name:expr, $as:ident) => {
        $jval
            .get($attr)
            .ok_or(DescriptorParseError::MissingAttribute {
                name: $attr,
                file_name: $file_name,
            })?
            .$as()
            .ok_or(DescriptorParseError::ParseAttribute {
                name: $attr,
                hint: $hint,
                file_name: $file_name,
            })?
            .to_owned()
    };
    ($jval:expr, $attr:expr, $hint:expr, $file_name:expr) => {
        $jval
            .get($attr)
            .ok_or(DescriptorParseError::MissingAttribute {
                name: $attr,
                file_name: $file_name,
            })?
            .to_owned()
    };
}

macro_rules! get_attr_default {
    ($jval:expr, $attr:expr, $hint:expr, $file_name:expr, $as:ident, $default:expr) => {
        $jval
            .get($attr)
            .unwrap_or(&serde_json::Value::from($default.to_owned()))
            .$as()
            .ok_or(DescriptorParseError::ParseAttribute {
                name: $attr,
                hint: $hint,
                file_name: $file_name,
            })?
            .to_owned()
    };
}

pub fn parse_descriptor(
    jval: serde_json::Value,
    file: &'static str,
) -> Result<Descriptor, DescriptorParseError> {
    let root_user = get_attr!(
        jval,
        "root_user",
        "the username of the root user on the target",
        file,
        as_str
    );
    let root_password = get_attr_default!(
        jval,
        "root_pass",
        "the password of the root user on the target",
        file,
        as_str,
        ""
    );
    let path = get_attr_default!(
        jval,
        "path",
        "the unix path to the deploy directory on the target",
        file,
        as_str,
        "~"
    );
    let start_cmd = get_attr!(
        jval,
        "start_cmd",
        "the command to start the robot-code on the target",
        file,
        as_str
    );
    let stop_cmd = get_attr!(
        jval,
        "stop_cmd",
        "the command to stop the robot-code on the target",
        file,
        as_str
    );
    let dep_lib_path = get_attr_default!(
        jval,
        "dep_lib_path",
        "the path to the deploy libraries relative to the deploy directory on the target",
        file,
        as_str,
        "./lib"
    );
    let serial_getter = parse_serial_getter(
        get_attr!(
            jval,
            "serial_getter",
            "the url to get the serial number from, will interpolate $ADDR$ with the address of the target",
            file
        ),
        file
    )?;
    let pinger = parse_pinger(
        get_attr!(
            jval,
            "pinger",
            "the url to use, will interpolate $ADDR$ with the address of the target",
            file
        ),
        file,
    )?;
    Ok(Descriptor {
        root_user,
        root_password,
        path,
        start_cmd,
        stop_cmd,
        dep_lib_path,
        serial_getter,
        pinger,
    })
}

pub fn parse_serial_getter(
    jval: serde_json::Value,
    file: &'static str,
) -> Result<SerialGetter, DescriptorParseError> {
    let url = get_attr!(
        jval,
        "url",
        "the url to get the serial number from, will interpolate $ADDR$ with the address of the target",
        file,
        as_str
    );
    let url = Url::parse(&url).map_err(|_| DescriptorParseError::ParseAttribute {
        name: "url",
        hint: "a string that can be parsed as a url",
        file_name: file,
    })?;
    let method = get_attr_default!(
        jval,
        "method",
        "the method to use to get the serial number from the target",
        file,
        as_str,
        "POST"
    );
    let method = match method.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        "CONNECT" => Method::CONNECT,
        "PATCH" => Method::PATCH,
        _ => {
            return Err(DescriptorParseError::ParseAttribute {
                name: "method",
                hint: "a string that can be parsed as a method",
                file_name: file,
            })
        }
    };
    let format = get_attr!(jval, "format", "the format of the keys", file, as_str);
    let format = match format.as_str() {
        "json" => ParserFormat::JSON,
        "xml" => ParserFormat::XML,
        "regex" => ParserFormat::REGEX,
        _ => {
            return Err(DescriptorParseError::ParseAttribute {
                name: "format",
                hint: "a string that can be parsed as a format [json, xml, regex]",
                file_name: file,
            })
        }
    };
    let paths = get_attr!(
        jval,
        "paths",
        "The keys to the serial number in the response",
        file,
        as_array
    )
    .iter()
    .map(|path| {
        path.as_str()
            .ok_or(DescriptorParseError::ParseAttribute {
                name: "paths",
                hint: "an array of strings",
                file_name: file,
            })
            .map(|s| s.to_owned())
    })
    .collect::<Result<Vec<String>, DescriptorParseError>>()?;
    let headers = get_attr_default!(
        jval,
        "headers",
        "Additional headers to send with the request",
        file,
        as_object,
        serde_json::Map::new()
    )
    .iter()
    .map(|(key, value)| {
        value
            .as_str()
            .ok_or(DescriptorParseError::ParseAttribute {
                name: "headers",
                hint: "an object of strings",
                file_name: file,
            })
            .map(|s| (key.to_owned(), s.to_owned()))
    })
    .collect::<Result<HashMap<String, String>, DescriptorParseError>>()?;
    let headers = headers
        .iter()
        .map(|(key, value)| {
            let mut header =
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|_| {
                    DescriptorParseError::ParseAttribute {
                        name: "headers",
                        hint: "an object of strings",
                        file_name: file,
                    }
                })?;
            let value = reqwest::header::HeaderValue::from_str(value).map_err(|_| {
                DescriptorParseError::ParseAttribute {
                    name: "headers",
                    hint: "an object of strings",
                    file_name: file,
                }
            })?;
            Ok((header, value))
        })
        .collect::<Result<HeaderMap, DescriptorParseError>>()?;
    Ok(SerialGetter {
        url,
        method,
        format,
        paths,
        headers,
    })
}

pub fn parse_pinger(
    jval: serde_json::Value,
    file: &'static str,
) -> Result<Pinger, DescriptorParseError> {
    let url = get_attr!(
        jval,
        "url",
        "the url to use, will interpolate $ADDR$ with the address of the target",
        file,
        as_str
    );
    let url = Url::parse(&url).map_err(|_| DescriptorParseError::ParseAttribute {
        name: "url",
        hint: "a string that can be parsed as a url",
        file_name: file,
    })?;
    let method = get_attr_default!(
        jval,
        "method",
        "the method to use to ping the target",
        file,
        as_str,
        "GET"
    );
    let method = match method.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        "CONNECT" => Method::CONNECT,
        "PATCH" => Method::PATCH,
        _ => {
            return Err(DescriptorParseError::ParseAttribute {
                name: "method",
                hint: "a string that can be parsed as a method",
                file_name: file,
            })
        }
    };
    let headers = get_attr_default!(
        jval,
        "headers",
        "Additional headers to send with the request",
        file,
        as_object,
        serde_json::Map::new()
    )
    .iter()
    .map(|(key, value)| {
        value
            .as_str()
            .ok_or(DescriptorParseError::ParseAttribute {
                name: "headers",
                hint: "an object of strings",
                file_name: file,
            })
            .map(|s| (key.to_owned(), s.to_owned()))
    })
    .collect::<Result<HashMap<String, String>, DescriptorParseError>>()?;
    let headers = headers
        .iter()
        .map(|(key, value)| {
            let mut header =
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|_| {
                    DescriptorParseError::ParseAttribute {
                        name: "headers",
                        hint: "an object of strings",
                        file_name: file,
                    }
                })?;
            let value = reqwest::header::HeaderValue::from_str(value).map_err(|_| {
                DescriptorParseError::ParseAttribute {
                    name: "headers",
                    hint: "an object of strings",
                    file_name: file,
                }
            })?;
            Ok((header, value))
        })
        .collect::<Result<HeaderMap, DescriptorParseError>>()?;
    Ok(Pinger {
        url,
        method,
        headers,
    })
}
