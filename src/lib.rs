mod state;
mod todo_handlers;

use state::SharedData;
use todo_handlers::{create_todo, delete_todo, get_all_todos, index, todoby_id, update_todo};
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Shared data is accessible across requests
    let shared_data = SharedData {
        name: "Rust Worker".to_string(),
    };

    // Create a new router with the shared data
    let router = Router::with_data(shared_data);

    router
        .get("/", index)
        .get("/shared-data", |_, ctx| {
            // Get the shared data from the context. This is available because we used with_data above.
            let shared_data = ctx.data.name;
            // Return the response
            Response::ok(shared_data)
        })
        .post_async("/todo", create_todo)
        .delete_async("/todo/:id", delete_todo)
        .put_async("/todo/:id", update_todo)
        .get_async("/todo/:id", todoby_id)
        .get_async("/todo", get_all_todos)
        .run(req, env)
        .await
}
