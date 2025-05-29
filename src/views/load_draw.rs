// use crate::components::{Echo, Hero};
use dioxus::prelude::*;
use crate::DB;
use crate::Route;
use std::collections::HashMap;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Load_Draw() -> Element {

    let conn = DB.lock().unwrap();

        let mut draws = Vec::new();
        if let Ok(mut stmt) = conn.prepare("SELECT id, name, last_modified FROM draw ORDER BY last_modified DESC") {
            let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
            });
            if let Ok(rows) = rows {
            for row in rows.flatten() {
                draws.push(row);
            }
            }
        }
        println!("Draws: {}", draws.len());


        let mut values = use_signal(HashMap::new);


        rsx! {
            form {
            oninput: move |ev| {
                values.set(ev.values());
            },

                for (sd_id, name, last_modified) in draws.iter() {
                    div {
                    key: "{sd_id}",
                    label {
                        input {
                        r#type: "radio",
                        name: "draw",
                        value: sd_id.to_string(),
                        }
                        "{name}"
                    }
                }
            }

            button {
                r#type: "button",
                onclick: move |_| {
                    // Process the selected draw here
                    let binding = values.read();
                    let sd_id = binding.get("draw").unwrap().first().unwrap().parse::<i64>().unwrap();
    
                    println!("Selected draw ID: {}", &sd_id);
                    use_navigator().replace(Route::Enter_Scores { sd_id: sd_id });




                },
                "Load Draw"
            }
        }
    }
}
