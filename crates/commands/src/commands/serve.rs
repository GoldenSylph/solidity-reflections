use crate::utils::{remark, success};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use clap::Parser;
use reflections_core::{config::Paths, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

/// Serve Swagger UI for collected ABIs
#[derive(Debug, Clone, Parser, bon::Builder)]
#[allow(clippy::duplicated_attributes)]
#[builder(on(String, into))]
#[clap(after_help = "For more information, read the README.md")]
#[non_exhaustive]
pub struct Serve {
    /// Path to the collected ABIs JSON file
    #[arg(short, long, default_value = "abis.json")]
    #[builder(default)]
    pub input: String,

    /// Port to serve on
    #[arg(short, long, default_value = "3000")]
    #[builder(default = 3000)]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    #[builder(default)]
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedABI {
    pub contract_name: String,
    pub file_path: String,
    pub abi: serde_json::Value,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABICollection {
    pub grouped: HashMap<String, Vec<CollectedABI>>,
    pub ungrouped: Vec<CollectedABI>,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPISpec {
    openapi: String,
    info: OpenAPIInfo,
    paths: HashMap<String, HashMap<String, OpenAPIPath>>,
    components: OpenAPIComponents,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPIInfo {
    title: String,
    version: String,
    description: String,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPIPath {
    summary: String,
    description: String,
    #[serde(rename = "operationId")]
    operation_id: String,
    responses: HashMap<String, OpenAPIResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Vec<OpenAPIParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "requestBody")]
    request_body: Option<OpenAPIRequestBody>,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPIParameter {
    name: String,
    #[serde(rename = "in")]
    location: String,
    required: bool,
    schema: OpenAPISchema,
    description: String,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPIRequestBody {
    required: bool,
    content: HashMap<String, OpenAPIContent>,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPIContent {
    schema: OpenAPISchema,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPIResponse {
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<HashMap<String, OpenAPIContent>>,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPISchema {
    #[serde(rename = "type")]
    schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Box<OpenAPISchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, OpenAPISchema>>,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAPIComponents {
    schemas: HashMap<String, OpenAPISchema>,
}

#[derive(Clone)]
struct AppState {
    openapi_spec: Arc<OpenAPISpec>,
}

pub(crate) async fn serve_command(paths: &Paths, cmd: Serve) -> Result<()> {
    let input_path = paths.root.join(&cmd.input);

    if !input_path.exists() {
        return Err(reflections_core::ReflectionsError::IOError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "ABIs file not found: {}. Run 'reflections collect' first.",
                cmd.input
            ),
        )));
    }

    remark!("Loading ABIs from {}", cmd.input);
    let content = fs::read_to_string(&input_path).map_err(|e| {
        reflections_core::ReflectionsError::IOError(std::io::Error::new(
            e.kind(),
            format!("Failed to read {}: {}", cmd.input, e),
        ))
    })?;

    let collection: ABICollection = serde_json::from_str(&content).map_err(|e| {
        reflections_core::ReflectionsError::IOError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse {}: {}", cmd.input, e),
        ))
    })?;

    remark!("Generating OpenAPI specification");
    let openapi_spec = generate_openapi_spec(&collection);

    let state = AppState {
        openapi_spec: Arc::new(openapi_spec),
    };

    let app = Router::new()
        .route("/", get(serve_swagger_ui))
        .route("/openapi.json", get(serve_openapi_spec))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", cmd.host, cmd.port)
        .parse()
        .expect("Invalid host/port combination");

    success!("Swagger UI available at http://{}", addr);
    remark!("Press Ctrl+C to stop the server");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| {
            reflections_core::ReflectionsError::IOError(std::io::Error::new(
                e.kind(),
                format!("Failed to bind to {addr}: {e}"),
            ))
        })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| {
            reflections_core::ReflectionsError::IOError(std::io::Error::other(
                format!("Server error: {e}"),
            ))
        })?;

    Ok(())
}

async fn serve_swagger_ui() -> impl IntoResponse {
    Html(SWAGGER_UI_HTML)
}

async fn serve_openapi_spec(State(state): State<AppState>) -> impl IntoResponse {
    Json((*state.openapi_spec).clone())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

fn generate_openapi_spec(collection: &ABICollection) -> OpenAPISpec {
    let mut paths = HashMap::new();
    let mut schemas = HashMap::new();

    // Process grouped contracts
    for (group_name, contracts) in &collection.grouped {
        for contract in contracts {
            process_contract(&mut paths, &mut schemas, contract, Some(group_name));
        }
    }

    // Process ungrouped contracts
    for contract in &collection.ungrouped {
        process_contract(&mut paths, &mut schemas, contract, None);
    }

    OpenAPISpec {
        openapi: "3.0.0".to_string(),
        info: OpenAPIInfo {
            title: "Solidity Contracts API".to_string(),
            version: "1.0.0".to_string(),
            description: "Auto-generated API documentation from Solidity contract ABIs".to_string(),
        },
        paths,
        components: OpenAPIComponents { schemas },
    }
}

fn process_contract(
    paths: &mut HashMap<String, HashMap<String, OpenAPIPath>>,
    _schemas: &mut HashMap<String, OpenAPISchema>,
    contract: &CollectedABI,
    group: Option<&String>,
) {
    let abi_array = match contract.abi.as_array() {
        Some(arr) => arr,
        None => return,
    };

    for item in abi_array {
        let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
        
        if item_type == "function" {
            let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            let state_mutability = item
                .get("stateMutability")
                .and_then(|s| s.as_str())
                .unwrap_or("nonpayable");

            let inputs = item.get("inputs").and_then(|i| i.as_array());
            let outputs = item.get("outputs").and_then(|o| o.as_array());

            let path_prefix = if let Some(g) = group {
                format!("/contracts/{}/{}", g, contract.contract_name)
            } else {
                format!("/contracts/{}", contract.contract_name)
            };
            let path = format!("{path_prefix}/{name}");

            // Determine HTTP method based on state mutability
            let method = match state_mutability {
                "view" | "pure" => "get",
                _ => "post",
            };

            let mut parameters = Vec::new();
            let mut request_properties = HashMap::new();
            
            if let Some(inputs_array) = inputs {
                for (i, input) in inputs_array.iter().enumerate() {
                    let param_name = input
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or(&format!("param{i}"))
                        .to_string();
                    
                    let param_type = input
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("string");

                    let schema = abi_type_to_openapi_schema(param_type);
                    
                    if method == "get" {
                        parameters.push(OpenAPIParameter {
                            name: param_name,
                            location: "query".to_string(),
                            required: true,
                            schema,
                            description: format!("Parameter of type {param_type}"),
                        });
                    } else {
                        request_properties.insert(param_name, schema);
                    }
                }
            }

            let mut responses = HashMap::new();
            
            if let Some(outputs_array) = outputs {
                if !outputs_array.is_empty() {
                    let mut response_properties = HashMap::new();
                    for (i, output) in outputs_array.iter().enumerate() {
                        let out_name = output
                            .get("name")
                            .and_then(|n| n.as_str())
                            .filter(|s| !s.is_empty())
                            .unwrap_or(&format!("output{i}"))
                            .to_string();
                        
                        let out_type = output
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("string");

                        response_properties.insert(out_name, abi_type_to_openapi_schema(out_type));
                    }

                    let mut content = HashMap::new();
                    content.insert(
                        "application/json".to_string(),
                        OpenAPIContent {
                            schema: OpenAPISchema {
                                schema_type: "object".to_string(),
                                items: None,
                                properties: Some(response_properties),
                            },
                        },
                    );

                    responses.insert(
                        "200".to_string(),
                        OpenAPIResponse {
                            description: "Successful response".to_string(),
                            content: Some(content),
                        },
                    );
                } else {
                    responses.insert(
                        "200".to_string(),
                        OpenAPIResponse {
                            description: "Successful response".to_string(),
                            content: None,
                        },
                    );
                }
            } else {
                responses.insert(
                    "200".to_string(),
                    OpenAPIResponse {
                        description: "Successful response".to_string(),
                        content: None,
                    },
                );
            }

            let openapi_path = OpenAPIPath {
                summary: format!("{} - {}", contract.contract_name, name),
                description: format!(
                    "Call {} function on {} contract ({})",
                    name, contract.contract_name, state_mutability
                ),
                operation_id: format!("{}_{}", contract.contract_name, name),
                responses,
                parameters: if !parameters.is_empty() { Some(parameters) } else { None },
                request_body: if !request_properties.is_empty() {
                    let mut content = HashMap::new();
                    content.insert(
                        "application/json".to_string(),
                        OpenAPIContent {
                            schema: OpenAPISchema {
                                schema_type: "object".to_string(),
                                items: None,
                                properties: Some(request_properties),
                            },
                        },
                    );
                    Some(OpenAPIRequestBody {
                        required: true,
                        content,
                    })
                } else {
                    None
                },
            };

            paths
                .entry(path)
                .or_default()
                .insert(method.to_string(), openapi_path);
        }
    }
}

fn abi_type_to_openapi_schema(abi_type: &str) -> OpenAPISchema {
    if let Some(element_type) = abi_type.strip_suffix("[]") {
        // Array type
        OpenAPISchema {
            schema_type: "array".to_string(),
            items: Some(Box::new(abi_type_to_openapi_schema(element_type))),
            properties: None,
        }
    } else if abi_type.starts_with("uint") || abi_type.starts_with("int") {
        OpenAPISchema {
            schema_type: "integer".to_string(),
            items: None,
            properties: None,
        }
    } else if abi_type == "bool" {
        OpenAPISchema {
            schema_type: "boolean".to_string(),
            items: None,
            properties: None,
        }
    } else if abi_type == "address" || abi_type.starts_with("bytes") || abi_type == "string" {
        OpenAPISchema {
            schema_type: "string".to_string(),
            items: None,
            properties: None,
        }
    } else {
        // Default to string for unknown types
        OpenAPISchema {
            schema_type: "string".to_string(),
            items: None,
            properties: None,
        }
    }
}

const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Solidity Contracts API</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.10.3/swagger-ui.css">
    <style>
        body {
            margin: 0;
            padding: 0;
        }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.3/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.3/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            window.ui = SwaggerUIBundle({
                url: '/openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>"#;
