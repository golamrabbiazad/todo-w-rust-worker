use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use worker::*;

// This is a shared data struct that we will pass to the router
struct SharedData {
    name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Todo {
    id: u8,
    name: String,
    description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TodoUpdate {
    name: String,
    description: String,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Shared data is accessible across requests
    let shared_data = SharedData {
        name: "Rust Worker".to_string(),
    };

    // Create a new router with the shared data
    let router = Router::with_data(shared_data);

    // Router definition
    router
        .get("/", |_, _ctx| Response::ok("Todo Api"))
        .get("/shared-data", |_, ctx| {
            // Get the shared data from the context. This is available because we used with_data above.
            let shared_data = ctx.data.name;
            // Return the response
            Response::ok(shared_data)
        })
        .post_async("/todo", |mut req, ctx| async move {
            // Get the KV namespace
            let kv = ctx.kv("Todo_KV")?;
            // Get the body of the request - Note that Todo implements Deserialize
            let body = req.json::<Todo>().await?;

            // Serialize the body to a string
            let value = to_string(&body)?;
            // Store the value in KV
            kv.put(&body.id.to_string(), value)?.execute().await?;
            // Return the response
            Response::from_json(&body)
        })
        .delete_async("/todo/:id", |_req, ctx| async move {
            // Get the id from the request, we use if let to check if the id exists
            if let Some(id) = ctx.param("id") {
                // Get the KV namespace
                let kv = ctx.kv("Todo_KV")?;
                // Delete the value from KV. In this case,
                // we use the id as the key and return a match statement in case of an error.
                return match kv.delete(id).await {
                    // ! NOTE: I could not find a way to return a 204 No Content response, so this has an empty body.
                    Ok(_) => Response::ok("").map(|resp| resp.with_status(204)),
                    Err(e) => Response::error(e.to_string(), 404),
                };
            }
            Response::error("Todo not found", 404)
        })
        .put_async("/todo/:id", |mut req, ctx| async move {
            // Get the id from the request, we use if let to check if the id exists
            if let Some(id) = ctx.param("id") {
                // Get the KV namespace
                let kv = ctx.kv("Todo_KV")?;
                // Get the body of the request - Note that TodoUpdate implements Deserialize
                let body = req.json::<TodoUpdate>().await?;
                // Check to see if the id exists in KV
                if kv.get(id).json::<Todo>().await?.is_none() {
                    // If the id does not exist, return an error
                    return Response::error("Todo not found", 404);
                }

                // Create a new Todo struct from the body and id
                let new_todo = Todo {
                    id: id.parse::<u8>().unwrap(),
                    name: body.name,
                    description: body.description,
                };

                // Serialize new_todo to a string
                let value = to_string(&new_todo)?;
                // Store the value in KV
                kv.put(&id, value)?.execute().await?;
                // Return the response
                return Response::from_json(&new_todo);
            }
            Response::error("Todo not found", 404)
        })
        .get_async("/todo/:id", |_req, ctx| async move {
            // Get the id from the request, we use if let to check if the id exists
            if let Some(id) = ctx.param("id") {
                // Get the KV namespace
                let kv = ctx.kv("Todo_KV")?;
                // Get the value from KV. In this case,
                // we use the id as the key and return a match statement because the value may not exist.
                return match kv.get(id).json::<Todo>().await? {
                    Some(todo) => Response::from_json(&todo),
                    None => Response::error("Todo not found", 404),
                };
            }
            Response::error("Todo not found", 404)
        })
        .get_async("/todo", |_req, ctx| async move {
            // Get the KV namespace
            let kv = ctx.kv("Todo_KV")?;

            // Get all the keys from KV
            let keys = kv.list().execute().await?.keys;

            // Create a Vec of only the key names
            let key_names = keys
                .into_iter()
                .map(|key| key.name)
                .collect::<Vec<String>>();

            console_debug!("{:?}", key_names);

            let futures = key_names.iter().map(|key| kv.get(key).json::<Todo>());

            // Wait for all the futures to complete. This is similar to Promise.all in JavaScript.
            let todos = join_all(futures)
                .await
                .into_iter()
                .filter_map(|todo| todo.ok())
                .collect::<Vec<_>>()
                .into_iter()
                .map(|todo| todo)
                .collect::<Vec<_>>();

            // Create a response from the todos Vec, wrapped in a Result type.
            let final_result = Response::from_json(&todos);
            console_debug!("Final Result: \n {:?}", &final_result);

            final_result
        })
        .run(req, env)
        .await
}
