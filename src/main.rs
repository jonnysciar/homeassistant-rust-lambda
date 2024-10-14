use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use reqwest;
use serde_json::{json, Value};
use std::env;

static BASE_URL: &str = "BASE_URL";
static TOKEN: &str = "LONG_LIVED_ACCESS_TOKEN";

async fn function_handler(
    event: LambdaEvent<Value>,
    client: &reqwest::Client
) -> Result<Value, Error> {
    let base_url = match env::var(BASE_URL) {
        Ok(s) => format!("{}/api/alexa/smart_home", s),
        _ => return Ok(json!({
            "event": {
                "payload": {
                    "error": "Failed to retrieve base url"
                }
            }
        })),
    };
    let request = event.payload;
    let token = request
        .get("directive")
        .and_then(|directive| {
            if directive.get("header")?.get("payloadVersion")?.as_str()? == "3" {
                return Some(directive);
            }
            None
        })
        .and_then(|directive| {
            let scope = directive
                .get("endpoint")
                .and_then(|endpoint| endpoint.get("scope"))
                .or_else(|| directive.get("payload")?.get("grantee"))
                .or_else(|| directive.get("payload")?.get("scope"))
                .and_then(|scope| {
                    if scope.get("type")?.as_str()? == "BearerToken" {
                        return Some(scope);
                    }
                    None
                });

            scope?.get("token")?.as_str().map(|t| t.to_string())
        })
        .or_else(|| env::var(TOKEN).ok());

    if token.is_none() {
        let error_response = json!({
            "event": {
                "payload": {
                    "error": "Failed to retrieve token"
                }
            }
        });

        return Ok(error_response);
    }

    let response = client
        .post(base_url)
        .header("Authorization", format!("Bearer {}", token.unwrap()))
        .header("Content-Type", "application/json")
        .json(&request).send()
        .await?;

    let status_code = response.status().as_u16();
    if status_code >= 400 {
        let error_response = json!({
            "event": {
                "payload": {
                    "type": if status_code == 401 || status_code == 403 {"INVALID_AUTHORIZATION_CREDENTIAL"} else {"INTERNAL_ERROR"},
                    "message": response.text().await?
                }
            }
        });

        return Ok(error_response);
    }

    return Ok(response.json().await?);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let client = reqwest::Client::new();

    return run(service_fn(|e| {
        function_handler(e, &client)
    }))
    .await;
}

#[cfg(test)]
mod tests {
    use crate::function_handler;
    use lambda_runtime::{Context, LambdaEvent};
    use serde_json::json;

    #[tokio::test]
    async fn response_is_good_for_simple_input() {
        let id = "ID";

        let mut context = Context::default();
        context.request_id = id.to_string();

        let payload = json!({
            "directive": {
                "header": {
                    "namespace": "Alexa.Discovery",
                    "name": "Discover",
                    "payloadVersion": "3",
                    "messageId": "1bd5d003-31b9-476f-ad03-71d471922820"
                },
                "payload": {
                    "scope": {
                        "type": "BearerToken"
                    }
                }
            }
        });

        let client = reqwest::Client::new();
        let event = LambdaEvent { payload, context };
        let result = function_handler(event, &client).await.unwrap();
        println!("{result}");
    }
}
